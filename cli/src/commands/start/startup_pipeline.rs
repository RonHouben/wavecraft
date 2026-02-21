use anyhow::{Context, Result};
use command_group::{CommandGroup, GroupChild};
use console::style;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tokio::sync::watch;

use crate::project::{
    read_engine_package_name,
    ts_codegen::{write_parameter_types, write_processor_types},
    ProjectMarkers,
};
use wavecraft_bridge::IpcHandler;
use wavecraft_dev_server::{DevServerHost, DevSession, RebuildCallbacks, WsServer};
use wavecraft_protocol::{ParameterInfo, ProcessorInfo};

pub(super) fn parse_allow_no_audio_env(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn allow_no_audio_runtime_fallback() -> bool {
    std::env::var(super::ALLOW_NO_AUDIO_ENV)
        .map(|value| parse_allow_no_audio_env(&value))
        .unwrap_or(false)
}

/// Run both development servers.
pub(super) fn run_dev_servers(project: &ProjectMarkers, ws_port: u16, ui_port: u16) -> Result<()> {
    println!();
    println!(
        "{}",
        style("Starting Wavecraft Development Servers")
            .cyan()
            .bold()
    );
    println!();

    super::preflight::ensure_ports_available(ws_port, ui_port)?;

    // 1. Build the plugin and load parameters (two-phase or cached)
    // Create tokio runtime for async parameter loading
    let runtime = tokio::runtime::Runtime::new().context("Failed to create async runtime")?;
    let metadata = runtime.block_on(super::metadata_cache::load_plugin_metadata(
        &project.engine_dir,
    ))?;
    let params = metadata.params;
    let processors = metadata.processors;

    write_parameter_types(&project.ui_dir, &params)
        .context("Failed to generate TypeScript parameter ID types")?;
    write_processor_types(&project.ui_dir, &processors)
        .context("Failed to generate TypeScript processor ID types")?;

    for param in &params {
        println!(
            "  - {}: {} ({})",
            param.id,
            param.name,
            param.group.as_deref().unwrap_or("ungrouped")
        );
    }

    // 2. Create AtomicParameterBridge for lock-free audio-thread param reads
    #[cfg(feature = "audio-dev")]
    let param_bridge = {
        use wavecraft_dev_server::AtomicParameterBridge;
        std::sync::Arc::new(AtomicParameterBridge::new(&params))
    };

    // 3. Start embedded WebSocket server
    println!(
        "{} Starting WebSocket server on port {}...",
        style("→").cyan(),
        ws_port
    );

    #[cfg(feature = "audio-dev")]
    let host = DevServerHost::with_param_bridge(params, std::sync::Arc::clone(&param_bridge));
    #[cfg(not(feature = "audio-dev"))]
    let host = DevServerHost::new(params);

    let host = std::sync::Arc::new(host);
    let handler = std::sync::Arc::new(IpcHandler::new(host.clone()));

    // Start WebSocket server (runtime already created above for param loading)
    let server = std::sync::Arc::new(WsServer::new(ws_port, handler.clone()));
    runtime.block_on(async { server.start().await.map_err(|e| anyhow::anyhow!("{}", e)) })?;

    println!("{} WebSocket server running", style("✓").green());

    // Create shutdown broadcast channel
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // 4. Initialize hot-reload development session
    println!("{} Setting up hot-reload...", style("→").cyan());

    // Create rebuild callbacks: wire CLI-specific functions into the dev-server pipeline
    let callbacks = build_rebuild_callbacks(project);

    let dev_session = runtime.block_on(async {
        DevSession::new(
            project.engine_dir.clone(),
            host.clone(),
            server.clone(),
            shutdown_rx,
            callbacks,
            #[cfg(feature = "audio-dev")]
            None, // Audio handle will be added if audio starts
        )
    })?;
    let watched_path = project.engine_dir.join("src");
    let relative_path = watched_path
        .strip_prefix(std::env::current_dir().unwrap_or_default())
        .unwrap_or(&watched_path);
    println!(
        "{} Watching {} for changes",
        style("👀").cyan(),
        relative_path.display()
    );
    println!();

    // 5. Start audio in-process via FFI (strict in SDK dev mode)
    // Store the AudioHandle so the cpal stream stays alive until shutdown.
    // When this variable is dropped (reverse declaration order for locals),
    // the FfiProcessor inside the closure is dropped while the Library in
    // `runtime_loader` is still loaded — preserving vtable pointer validity.
    #[cfg(feature = "audio-dev")]
    let (audio_handle, _runtime_loader) = super::audio_runtime::start_audio_runtime(
        &runtime,
        &project.engine_dir,
        host.clone(),
        server.handle(),
        param_bridge.clone(),
        allow_no_audio_runtime_fallback(),
    )?;
    #[cfg(feature = "audio-dev")]
    let has_audio = audio_handle.is_some();
    #[cfg(not(feature = "audio-dev"))]
    let has_audio = false;

    // 6. Start UI dev server
    println!(
        "{} Starting UI dev server on port {}...",
        style("→").cyan(),
        ui_port
    );

    let ui_server = start_ui_dev_server(project, ui_port)?;

    // Print success message
    println!();
    println!("{}", style("✓ All servers running!").green().bold());
    println!();
    println!("  WebSocket: ws://127.0.0.1:{}", ws_port);
    println!("  UI:        http://localhost:{}", ui_port);
    if has_audio {
        println!("  Audio:     Real-time OS input (in-process FFI)");
    } else if allow_no_audio_runtime_fallback() {
        println!(
            "  Audio:     Disabled (degraded mode via {}=1)",
            super::ALLOW_NO_AUDIO_ENV
        );
    }
    println!();
    println!("{}", style("Press Ctrl+C to stop").dim());
    println!();

    // Wait for shutdown (keeps runtime alive)
    let shutdown_reason = super::shutdown::wait_for_shutdown(ui_server, shutdown_tx)?;

    #[cfg(feature = "audio-dev")]
    drop(audio_handle);
    drop(dev_session);
    drop(runtime);

    match shutdown_reason {
        super::shutdown::ShutdownReason::UiExited(status) => Err(anyhow::anyhow!(
            "UI dev server exited unexpectedly with status {}",
            status
        )),
        super::shutdown::ShutdownReason::UiExitedUnknown => {
            Err(anyhow::anyhow!("UI dev server exited unexpectedly"))
        }
        super::shutdown::ShutdownReason::CtrlC | super::shutdown::ShutdownReason::ChannelClosed => {
            Ok(())
        }
    }
}

fn build_rebuild_callbacks(project: &ProjectMarkers) -> RebuildCallbacks {
    RebuildCallbacks {
        package_name: read_engine_package_name(&project.engine_dir),
        write_sidecar: Some(std::sync::Arc::new(
            |engine_dir: &Path, params: &[ParameterInfo]| {
                super::metadata_cache::write_sidecar_cache(engine_dir, params)
            },
        )),
        write_ts_types: Some(std::sync::Arc::new({
            let ui_dir = project.ui_dir.clone();
            move |params: &[ParameterInfo]| write_parameter_types(&ui_dir, params)
        })),
        write_processor_ts_types: Some(std::sync::Arc::new({
            let ui_dir = project.ui_dir.clone();
            move |processors: &[ProcessorInfo]| write_processor_types(&ui_dir, processors)
        })),
        param_loader: std::sync::Arc::new(move |engine_dir: PathBuf| {
            Box::pin(super::reload_extractors::load_parameters_from_dylib(
                engine_dir,
            ))
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<Vec<ParameterInfo>>> + Send>,
                >
        }),
        processor_loader: Some(std::sync::Arc::new(move |engine_dir: PathBuf| {
            Box::pin(super::reload_extractors::load_processors_from_dylib(
                engine_dir,
            ))
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<Vec<ProcessorInfo>>> + Send>,
                >
        })),
    }
}

fn start_ui_dev_server(project: &ProjectMarkers, ui_port: u16) -> Result<GroupChild> {
    let ui_port_str = format!("--port={}", ui_port);
    let mut ui_server = Command::new("npm")
        .args(["run", "dev", "--", &ui_port_str, "--strictPort"])
        .current_dir(&project.ui_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .group_spawn()
        .context("Failed to start UI dev server")?;

    // Give the UI server a moment to fail fast (e.g., port already in use).
    thread::sleep(Duration::from_millis(300));
    if let Some(status) = ui_server
        .try_wait()
        .context("Failed to check UI dev server status")?
    {
        anyhow::bail!("UI dev server exited early with status {}", status);
    }

    Ok(ui_server)
}

#[cfg(test)]
mod tests {
    use super::parse_allow_no_audio_env;

    #[test]
    fn parse_allow_no_audio_env_accepts_opt_in_values() {
        assert!(parse_allow_no_audio_env("1"));
        assert!(parse_allow_no_audio_env("true"));
        assert!(parse_allow_no_audio_env("YES"));
        assert!(parse_allow_no_audio_env(" on "));
    }

    #[test]
    fn parse_allow_no_audio_env_rejects_non_opt_in_values() {
        assert!(!parse_allow_no_audio_env("0"));
        assert!(!parse_allow_no_audio_env("false"));
        assert!(!parse_allow_no_audio_env(""));
    }
}
