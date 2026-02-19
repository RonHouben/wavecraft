//! Development server command - starts WebSocket + UI dev servers.
//!
//! This command provides the development experience for wavecraft plugins:
//! 1. Builds the plugin in debug mode
//! 2. Loads parameter metadata via FFI from the compiled dylib
//! 3. Loads and validates the audio processor vtable, then runs audio in-process
//! 4. Starts an embedded WebSocket server for browser UI communication
//! 5. Starts the Vite dev server for UI hot-reloading

use anyhow::{Context, Result};
use command_group::{CommandGroup, GroupChild};
use console::style;
use regex::Regex;
use std::io::{self, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};
use tokio::sync::watch;
use walkdir::WalkDir;

use crate::project::{
    find_plugin_dylib, has_node_modules, read_engine_package_name, resolve_debug_dir,
    ts_codegen::{write_parameter_types, write_processor_types},
    ProjectMarkers,
};
use wavecraft_bridge::IpcHandler;
use wavecraft_dev_server::{DevServerHost, DevSession, RebuildCallbacks, WsServer};
use wavecraft_protocol::{AudioDiagnosticCode, AudioRuntimePhase, ParameterInfo, ProcessorInfo};

/// Re-export PluginParamLoader for audio dev mode
use wavecraft_bridge::PluginParamLoader as PluginLoader;

/// Options for the `start` command.
#[derive(Debug)]
pub struct StartCommand {
    /// WebSocket server port
    pub port: u16,
    /// Vite UI server port
    pub ui_port: u16,
    /// Auto-install dependencies without prompting
    pub install: bool,
    /// Fail if dependencies are missing (no prompt)
    pub no_install: bool,
}

const SDK_TSCONFIG_PATHS_MARKER: &str =
    r#""@wavecraft/core": ["../../ui/packages/core/src/index.ts"]"#;
const ALLOW_NO_AUDIO_ENV: &str = "WAVECRAFT_ALLOW_NO_AUDIO";
const SDK_TSCONFIG_PATHS_SNIPPET: &str = r#"    /* SDK development — resolve @wavecraft packages from monorepo source */
    "baseUrl": ".",
    "paths": {
      "@wavecraft/core": ["../../ui/packages/core/src/index.ts"],
      "@wavecraft/core/*": ["../../ui/packages/core/src/*"],
      "@wavecraft/components": ["../../ui/packages/components/src/index.ts"],
      "@wavecraft/components/*": ["../../ui/packages/components/src/*"]
    }"#;

#[derive(Debug, PartialEq, Eq)]
enum TsconfigPathsInjection {
    Updated(String),
    Unchanged,
    Warning(&'static str),
}

impl StartCommand {
    pub fn execute(&self) -> Result<()> {
        // 1. Detect project
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let project = ProjectMarkers::detect(&cwd)?;

        // 2. Check dependencies
        if !has_node_modules(&project) {
            if self.no_install {
                anyhow::bail!(
                    "Dependencies not installed. Run `npm install` in the ui/ directory,\n\
                     or use `wavecraft start --install` to install automatically."
                );
            }

            let should_install = if self.install {
                true
            } else {
                prompt_install()?
            };

            if should_install {
                install_dependencies(&project)?;
            } else {
                anyhow::bail!("Cannot start without dependencies. Run `npm install` in ui/ first.");
            }
        }

        // 3. Start servers
        ensure_sdk_ui_paths_for_typescript(&project)?;
        run_dev_servers(&project, self.port, self.ui_port)
    }
}

fn find_object_bounds_after_key(content: &str, key: &str) -> Option<(usize, usize)> {
    let key_start = content.find(key)?;
    let bytes = content.as_bytes();
    let mut index = key_start + key.len();

    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }

    if index >= bytes.len() || bytes[index] != b':' {
        return None;
    }
    index += 1;

    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }

    while index < bytes.len() && bytes[index] != b'{' {
        index += 1;
    }

    if index >= bytes.len() || bytes[index] != b'{' {
        return None;
    }

    let open_index = index;
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut is_escaped = false;
    let mut cursor = open_index;

    while cursor < bytes.len() {
        let ch = bytes[cursor];

        if in_string {
            if is_escaped {
                is_escaped = false;
            } else if ch == b'\\' {
                is_escaped = true;
            } else if ch == b'"' {
                in_string = false;
            }
            cursor += 1;
            continue;
        }

        if ch == b'"' {
            in_string = true;
            cursor += 1;
            continue;
        }

        if ch == b'/' && cursor + 1 < bytes.len() {
            let next = bytes[cursor + 1];
            if next == b'/' {
                cursor += 2;
                while cursor < bytes.len() && bytes[cursor] != b'\n' {
                    cursor += 1;
                }
                continue;
            }

            if next == b'*' {
                cursor += 2;
                while cursor + 1 < bytes.len() {
                    if bytes[cursor] == b'*' && bytes[cursor + 1] == b'/' {
                        cursor += 2;
                        break;
                    }
                    cursor += 1;
                }
                continue;
            }
        }

        if ch == b'{' {
            depth += 1;
        } else if ch == b'}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some((open_index, cursor));
            }
        }

        cursor += 1;
    }

    None
}

