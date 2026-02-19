#[cfg(feature = "audio-dev")]
use anyhow::Result;
#[cfg(feature = "audio-dev")]
use console::style;
#[cfg(feature = "audio-dev")]
use std::path::Path;
#[cfg(feature = "audio-dev")]
use std::sync::Arc;

#[cfg(feature = "audio-dev")]
use crate::project::find_plugin_dylib;
#[cfg(feature = "audio-dev")]
use wavecraft_dev_server::{AtomicParameterBridge, DevServerHost, WsHandle};
#[cfg(feature = "audio-dev")]
use wavecraft_protocol::{AudioDiagnosticCode, AudioRuntimePhase, AudioRuntimeStatus};

#[cfg(feature = "audio-dev")]
pub(super) struct AudioStartupSuccess {
    handle: wavecraft_dev_server::AudioHandle,
    sample_rate: f32,
    buffer_size: u32,
}

#[cfg(feature = "audio-dev")]
pub(super) struct AudioStartupFailure {
    code: AudioDiagnosticCode,
    message: String,
    hint: Option<&'static str>,
}

#[cfg(feature = "audio-dev")]
pub(super) fn status_for_running_audio(sample_rate: f32, buffer_size: u32) -> AudioRuntimeStatus {
    wavecraft_dev_server::audio_status(
        AudioRuntimePhase::RunningFullDuplex,
        Some(sample_rate),
        Some(buffer_size),
    )
}

#[cfg(feature = "audio-dev")]
pub(super) fn classify_audio_init_error(
    error_text: &str,
) -> (AudioDiagnosticCode, Option<&'static str>) {
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
pub(super) fn classify_runtime_loader_error(
    error_text: &str,
) -> (AudioDiagnosticCode, Option<&'static str>) {
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

/// Try to start audio processing in-process via FFI vtable.
///
/// - If successful, returns the started audio handle and mode details.
/// - If initialization fails, returns a structured diagnostic code and message.
#[cfg(feature = "audio-dev")]
fn try_start_audio_in_process(
    loader: &super::PluginLoader,
    host: Arc<DevServerHost>,
    ws_handle: WsHandle,
    param_bridge: Arc<AtomicParameterBridge>,
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
                super::ALLOW_NO_AUDIO_ENV
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
                super::ALLOW_NO_AUDIO_ENV
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
                super::ALLOW_NO_AUDIO_ENV
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
fn load_runtime_plugin_loader(engine_dir: &Path) -> Result<super::PluginLoader> {
    let dylib_path = match find_plugin_dylib(engine_dir) {
        Ok(path) => path,
        Err(error) => {
            anyhow::bail!(
                "Unable to locate plugin library for audio runtime: {:#}",
                error
            );
        }
    };

    match super::PluginLoader::load(&dylib_path) {
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

#[cfg(feature = "audio-dev")]
pub(super) fn start_audio_runtime(
    runtime: &tokio::runtime::Runtime,
    engine_dir: &Path,
    host: Arc<DevServerHost>,
    ws_handle: WsHandle,
    param_bridge: Arc<AtomicParameterBridge>,
    allow_no_audio: bool,
) -> Result<(
    Option<wavecraft_dev_server::AudioHandle>,
    Option<super::PluginLoader>,
)> {
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

    let runtime_loader = match load_runtime_plugin_loader(engine_dir) {
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
                        code, message, super::ALLOW_NO_AUDIO_ENV
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
                        style(format!("⚠ Failed to broadcast audio status: {}", error)).yellow()
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
                        style(format!("⚠ Failed to broadcast audio status: {}", error)).yellow()
                    );
                }

                if allow_no_audio {
                    println!(
                        "{}",
                        style(format!(
                            "⚠ Audio runtime disabled ({:?}): {}. Continuing in degraded mode because {}=1.",
                            failure.code,
                            failure.message,
                            super::ALLOW_NO_AUDIO_ENV
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

    Ok((audio_handle, runtime_loader))
}

#[cfg(all(test, feature = "audio-dev"))]
mod tests {
    use super::{
        classify_audio_init_error, classify_runtime_loader_error, status_for_running_audio,
    };
    use wavecraft_protocol::{AudioDiagnosticCode, AudioRuntimePhase};

    #[test]
    fn classify_audio_init_error_maps_permission_denied() {
        let (code, hint) = classify_audio_init_error("Microphone permission denied by system");
        assert_eq!(code, AudioDiagnosticCode::InputPermissionDenied);
        assert!(hint.is_some());
    }

    #[test]
    fn classify_audio_init_error_maps_no_input_device() {
        let (code, hint) = classify_audio_init_error("No input device available");
        assert_eq!(code, AudioDiagnosticCode::NoInputDevice);
        assert!(hint.is_some());
    }

    #[test]
    fn classify_audio_init_error_defaults_to_unknown() {
        let (code, hint) = classify_audio_init_error("backend crashed with opaque cpal error");
        assert_eq!(code, AudioDiagnosticCode::Unknown);
        assert!(hint.is_none());
    }

    #[test]
    fn classify_runtime_loader_error_maps_vtable_missing() {
        let (code, hint) = classify_runtime_loader_error(
            "Symbol not found: wavecraft_dev_create_processor: dlsym failed",
        );
        assert_eq!(code, AudioDiagnosticCode::VtableMissing);
        assert!(hint.is_some());
    }

    #[test]
    fn classify_runtime_loader_error_maps_loader_unavailable() {
        let (code, hint) = classify_runtime_loader_error(
            "Failed to load plugin runtime from /tmp/libplugin.dylib: image not found",
        );
        assert_eq!(code, AudioDiagnosticCode::LoaderUnavailable);
        assert!(hint.is_some());
    }

    #[test]
    fn status_for_running_audio_marks_full_duplex_as_running() {
        let status = status_for_running_audio(48_000.0, 256);
        assert_eq!(status.phase, AudioRuntimePhase::RunningFullDuplex);
        assert!(status.diagnostic.is_none());
        assert_eq!(status.sample_rate, Some(48_000.0));
        assert_eq!(status.buffer_size, Some(256));
    }

    #[test]
    fn status_for_running_audio_does_not_degrade_when_running() {
        let status = status_for_running_audio(44_100.0, 512);
        assert_eq!(status.phase, AudioRuntimePhase::RunningFullDuplex);
        assert!(status.diagnostic.is_none());
        assert_eq!(status.sample_rate, Some(44_100.0));
        assert_eq!(status.buffer_size, Some(512));
    }

    #[test]
    fn classify_audio_init_error_maps_no_output_device() {
        let (code, hint) = classify_audio_init_error("No output device available");
        assert_eq!(code, AudioDiagnosticCode::NoOutputDevice);
        assert!(hint.is_some());
    }
}
