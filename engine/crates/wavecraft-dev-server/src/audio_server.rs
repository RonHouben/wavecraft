//! Audio server for OS audio input testing in dev mode.
//!
//! This module provides an audio server that processes real microphone input
//! using a `DevAudioProcessor` implementation (typically an `FfiProcessor`
//! loaded from the user's cdylib), communicating meter data back via a
//! callback channel.

#[cfg(feature = "audio")]
pub mod implementation {
    use anyhow::{Context, Result};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{Device, Stream, StreamConfig};
    use wavecraft_protocol::MeterUpdateNotification;

    use crate::ffi_processor::DevAudioProcessor;

    /// Configuration for audio server.
    #[derive(Debug, Clone)]
    pub struct AudioConfig {
        /// Desired sample rate (e.g., 44100.0). Falls back to system default.
        pub sample_rate: f32,
        /// Buffer size in samples.
        pub buffer_size: u32,
    }

    /// Handle returned by `AudioServer::start()` that keeps the audio
    /// stream alive. Drop this handle to stop audio capture.
    pub struct AudioHandle {
        _stream: Stream,
    }

    /// Audio server that processes OS input through a `DevAudioProcessor`.
    pub struct AudioServer {
        processor: Box<dyn DevAudioProcessor>,
        config: AudioConfig,
        device: Device,
        stream_config: StreamConfig,
    }

    impl AudioServer {
        /// Create a new audio server with the given processor and config.
        pub fn new(processor: Box<dyn DevAudioProcessor>, config: AudioConfig) -> Result<Self> {
            let host = cpal::default_host();
            let device = host
                .default_input_device()
                .context("No input device available")?;

            tracing::info!("Using input device: {}", device.name()?);

            let supported_config = device
                .default_input_config()
                .context("Failed to get default input config")?;

            tracing::info!("Using sample rate: {} Hz", supported_config.sample_rate().0);

            let stream_config = supported_config.into();

            Ok(Self {
                processor,
                config,
                device,
                stream_config,
            })
        }

        /// Start audio capture and processing.
        ///
        /// Returns an `AudioHandle` that keeps the stream alive, plus sends
        /// meter updates through the provided sender.
        ///
        /// Drop the handle to stop audio capture.
        pub fn start(
            mut self,
            meter_tx: tokio::sync::mpsc::UnboundedSender<MeterUpdateNotification>,
        ) -> Result<AudioHandle> {
            // Set sample rate from the actual device config
            let actual_sample_rate = self.stream_config.sample_rate.0 as f32;
            self.processor.set_sample_rate(actual_sample_rate);

            // We need to move the processor into the audio callback closure.
            // cpal's callback runs on a dedicated OS audio thread.
            let mut processor = self.processor;
            let _buffer_size = self.config.buffer_size;
            let mut frame_counter = 0u64;

            // Capture the actual channel count from the stream config before
            // moving self.stream_config into build_input_stream. This handles
            // mono (1 ch), stereo (2 ch), and multi-channel (>2 ch) devices.
            let num_channels = self.stream_config.channels as usize;

            let stream = self
                .device
                .build_input_stream(
                    &self.stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        frame_counter += 1;

                        // Convert interleaved cpal input to deinterleaved stereo.
                        // cpal provides interleaved data: [Ch0_S0, Ch1_S0, ..., Ch0_S1, ...]
                        let num_samples = data.len() / num_channels.max(1);

                        if num_samples == 0 || num_channels == 0 {
                            return;
                        }

                        let mut left = vec![0.0f32; num_samples];
                        let mut right = vec![0.0f32; num_samples];

                        for i in 0..num_samples {
                            left[i] = data[i * num_channels];
                            // Stereo or more: use second channel for right.
                            // Mono: duplicate left channel to right.
                            if num_channels > 1 {
                                right[i] = data[i * num_channels + 1];
                            } else {
                                right[i] = left[i];
                            }
                        }

                        // Process through the user's DSP
                        {
                            let mut channels: Vec<&mut [f32]> =
                                vec![left.as_mut_slice(), right.as_mut_slice()];
                            processor.process(&mut channels);
                        }

                        // Compute meters from processed output
                        let peak_left = left.iter().copied().fold(0.0f32, |a, b| a.max(b.abs()));
                        let rms_left =
                            (left.iter().map(|x| x * x).sum::<f32>() / left.len() as f32).sqrt();
                        let peak_right = right.iter().copied().fold(0.0f32, |a, b| a.max(b.abs()));
                        let rms_right =
                            (right.iter().map(|x| x * x).sum::<f32>() / right.len() as f32).sqrt();

                        // Send meter update every ~16ms (60 Hz)
                        if frame_counter % 60 == 0 {
                            let notification = MeterUpdateNotification {
                                timestamp_us: frame_counter,
                                left_peak: peak_left,
                                left_rms: rms_left,
                                right_peak: peak_right,
                                right_rms: rms_right,
                            };
                            // Non-blocking send — if the receiver is behind, drop
                            let _ = meter_tx.send(notification);
                        }
                    },
                    |err| {
                        tracing::error!("Audio stream error: {}", err);
                    },
                    None,
                )
                .context("Failed to build input stream")?;

            stream.play().context("Failed to start audio stream")?;
            tracing::info!("Audio stream started");

            Ok(AudioHandle { _stream: stream })
        }
    }
}

#[cfg(feature = "audio")]
pub use implementation::*;