fn apply_sdk_tsconfig_paths(content: &str) -> Result<TsconfigPathsInjection> {
    if !content.contains("\"compilerOptions\"") {
        return Ok(TsconfigPathsInjection::Warning(
            "could not inject SDK TypeScript paths: `compilerOptions` block not found",
        ));
    }

    if content.contains(SDK_TSCONFIG_PATHS_MARKER) {
        return Ok(TsconfigPathsInjection::Unchanged);
    }

    let (compiler_options_start, compiler_options_end) =
        match find_object_bounds_after_key(content, "\"compilerOptions\"") {
            Some(bounds) => bounds,
            None => return Ok(TsconfigPathsInjection::Warning(
                "could not inject SDK TypeScript paths: failed to locate `compilerOptions` object",
            )),
        };

    let compiler_options_content = &content[compiler_options_start + 1..compiler_options_end];
    if compiler_options_content.contains("\"paths\"") {
        return Ok(TsconfigPathsInjection::Warning(
            "could not auto-inject SDK TypeScript paths: `compilerOptions.paths` already exists, please add @wavecraft mappings manually",
        ));
    }

    let anchor_re = Regex::new(
        r#"\"(noFallthroughCasesInSwitch|noUnusedParameters|noUnusedLocals|strict|jsx|noEmit|moduleResolution|target)\"\s*:\s*[^\n]*"#,
    )
    .context("Invalid regex for tsconfig anchor detection")?;

    if let Some(anchor) = anchor_re.find(compiler_options_content) {
        let anchor_start = compiler_options_start + 1 + anchor.start();
        let anchor_end = compiler_options_start + 1 + anchor.end();
        let anchor_text = &content[anchor_start..anchor_end];
        let needs_comma = !anchor_text.trim_end().ends_with(',');
        let comma = if needs_comma { "," } else { "" };
        let has_following_properties =
            has_jsonc_property_after_anchor(&content[anchor_end..compiler_options_end]);

        let mut updated = String::with_capacity(content.len() + 256);
        updated.push_str(&content[..anchor_end]);
        updated.push_str(comma);
        updated.push_str("\n\n");
        updated.push_str(SDK_TSCONFIG_PATHS_SNIPPET);
        if has_following_properties {
            updated.push(',');
        }
        updated.push_str(&content[anchor_end..]);

        return Ok(TsconfigPathsInjection::Updated(updated));
    }

    let trimmed = compiler_options_content.trim_end();
    let has_properties = trimmed.contains('"') && trimmed.contains(':');
    let needs_comma = has_properties && !trimmed.ends_with(',');
    let comma = if needs_comma { "," } else { "" };

    let mut updated = String::with_capacity(content.len() + 256);
    updated.push_str(&content[..compiler_options_end]);
    updated.push_str(comma);
    updated.push_str("\n\n");
    updated.push_str(SDK_TSCONFIG_PATHS_SNIPPET);
    updated.push('\n');
    updated.push_str(&content[compiler_options_end..]);

    Ok(TsconfigPathsInjection::Updated(updated))
}

fn has_jsonc_property_after_anchor(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        while index < bytes.len() && (bytes[index].is_ascii_whitespace() || bytes[index] == b',') {
            index += 1;
        }

        if index >= bytes.len() {
            return false;
        }

        if bytes[index] == b'/' && index + 1 < bytes.len() {
            if bytes[index + 1] == b'/' {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                continue;
            }

            if bytes[index + 1] == b'*' {
                index += 2;
                while index + 1 < bytes.len() {
                    if bytes[index] == b'*' && bytes[index + 1] == b'/' {
                        index += 2;
                        break;
                    }
                    index += 1;
                }
                continue;
            }
        }

        return bytes[index] == b'"';
    }

    false
}

fn ensure_sdk_ui_paths_for_typescript(project: &ProjectMarkers) -> Result<()> {
    if !project.sdk_mode {
        return Ok(());
    }

    let tsconfig_path = project.ui_dir.join("tsconfig.json");
    if !tsconfig_path.is_file() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&tsconfig_path)
        .with_context(|| format!("Failed to read {}", tsconfig_path.display()))?;

    match apply_sdk_tsconfig_paths(&content)? {
        TsconfigPathsInjection::Updated(updated) => {
            std::fs::write(&tsconfig_path, updated)
                .with_context(|| format!("Failed to write {}", tsconfig_path.display()))?;

            println!(
                "{} Enabled SDK TypeScript path mappings in {}",
                style("✓").green(),
                tsconfig_path.display()
            );
        }
        TsconfigPathsInjection::Unchanged => {}
        TsconfigPathsInjection::Warning(message) => {
            println!("{} {}", style("⚠").yellow(), message);
            println!(
                "  Add @wavecraft path mappings manually in {} if needed.",
                tsconfig_path.display()
            );
        }
    }

    Ok(())
}

