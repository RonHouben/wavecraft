//! Audio server for OS audio input testing in dev mode.
//!
//! This module provides a generic audio server that processes real microphone input
//! using a user-provided `Processor` implementation, communicating with the CLI via WebSocket.

#[cfg(feature = "audio")]
pub mod implementation {
    use anyhow::{Context, Result};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{Device, Stream, StreamConfig};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use wavecraft_dsp::Processor;
    use wavecraft_protocol::{MeterUpdateNotification, RegisterAudioParams};

    /// Configuration for audio server
    #[derive(Debug, Clone)]
    pub struct AudioConfig {
        /// WebSocket URL to connect to CLI server
        pub websocket_url: String,
        /// Desired sample rate (e.g., 44100.0)
        pub sample_rate: f32,
        /// Buffer size in samples
        pub buffer_size: u32,
    }

    /// Audio server that processes OS input through a Processor
    pub struct AudioServer<P: Processor> {
        processor: Arc<RwLock<P>>,
        config: AudioConfig,
        device: Device,
        stream_config: StreamConfig,
    }

    impl<P: Processor> AudioServer<P> {
        /// Create a new audio server with the given processor and config
        pub fn new(processor: P, config: AudioConfig) -> Result<Self> {
            // Get default input device
            let host = cpal::default_host();
            let device = host
                .default_input_device()
                .context("No input device available")?;

            tracing::info!("Using input device: {}", device.name()?);

            // Get supported config closest to requested sample rate
            let supported_config = device
                .default_input_config()
                .context("Failed to get default input config")?;

            tracing::info!(
                "Using sample rate: {} Hz",
                supported_config.sample_rate().0
            );

            let stream_config = supported_config.into();

            Ok(Self {
                processor: Arc::new(RwLock::new(processor)),
                config,
                device,
                stream_config,
            })
        }

        /// Run the audio server (blocking)
        pub async fn run(self) -> Result<()> {
            // Connect to WebSocket server
            let ws_client = WebSocketClient::connect(&self.config.websocket_url).await?;

            // Register as audio client
            ws_client
                .register_audio(RegisterAudioParams {
                    client_id: "dev-audio".to_string(),
                    sample_rate: self.stream_config.sample_rate.0 as f32,
                    buffer_size: self.config.buffer_size,
                })
                .await?;

            tracing::info!("Registered with dev server");

            // Build audio stream
            let _processor = Arc::clone(&self.processor);
            let ws_client_clone = ws_client.clone();
            let mut frame_counter = 0u64;

            let stream = self
                .device
                .build_input_stream(
                    &self.stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // Process audio (simplified for now - needs proper buffer handling)
                        frame_counter += 1;

                        // Compute meters (simplified)
                        let peak = data.iter().copied().fold(0.0f32, |a, b| a.max(b.abs()));
                        let rms = (data.iter().map(|x| x * x).sum::<f32>() / data.len() as f32)
                            .sqrt();

                        // Send meter update every ~16ms (60 Hz)
                        if frame_counter % 735 == 0 {
                            let notification = MeterUpdateNotification {
                                timestamp_us: frame_counter,
                                left_peak: peak,
                                left_rms: rms,
                                right_peak: peak,
                                right_rms: rms,
                            };

                            // Non-blocking send (best effort)
                            let ws = ws_client_clone.clone();
                            tokio::spawn(async move {
                                let _ = ws.send_meter_update(notification).await;
                            });
                        }
                    },
                    |err| {
                        tracing::error!("Audio stream error: {}", err);
                    },
                    None,
                )
                .context("Failed to build input stream")?;

            // Start the stream
            stream.play().context("Failed to start audio stream")?;
            tracing::info!("Audio stream started");

            // Keep stream alive and handle parameter updates
            self.handle_parameter_updates(ws_client, stream).await?;

            Ok(())
        }

        async fn handle_parameter_updates(
            &self,
            _ws_client: WebSocketClient,
            _stream: Stream,
        ) -> Result<()> {
            // TODO: Listen for parameter updates from WebSocket and apply to processor
            // For now, just keep running
            tokio::signal::ctrl_c().await?;
            tracing::info!("Shutting down audio server");
            Ok(())
        }
    }

    /// WebSocket client for audio binary
    #[derive(Clone)]
    pub struct WebSocketClient {
        // Placeholder - will implement proper client
        #[allow(dead_code)]
        url: String,
    }

    impl WebSocketClient {
        pub async fn connect(url: &str) -> Result<Self> {
            tracing::info!("Connecting to WebSocket server at {}", url);
            // TODO: Implement actual WebSocket connection
            Ok(Self {
                url: url.to_string(),
            })
        }

        pub async fn register_audio(&self, _params: RegisterAudioParams) -> Result<()> {
            tracing::info!("Registering audio client");
            // TODO: Send registration message
            Ok(())
        }

        pub async fn send_meter_update(&self, _notification: MeterUpdateNotification) -> Result<()> {
            // TODO: Send meter update notification
            Ok(())
        }
    }
}

#[cfg(feature = "audio")]
pub use implementation::*;