/// Prompt user to install dependencies.
fn prompt_install() -> Result<bool> {
    print!(
        "{} Dependencies not installed. Run npm install? [Y/n] ",
        style("?").cyan().bold()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    Ok(response.is_empty() || response == "y" || response == "yes")
}

fn parse_allow_no_audio_env(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn allow_no_audio_runtime_fallback() -> bool {
    std::env::var(ALLOW_NO_AUDIO_ENV)
        .map(|value| parse_allow_no_audio_env(&value))
        .unwrap_or(false)
}

/// Try to start audio processing in-process via FFI vtable.
///
/// - If successful, returns the started audio handle and mode details.
/// - If initialization fails, returns a structured diagnostic code and message.
#[cfg(feature = "audio-dev")]
struct AudioStartupSuccess {
    handle: wavecraft_dev_server::AudioHandle,
    sample_rate: f32,
    buffer_size: u32,
}

#[cfg(feature = "audio-dev")]
struct AudioStartupFailure {
    code: AudioDiagnosticCode,
    message: String,
    hint: Option<&'static str>,
}

#[cfg(feature = "audio-dev")]
fn status_for_running_audio(
    sample_rate: f32,
    buffer_size: u32,
) -> wavecraft_protocol::AudioRuntimeStatus {
    wavecraft_dev_server::audio_status(
        AudioRuntimePhase::RunningFullDuplex,
        Some(sample_rate),
        Some(buffer_size),
    )
}

#[cfg(feature = "audio-dev")]
fn classify_audio_init_error(error_text: &str) -> (AudioDiagnosticCode, Option<&'static str>) {
    let lower = error_text.to_lowercase();

    if lower.contains("permission") || lower.contains("denied") {
        (
            AudioDiagnosticCode::InputPermissionDenied,
            Some("Grant microphone access to Terminal/host app in macOS Privacy settings."),
        )
    } else if lower.contains("no output device") || lower.contains("default output config") {
        (
            AudioDiagnosticCode::NoOutputDevice,
            Some("Ensure a default system output device is available and enabled, then retry `wavecraft start`."),
        )
    } else if lower.contains("no input device") {
        (
            AudioDiagnosticCode::NoInputDevice,
            Some("Connect/enable an input device and retry `wavecraft start`."),
        )
    } else {
        (AudioDiagnosticCode::Unknown, None)
    }
}

#[cfg(feature = "audio-dev")]
fn classify_runtime_loader_error(error_text: &str) -> (AudioDiagnosticCode, Option<&'static str>) {
    let lower = error_text.to_lowercase();

    if lower.contains("wavecraft_dev_create_processor")
        || lower.contains("vtable")
        || lower.contains("version mismatch")
    {
        (
            AudioDiagnosticCode::VtableMissing,
            Some(
                "Rebuild the plugin with current SDK dev exports and ensure dev processor vtable symbols are present.",
            ),
        )
    } else {
        (
            AudioDiagnosticCode::LoaderUnavailable,
            Some("Ensure the plugin dylib is built and loadable, then retry `wavecraft start`."),
        )
    }
}

#[cfg(feature = "audio-dev")]
fn try_start_audio_in_process(
    loader: &PluginLoader,
    host: std::sync::Arc<DevServerHost>,
    ws_handle: wavecraft_dev_server::WsHandle,
    param_bridge: std::sync::Arc<wavecraft_dev_server::AtomicParameterBridge>,
) -> Result<AudioStartupSuccess, AudioStartupFailure> {
    use wavecraft_dev_server::{AudioConfig, AudioServer, FfiProcessor};

    println!();
    println!("{} Checking for audio processor...", style("→").cyan());

    let vtable = loader.dev_processor_vtable();
    println!("{} Audio processor vtable loaded", style("✓").green());

    let processor = match FfiProcessor::new(vtable) {
        Some(p) => p,
        None => {
            println!(
                "{}",
                style("⚠ Failed to create audio processor (create returned null)").yellow()
            );
            println!(
                "  Audio runtime startup failed (strict mode aborts; set {}=1 to continue without audio).",
                ALLOW_NO_AUDIO_ENV
            );
            println!();
            return Err(AudioStartupFailure {
                code: AudioDiagnosticCode::ProcessorCreateFailed,
                message: "Audio processor create() returned null".to_string(),
                hint: Some("Check the processor constructor and FFI vtable exports in the plugin."),
            });
        }
    };

    let config = AudioConfig {
        sample_rate: 44100.0,
        buffer_size: 512,
    };
    let target_sample_rate = config.sample_rate;
    let target_buffer_size = config.buffer_size;

    let server = match AudioServer::new(Box::new(processor), config, param_bridge) {
        Ok(s) => s,
        Err(e) => {
            let error_text = e.to_string();
            let (code, hint) = classify_audio_init_error(&error_text);

            println!(
                "{}",
                style(format!("⚠ Audio init failed: {:#}", e)).yellow()
            );

            println!(
                "  Audio runtime startup failed (strict mode aborts; set {}=1 to continue without audio).",
                ALLOW_NO_AUDIO_ENV
            );

            return Err(AudioStartupFailure {
                code,
                message: error_text,
                hint,
            });
        }
    };

    // Start audio server. Returns lock-free ring buffer consumers for
    // meter and oscilloscope data (RT-safe: audio thread writes without allocations).
    let (handle, mut meter_consumer, mut oscilloscope_consumer) = match server.start() {
        Ok((h, meter, oscilloscope)) => (h, meter, oscilloscope),
        Err(e) => {
            println!(
                "{}",
                style(format!("⚠ Failed to start audio: {}", e)).yellow()
            );
            println!(
                "  Audio runtime startup failed (strict mode aborts; set {}=1 to continue without audio).",
                ALLOW_NO_AUDIO_ENV
            );

            return Err(AudioStartupFailure {
                code: AudioDiagnosticCode::StreamStartFailed,
                message: e.to_string(),
                hint: Some("Check current audio device availability and retry."),
            });
        }
    };

    // Spawn a task that drains the lock-free meter ring buffer and
    // forwards updates to WebSocket clients.
    tokio::spawn(async move {
        use wavecraft_protocol::{IpcNotification, NOTIFICATION_METER_UPDATE};

        let mut interval = tokio::time::interval(std::time::Duration::from_millis(16));
        loop {
            interval.tick().await;
            // Drain all available meter frames, keeping only the latest.
            let mut latest = None;
            while let Ok(notification) = meter_consumer.pop() {
                latest = Some(notification);
            }
            if let Some(notification) = latest {
                host.set_latest_meter_frame(&notification);

                if let Ok(json) = serde_json::to_string(&IpcNotification::new(
                    NOTIFICATION_METER_UPDATE,
                    notification,
                )) {
                    ws_handle.broadcast(&json).await;
                }
            }

            if let Some(frame) = oscilloscope_consumer.read_latest() {
                host.set_latest_oscilloscope_frame(frame.to_protocol_frame());
            }
        }
    });

    println!(
        "{} Audio server started — full-duplex (input + output)",
        style("✓").green()
    );
    println!();

    Ok(AudioStartupSuccess {
        handle,
        sample_rate: target_sample_rate,
        buffer_size: target_buffer_size,
    })
}

/// Load plugin runtime for audio startup independently from metadata cache path.
#[cfg(feature = "audio-dev")]
fn load_runtime_plugin_loader(engine_dir: &Path) -> Result<PluginLoader> {
    let dylib_path = match find_plugin_dylib(engine_dir) {
        Ok(path) => path,
        Err(error) => {
            anyhow::bail!(
                "Unable to locate plugin library for audio runtime: {:#}",
                error
            );
        }
    };

    match PluginLoader::load(&dylib_path) {
        Ok(loader) => Ok(loader),
        Err(error) => {
            println!(
                "{}",
                style(format!(
                    "⚠ Failed to load plugin runtime from {}: {:#}",
                    dylib_path.display(),
                    error
                ))
                .yellow()
            );

            anyhow::bail!(
                "Failed to load plugin runtime from {}: {:#}",
                dylib_path.display(),
                error
            )
        }
    }
}

/// Install npm dependencies in the ui/ directory.
fn install_dependencies(project: &ProjectMarkers) -> Result<()> {
    println!("{} Installing dependencies...", style("→").cyan());

    let status = Command::new("npm")
        .args(["install"])
        .current_dir(&project.ui_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run npm install. Is npm installed and in your PATH?")?;

    if !status.success() {
        anyhow::bail!("npm install failed. Please check the output above and try again.");
    }

    println!("{} Dependencies installed", style("✓").green());
    Ok(())
}

/// Path to the sidecar parameter cache file.
const PARAM_SIDECAR_FILENAME: &str = "wavecraft-params.json";
const PROCESSOR_SIDECAR_FILENAME: &str = "wavecraft-processors.json";

#[derive(Debug, Clone)]
struct PluginMetadata {
    params: Vec<ParameterInfo>,
    processors: Vec<ProcessorInfo>,
}

fn sidecar_json_path(engine_dir: &Path, file_name: &str) -> Result<PathBuf> {
    let debug_dir = resolve_debug_dir(engine_dir)?;
    Ok(debug_dir.join(file_name))
}

fn params_sidecar_json_path(engine_dir: &Path) -> Result<PathBuf> {
    sidecar_json_path(engine_dir, PARAM_SIDECAR_FILENAME)
}

fn processors_sidecar_json_path(engine_dir: &Path) -> Result<PathBuf> {
    sidecar_json_path(engine_dir, PROCESSOR_SIDECAR_FILENAME)
}

/// Try reading cached parameters from the sidecar JSON file.
///
/// Returns `Some(params)` if the file exists and is newer than the dylib
/// (i.e., no source changes since last extraction). Returns `None` otherwise.
fn try_read_cached_params(engine_dir: &Path) -> Option<Vec<ParameterInfo>> {
    let sidecar_path = params_sidecar_json_path(engine_dir).ok()?;
    if !sidecar_path.exists() {
        return None;
    }

    // Check if sidecar is still valid.
    //
    // A sidecar is valid only when it is newer than:
    // - the compiled dylib currently used for extraction
    // - the newest file under engine/src (source edits before rebuild)
    // - the currently running CLI binary (cache format/logic migrations)
    let dylib_path = find_plugin_dylib(engine_dir).ok()?;
    let sidecar_mtime = std::fs::metadata(&sidecar_path).ok()?.modified().ok()?;
    let dylib_mtime = std::fs::metadata(&dylib_path).ok()?.modified().ok()?;

    if dylib_mtime > sidecar_mtime {
        println!("  Sidecar cache stale (dylib newer), rebuilding...");

        return None;
    }

    if let Some(src_mtime) = newest_file_mtime_under(&engine_dir.join("src")) {
        if src_mtime > sidecar_mtime {
            println!("  Sidecar cache stale (engine source newer), rebuilding...");

            return None;
        }
    }

    if let Some(cli_mtime) = current_exe_mtime() {
        if cli_mtime > sidecar_mtime {
            println!("  Sidecar cache stale (CLI binary newer), rebuilding...");

            return None;
        }
    }

    // Load parameters from JSON file (inline to avoid publish dep issues)
    let contents = std::fs::read_to_string(&sidecar_path).ok()?;
    serde_json::from_str(&contents).ok()
}

/// Try reading cached processors from sidecar JSON file.
fn try_read_cached_processors(engine_dir: &Path) -> Option<Vec<ProcessorInfo>> {
    let sidecar_path = processors_sidecar_json_path(engine_dir).ok()?;
    if !sidecar_path.exists() {
        return None;
    }

    let dylib_path = find_plugin_dylib(engine_dir).ok()?;
    let sidecar_mtime = std::fs::metadata(&sidecar_path).ok()?.modified().ok()?;
    let dylib_mtime = std::fs::metadata(&dylib_path).ok()?.modified().ok()?;

    if dylib_mtime > sidecar_mtime {
        println!("  Processor sidecar cache stale (dylib newer), rebuilding...");
        return None;
    }

    if let Some(src_mtime) = newest_file_mtime_under(&engine_dir.join("src")) {
        if src_mtime > sidecar_mtime {
            println!("  Processor sidecar cache stale (engine source newer), rebuilding...");
            return None;
        }
    }

    if let Some(cli_mtime) = current_exe_mtime() {
        if cli_mtime > sidecar_mtime {
            println!("  Processor sidecar cache stale (CLI binary newer), rebuilding...");
            return None;
        }
    }

    let contents = std::fs::read_to_string(&sidecar_path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn newest_file_mtime_under(root: &Path) -> Option<SystemTime> {
    if !root.is_dir() {
        return None;
    }

    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| entry.metadata().ok())
        .filter_map(|metadata| metadata.modified().ok())
        .max()
}

fn current_exe_mtime() -> Option<SystemTime> {
    let current_exe = std::env::current_exe().ok()?;
    std::fs::metadata(current_exe).ok()?.modified().ok()
}

/// Write parameter metadata to the sidecar JSON cache.
pub(crate) fn write_sidecar_cache(engine_dir: &Path, params: &[ParameterInfo]) -> Result<()> {
    let sidecar_path = params_sidecar_json_path(engine_dir)?;
    let json = serde_json::to_string_pretty(params).context("Failed to serialize parameters")?;
    std::fs::write(&sidecar_path, json).context("Failed to write sidecar cache")?;
    Ok(())
}

fn write_processors_sidecar_cache(engine_dir: &Path, processors: &[ProcessorInfo]) -> Result<()> {
    let sidecar_path = processors_sidecar_json_path(engine_dir)?;
    let json =
        serde_json::to_string_pretty(processors).context("Failed to serialize processors")?;
    std::fs::write(&sidecar_path, json).context("Failed to write processor sidecar cache")?;
    Ok(())
}

/// Load plugin metadata (parameters + processors) using cached sidecars or
/// feature-gated discovery build.
async fn load_plugin_metadata(engine_dir: &Path) -> Result<PluginMetadata> {
    // 1. Try cached sidecars
    if let (Some(params), Some(processors)) = (
        try_read_cached_params(engine_dir),
        try_read_cached_processors(engine_dir),
    ) {
        println!(
            "{} Loaded {} parameters and {} processors (cached)",
            style("✓").green(),
            params.len(),
            processors.len()
        );
        return Ok(PluginMetadata { params, processors });
    }

    // 2. Build with _param-discovery feature (skip nih-plug exports)
    println!("{} Building for metadata discovery...", style("→").cyan());

    let mut build_cmd = Command::new("cargo");
    build_cmd.args(["build", "--lib", "--features", "_param-discovery"]);

    if let Some(package_name) = read_engine_package_name(engine_dir) {
        build_cmd.args(["--package", &package_name]);
    }

    let build_result = build_cmd
        .current_dir(engine_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    let status = build_result.context("Failed to run cargo build for metadata discovery")?;
    if !status.success() {
        anyhow::bail!(
            "Metadata discovery build failed. This project must support --features _param-discovery in SDK dev mode."
        );
    }

    let dylib_path = find_plugin_dylib(engine_dir)
        .context("Failed to find plugin library after discovery build")?;

    println!("  Found dylib: {}", dylib_path.display());

    println!("{} Loading plugin metadata...", style("→").cyan());
    #[cfg(feature = "audio-dev")]
    let (params, processors) = {
        let loader = PluginLoader::load(&dylib_path)
            .context("Failed to load plugin for metadata discovery")?;
        (loader.parameters().to_vec(), loader.processors().to_vec())
    };
    #[cfg(not(feature = "audio-dev"))]
    let (params, processors) = {
        let params = crate::project::param_extract::extract_params_subprocess(
            &dylib_path,
            crate::project::param_extract::DEFAULT_EXTRACT_TIMEOUT,
        )
        .await
        .context("Failed to extract parameters from plugin")?;
        let processors = crate::project::param_extract::extract_processors_subprocess(
            &dylib_path,
            crate::project::param_extract::DEFAULT_EXTRACT_TIMEOUT,
        )
        .await
        .context("Failed to extract processors from plugin")?;
        (params, processors)
    };

    if let Err(e) = write_sidecar_cache(engine_dir, &params) {
        println!("  Warning: failed to write param cache: {}", e);
    }
    if let Err(e) = write_processors_sidecar_cache(engine_dir, &processors) {
        println!("  Warning: failed to write processor cache: {}", e);
    }

    println!(
        "{} Loaded {} parameters and {} processors",
        style("✓").green(),
        params.len(),
        processors.len()
    );

    Ok(PluginMetadata { params, processors })
}

/// Run both development servers.
fn run_dev_servers(project: &ProjectMarkers, ws_port: u16, ui_port: u16) -> Result<()> {
    println!();
    println!(
        "{}",
        style("Starting Wavecraft Development Servers")
            .cyan()
            .bold()
    );
    println!();

    ensure_port_available(ws_port, "WebSocket server", "--port")?;
    ensure_port_available(ui_port, "UI dev server", "--ui-port")?;

    // 1. Build the plugin and load parameters (two-phase or cached)
    // Create tokio runtime for async parameter loading
    let runtime = tokio::runtime::Runtime::new().context("Failed to create async runtime")?;
    let metadata = runtime.block_on(load_plugin_metadata(&project.engine_dir))?;
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
    let callbacks = RebuildCallbacks {
        package_name: read_engine_package_name(&project.engine_dir),
        write_sidecar: Some(std::sync::Arc::new(
            |engine_dir: &Path, params: &[ParameterInfo]| write_sidecar_cache(engine_dir, params),
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
            Box::pin(load_parameters_from_dylib(engine_dir))
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<Vec<ParameterInfo>>> + Send>,
                >
        }),
        processor_loader: Some(std::sync::Arc::new(move |engine_dir: PathBuf| {
            Box::pin(load_processors_from_dylib(engine_dir))
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<Vec<ProcessorInfo>>> + Send>,
                >
        })),
    };

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
    let (audio_handle, _runtime_loader) = {
        let ws_handle = server.handle();
        let allow_no_audio = allow_no_audio_runtime_fallback();

        let initializing_status =
            wavecraft_dev_server::audio_status(AudioRuntimePhase::Initializing, None, None);

        host.set_audio_status(initializing_status.clone());

        if let Err(error) =
            runtime.block_on(ws_handle.broadcast_audio_status_changed(&initializing_status))
        {
            println!(
                "{}",
                style(format!(
                    "⚠ Failed to broadcast audio init status: {}",
                    error
                ))
                .yellow()
            );
        }

        let runtime_loader = match load_runtime_plugin_loader(&project.engine_dir) {
            Ok(loader) => Some(loader),
            Err(error) => {
                let message = error.to_string();
                let (code, hint) = classify_runtime_loader_error(&message);
                let status = wavecraft_dev_server::audio_status_with_diagnostic(
                    AudioRuntimePhase::Failed,
                    code,
                    message.clone(),
                    hint,
                    None,
                    None,
                );

                host.set_audio_status(status.clone());
                if let Err(broadcast_error) =
                    runtime.block_on(ws_handle.broadcast_audio_status_changed(&status))
                {
                    println!(
                        "{}",
                        style(format!(
                            "⚠ Failed to broadcast audio status: {}",
                            broadcast_error
                        ))
                        .yellow()
                    );
                }

                if allow_no_audio {
                    println!(
                        "{}",
                        style(format!(
                            "⚠ Audio runtime disabled ({:?}): {}. Continuing in degraded mode because {}=1.",
                            code, message, ALLOW_NO_AUDIO_ENV
                        ))
                        .yellow()
                    );
                    None
                } else {
                    anyhow::bail!("Audio startup failed ({:?}): {}", code, message);
                }
            }
        };

        let audio_handle = if let Some(runtime_loader) = runtime_loader.as_ref() {
            match runtime.block_on(async {
                try_start_audio_in_process(
                    runtime_loader,
                    host.clone(),
                    ws_handle.clone(),
                    param_bridge.clone(),
                )
            }) {
                Ok(started) => {
                    let status = status_for_running_audio(started.sample_rate, started.buffer_size);
                    host.set_audio_status(status.clone());
                    if let Err(error) =
                        runtime.block_on(ws_handle.broadcast_audio_status_changed(&status))
                    {
                        println!(
                            "{}",
                            style(format!("⚠ Failed to broadcast audio status: {}", error))
                                .yellow()
                        );
                    }

                    Some(started.handle)
                }
                Err(failure) => {
                    let status = wavecraft_dev_server::audio_status_with_diagnostic(
                        AudioRuntimePhase::Failed,
                        failure.code,
                        failure.message.clone(),
                        failure.hint,
                        None,
                        None,
                    );
                    host.set_audio_status(status.clone());
                    if let Err(error) =
                        runtime.block_on(ws_handle.broadcast_audio_status_changed(&status))
                    {
                        println!(
                            "{}",
                            style(format!("⚠ Failed to broadcast audio status: {}", error))
                                .yellow()
                        );
                    }

                    if allow_no_audio {
                        println!(
                            "{}",
                            style(format!(
                                "⚠ Audio runtime disabled ({:?}): {}. Continuing in degraded mode because {}=1.",
                                failure.code, failure.message, ALLOW_NO_AUDIO_ENV
                            ))
                            .yellow()
                        );
                        None
                    } else {
                        anyhow::bail!(
                            "Audio startup failed ({:?}): {}",
                            failure.code,
                            failure.message
                        );
                    }
                }
            }
        } else {
            None
        };

        (audio_handle, runtime_loader)
    };
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
            ALLOW_NO_AUDIO_ENV
        );
    }
    println!();
    println!("{}", style("Press Ctrl+C to stop").dim());
    println!();

    // Wait for shutdown (keeps runtime alive)
    let shutdown_reason = wait_for_shutdown(ui_server, shutdown_tx)?;

    #[cfg(feature = "audio-dev")]
    drop(audio_handle);
    drop(dev_session);
    drop(runtime);

    match shutdown_reason {
        ShutdownReason::UiExited(status) => Err(anyhow::anyhow!(
            "UI dev server exited unexpectedly with status {}",
            status
        )),
        ShutdownReason::UiExitedUnknown => {
            Err(anyhow::anyhow!("UI dev server exited unexpectedly"))
        }
        ShutdownReason::CtrlC | ShutdownReason::ChannelClosed => Ok(()),
    }
}

fn ensure_port_available(port: u16, label: &str, flag: &str) -> Result<()> {
    match TcpListener::bind(("0.0.0.0", port)) {
        Ok(listener) => {
            drop(listener);
            Ok(())
        }
        Err(err) => anyhow::bail!(
            "{} port {} is already in use ({}). Stop the process using it or run `wavecraft start {}` with a free port.",
            label,
            port,
            err,
            flag
        ),
    }
}

/// Set up Ctrl+C handler and wait for shutdown.
///
/// Audio runs in-process (via FFI) on the tokio runtime's thread pool,
/// so dropping the runtime is sufficient to stop audio. Only the UI
/// child process needs explicit cleanup.
#[derive(Debug)]
enum ShutdownReason {
    CtrlC,
    UiExited(i32),
    UiExitedUnknown,
    ChannelClosed,
}

fn wait_for_shutdown(
    mut ui_server: GroupChild,
    shutdown_tx: watch::Sender<bool>,
) -> Result<ShutdownReason> {
    let (tx, rx) = mpsc::channel();
    let shutdown_tx_for_handler = shutdown_tx.clone();

    ctrlc::set_handler(move || {
        let _ = shutdown_tx_for_handler.send(true);
        let _ = tx.send(());
    })
    .context("Failed to set Ctrl+C handler")?;

    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(_) => {
                println!();
                println!("{} Shutting down servers...", style("→").cyan());
                send_shutdown_signal(&shutdown_tx);
                kill_process(&mut ui_server);
                println!("{} Servers stopped", style("✓").green());
                return Ok(ShutdownReason::CtrlC);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Check UI server
                if let Some(status) = ui_server
                    .try_wait()
                    .context("Failed to check UI dev server status")?
                {
                    println!();
                    println!(
                        "{} UI dev server exited unexpectedly ({}).",
                        style("✗").red(),
                        status
                    );
                    println!("{} Shutting down servers...", style("→").cyan());
                    send_shutdown_signal(&shutdown_tx);
                    println!("{} Servers stopped", style("✓").green());
                    if let Some(code) = status.code() {
                        return Ok(ShutdownReason::UiExited(code));
                    }
                    return Ok(ShutdownReason::UiExitedUnknown);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                println!();
                println!("{} Shutting down servers...", style("→").cyan());
                send_shutdown_signal(&shutdown_tx);
                kill_process(&mut ui_server);
                println!("{} Servers stopped", style("✓").green());
                return Ok(ShutdownReason::ChannelClosed);
            }
        }
    }
}

fn send_shutdown_signal(shutdown_tx: &watch::Sender<bool>) {
    let _ = shutdown_tx.send(true);
}

/// Kill a child process group gracefully.
fn kill_process(child: &mut GroupChild) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Load parameters from the rebuilt dylib via subprocess isolation.
///
/// To avoid dylib caching issues on macOS, we copy the dylib to a unique
/// temporary location before loading. The subprocess extracts parameters
/// and exits cleanly, with the temp file deleted after.
///
/// This function stays in the CLI because it depends on CLI-specific
/// infrastructure: `find_plugin_dylib`, `extract_params_subprocess`,
/// and `create_temp_dylib_copy`.
async fn load_parameters_from_dylib(engine_dir: PathBuf) -> Result<Vec<ParameterInfo>> {
    println!("  {} Finding plugin dylib...", style("→").dim());
    let lib_path =
        find_plugin_dylib(&engine_dir).context("Failed to find plugin dylib after rebuild")?;
    println!("  {} Found: {}", style("→").dim(), lib_path.display());

    println!("  {} Copying to temp location...", style("→").dim());
    let temp_path = create_temp_dylib_copy(&lib_path)?;
    println!("  {} Temp: {}", style("→").dim(), temp_path.display());

    println!(
        "  {} Loading parameters via subprocess...",
        style("→").dim()
    );
    let params = crate::project::param_extract::extract_params_subprocess(
        &temp_path,
        crate::project::param_extract::DEFAULT_EXTRACT_TIMEOUT,
    )
    .await
    .with_context(|| format!("Failed to extract parameters from: {}", temp_path.display()))?;
    println!(
        "  {} Loaded {} parameters via subprocess",
        style("→").dim(),
        params.len()
    );

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    Ok(params)
}

/// Load processors from the rebuilt dylib via subprocess isolation.
async fn load_processors_from_dylib(engine_dir: PathBuf) -> Result<Vec<ProcessorInfo>> {
    println!("  {} Finding plugin dylib...", style("→").dim());
    let lib_path =
        find_plugin_dylib(&engine_dir).context("Failed to find plugin dylib after rebuild")?;
    println!("  {} Found: {}", style("→").dim(), lib_path.display());

    println!("  {} Copying to temp location...", style("→").dim());
    let temp_path = create_temp_dylib_copy(&lib_path)?;
    println!("  {} Temp: {}", style("→").dim(), temp_path.display());

    println!(
        "  {} Loading processors via subprocess...",
        style("→").dim()
    );
    let processors = crate::project::param_extract::extract_processors_subprocess(
        &temp_path,
        crate::project::param_extract::DEFAULT_EXTRACT_TIMEOUT,
    )
    .await
    .with_context(|| format!("Failed to extract processors from: {}", temp_path.display()))?;

    let _ = std::fs::remove_file(&temp_path);

    println!(
        "  {} Loaded {} processors via subprocess",
        style("→").dim(),
        processors.len()
    );

    Ok(processors)
}

/// Create a temporary copy of the dylib with a unique name.
///
/// This ensures libloading loads a fresh dylib rather than returning
/// a cached handle from a previous load of the same path.
fn create_temp_dylib_copy(dylib_path: &Path) -> Result<PathBuf> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let extension = dylib_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("dylib");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    let temp_name = format!("wavecraft_hotreload_{}.{}", timestamp, extension);
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(temp_name);

    std::fs::copy(dylib_path, &temp_path).with_context(|| {
        format!(
            "Failed to copy dylib to temp location: {}",
            temp_path.display()
        )
    })?;

    Ok(temp_path)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{apply_sdk_tsconfig_paths, parse_allow_no_audio_env, TsconfigPathsInjection};

    #[cfg(feature = "audio-dev")]
    use super::classify_audio_init_error;
    #[cfg(feature = "audio-dev")]
    use super::classify_runtime_loader_error;
    #[cfg(feature = "audio-dev")]
    use wavecraft_protocol::{AudioDiagnosticCode, AudioRuntimePhase};

    #[test]
    fn injects_sdk_paths_when_missing() {
        let input = r#"{
    "compilerOptions": {
        "strict": true,
        "noFallthroughCasesInSwitch": true
    }
}"#;

        let output = apply_sdk_tsconfig_paths(input).expect("should parse");

        let TsconfigPathsInjection::Updated(output) = output else {
            panic!("should inject");
        };

        assert!(output.contains(r#""baseUrl": ".""#));
        assert!(output.contains(r#""@wavecraft/core": ["../../ui/packages/core/src/index.ts"]"#));
        assert!(output
            .contains(r#""@wavecraft/components": ["../../ui/packages/components/src/index.ts"]"#));
    }

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

    #[test]
    fn is_idempotent_when_paths_present() {
        let input = r#"{
    "compilerOptions": {
        "noFallthroughCasesInSwitch": true,
        "baseUrl": ".",
        "paths": {
            "@wavecraft/core": ["../../ui/packages/core/src/index.ts"]
        }
    }
}"#;

        let output = apply_sdk_tsconfig_paths(input).expect("should parse");
        assert_eq!(output, TsconfigPathsInjection::Unchanged);
    }

    #[test]
    fn injects_when_primary_anchor_is_missing() {
        let input = r#"{
    "compilerOptions": {
        "strict": true
    }
}"#;

        let output = apply_sdk_tsconfig_paths(input).expect("should parse");

        let TsconfigPathsInjection::Updated(output) = output else {
            panic!("should inject using fallback anchor");
        };

        assert!(output.contains(r#""baseUrl": ".""#));
        assert!(output.contains(r#""paths": {"#));
    }

    #[test]
    fn returns_warning_when_compiler_options_missing() {
        let input = r#"{
    "include": ["src"]
}"#;

        let output = apply_sdk_tsconfig_paths(input).expect("should parse");

        assert_eq!(
            output,
            TsconfigPathsInjection::Warning(
                "could not inject SDK TypeScript paths: `compilerOptions` block not found"
            )
        );
    }

    #[test]
    fn injects_paths_with_trailing_comma_before_following_property() {
        let input = r#"{
    "compilerOptions": {
        "moduleResolution": "bundler",
        "allowSyntheticDefaultImports": true,
        "types": ["node"]
    }
}"#;

        let output = apply_sdk_tsconfig_paths(input).expect("should parse");

        let TsconfigPathsInjection::Updated(output) = output else {
            panic!("should inject");
        };

        assert!(
            output.contains("\"@wavecraft/components/*\": [\"../../ui/packages/components/src/*\"]\n    },\n        \"allowSyntheticDefaultImports\""),
            "Expected trailing comma after injected paths block before following property:\n{}",
            output
        );
        assert!(
            !output.contains("\"@wavecraft/components/*\": [\"../../ui/packages/components/src/*\"]\n    }\n        \"allowSyntheticDefaultImports\""),
            "Injected paths block must not be adjacent to next property without comma:\n{}",
            output
        );
    }

    #[cfg(feature = "audio-dev")]
    #[test]
    fn classify_audio_init_error_maps_permission_denied() {
        let (code, hint) = classify_audio_init_error("Microphone permission denied by system");
        assert_eq!(code, AudioDiagnosticCode::InputPermissionDenied);
        assert!(hint.is_some());
    }

    #[cfg(feature = "audio-dev")]
    #[test]
    fn classify_audio_init_error_maps_no_input_device() {
        let (code, hint) = classify_audio_init_error("No input device available");
        assert_eq!(code, AudioDiagnosticCode::NoInputDevice);
        assert!(hint.is_some());
    }

    #[cfg(feature = "audio-dev")]
    #[test]
    fn classify_audio_init_error_defaults_to_unknown() {
        let (code, hint) = classify_audio_init_error("backend crashed with opaque cpal error");
        assert_eq!(code, AudioDiagnosticCode::Unknown);
        assert!(hint.is_none());
    }

    #[cfg(feature = "audio-dev")]
    #[test]
    fn classify_runtime_loader_error_maps_vtable_missing() {
        let (code, hint) = classify_runtime_loader_error(
            "Symbol not found: wavecraft_dev_create_processor: dlsym failed",
        );
        assert_eq!(code, AudioDiagnosticCode::VtableMissing);
        assert!(hint.is_some());
    }

    #[cfg(feature = "audio-dev")]
    #[test]
    fn classify_runtime_loader_error_maps_loader_unavailable() {
        let (code, hint) = classify_runtime_loader_error(
            "Failed to load plugin runtime from /tmp/libplugin.dylib: image not found",
        );
        assert_eq!(code, AudioDiagnosticCode::LoaderUnavailable);
        assert!(hint.is_some());
    }

    #[cfg(feature = "audio-dev")]
    #[test]
    fn status_for_running_audio_marks_full_duplex_as_running() {
        let status = super::status_for_running_audio(48_000.0, 256);
        assert_eq!(status.phase, AudioRuntimePhase::RunningFullDuplex);
        assert!(status.diagnostic.is_none());
        assert_eq!(status.sample_rate, Some(48_000.0));
        assert_eq!(status.buffer_size, Some(256));
    }

    #[cfg(feature = "audio-dev")]
    #[test]
    fn status_for_running_audio_does_not_degrade_when_running() {
        let status = super::status_for_running_audio(44_100.0, 512);
        assert_eq!(status.phase, AudioRuntimePhase::RunningFullDuplex);
        assert!(status.diagnostic.is_none());
        assert_eq!(status.sample_rate, Some(44_100.0));
        assert_eq!(status.buffer_size, Some(512));
    }

    #[cfg(feature = "audio-dev")]
    #[test]
    fn classify_audio_init_error_maps_no_output_device() {
        let (code, hint) = classify_audio_init_error("No output device available");
        assert_eq!(code, AudioDiagnosticCode::NoOutputDevice);
        assert!(hint.is_some());
    }

    #[test]
    fn cached_sidecar_path_preserves_full_frequency_range_for_browser_dev_mode() {
        use wavecraft_protocol::{ParameterInfo, ParameterType};

        let temp = tempfile::tempdir().expect("temp dir should be created");
        let engine_dir = temp.path().join("engine");
        let src_dir = engine_dir.join("src");
        let debug_dir = engine_dir.join("target").join("debug");

        fs::create_dir_all(&src_dir).expect("src dir should be created");
        fs::create_dir_all(&debug_dir).expect("debug dir should be created");

        // Build output discovery depends on Cargo.toml + dylib naming convention.
        fs::write(
            engine_dir.join("Cargo.toml"),
            "[package]\nname = \"wavecraft-dev-template\"\n[lib]\nname = \"wavecraft_dev_template\"\n",
        )
        .expect("Cargo.toml should be written");

        fs::write(src_dir.join("lib.rs"), "// test source").expect("source file should be written");

        #[cfg(target_os = "macos")]
        let dylib_name = "libwavecraft_dev_template.dylib";
        #[cfg(target_os = "linux")]
        let dylib_name = "libwavecraft_dev_template.so";
        #[cfg(target_os = "windows")]
        let dylib_name = "wavecraft_dev_template.dll";

        fs::write(debug_dir.join(dylib_name), b"test dylib")
            .expect("dylib placeholder should be written");

        let params = vec![ParameterInfo {
            id: "oscillator_frequency".to_string(),
            name: "Frequency".to_string(),
            param_type: ParameterType::Float,
            value: 440.0,
            default: 440.0,
            min: 20.0,
            max: 5_000.0,
            unit: Some("Hz".to_string()),
            group: Some("Oscillator".to_string()),
            variants: None,
        }];

        super::write_sidecar_cache(&engine_dir, &params).expect("sidecar cache should be written");

        let cached = super::try_read_cached_params(&engine_dir)
            .expect("cached sidecar should be used in start path");

        let frequency = cached
            .iter()
            .find(|param| param.id == "oscillator_frequency")
            .expect("frequency parameter should exist");

        assert!((frequency.min - 20.0).abs() < f32::EPSILON);
        assert!((frequency.max - 5_000.0).abs() < f32::EPSILON);
        assert!((frequency.value - 440.0).abs() < f32::EPSILON);
    }
}
