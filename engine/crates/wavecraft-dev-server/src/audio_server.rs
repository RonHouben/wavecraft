//! Audio server for full-duplex audio I/O in dev mode.
//!
//! This module provides an audio server that captures microphone input,
//! processes it through a `DevAudioProcessor` (typically an `FfiProcessor`
//! loaded from the user's cdylib), and sends the processed audio to the
//! output device (speakers/headphones). Meter data is communicated back
//! via a callback channel.
//!
//! # Architecture
//!
//! ```text
//! OS Mic → cpal input callback → deinterleave → FfiProcessor::process()
//!                                                        │
//!                                              ┌─────────┴──────────┐
//!                                              │                    │
//!                                         meter compute      interleave
//!                                              │               → SPSC ring
//!                                              ▼                    │
//!                                        WebSocket broadcast        │
//!                                                                   ▼
//!                                              cpal output callback → Speakers
//! ```

#[cfg(feature = "audio")]
pub mod implementation {
    use std::sync::Arc;

    use anyhow::{Context, Result};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{Device, Stream, StreamConfig};
    use wavecraft_protocol::MeterUpdateNotification;

    use crate::atomic_params::AtomicParameterBridge;
    use crate::ffi_processor::DevAudioProcessor;

    /// Configuration for audio server.
    #[derive(Debug, Clone)]
    pub struct AudioConfig {
        /// Desired sample rate (e.g., 44100.0). Falls back to system default.
        pub sample_rate: f32,
        /// Buffer size in samples.
        pub buffer_size: u32,
    }

    /// Handle returned by `AudioServer::start()` that keeps both audio
    /// streams alive. Drop this handle to stop audio capture and playback.
    pub struct AudioHandle {
        _input_stream: Stream,
        _output_stream: Option<Stream>,
    }

    /// Audio server that processes OS input through a `DevAudioProcessor`
    /// and routes the processed audio to the output device.
    pub struct AudioServer {
        processor: Box<dyn DevAudioProcessor>,
        config: AudioConfig,
        input_device: Device,
        output_device: Option<Device>,
        input_config: StreamConfig,
        output_config: Option<StreamConfig>,
        param_bridge: Arc<AtomicParameterBridge>,
    }

    impl AudioServer {
        /// Create a new audio server with the given processor, config, and
        /// parameter bridge for lock-free audio-thread parameter reads.
        pub fn new(
            processor: Box<dyn DevAudioProcessor>,
            config: AudioConfig,
            param_bridge: Arc<AtomicParameterBridge>,
        ) -> Result<Self> {
            let host = cpal::default_host();

            // Input device (required)
            let input_device = host
                .default_input_device()
                .context("No input device available")?;
            tracing::info!("Using input device: {}", input_device.name()?);

            let supported_input = input_device
                .default_input_config()
                .context("Failed to get default input config")?;
            let input_sample_rate = supported_input.sample_rate().0;
            tracing::info!("Input sample rate: {} Hz", input_sample_rate);
            let input_config: StreamConfig = supported_input.into();

            // Output device (optional — graceful fallback to metering-only)
            let (output_device, output_config) = match host.default_output_device() {
                Some(dev) => {
                    match dev.name() {
                        Ok(name) => tracing::info!("Using output device: {}", name),
                        Err(_) => tracing::info!("Using output device: (unnamed)"),
                    }
                    match dev.default_output_config() {
                        Ok(supported_output) => {
                            let output_sr = supported_output.sample_rate().0;
                            tracing::info!("Output sample rate: {} Hz", output_sr);
                            if output_sr != input_sample_rate {
                                tracing::warn!(
                                    "Input/output sample rate mismatch ({} vs {}). \
                                     Processing at input rate; output device may resample.",
                                    input_sample_rate,
                                    output_sr
                                );
                            }
                            let cfg: StreamConfig = supported_output.into();
                            (Some(dev), Some(cfg))
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to get output config: {}. Metering-only mode.",
                                e
                            );
                            (None, None)
                        }
                    }
                }
                None => {
                    tracing::warn!("No output device available. Metering-only mode.");
                    (None, None)
                }
            };

            Ok(Self {
                processor,
                config,
                input_device,
                output_device,
                input_config,
                output_config,
                param_bridge,
            })
        }

        /// Start audio capture, processing, and playback.
        ///
        /// Returns an `AudioHandle` that keeps both streams alive, plus a
        /// `MeterConsumer` for draining meter frames from a lock-free ring
        /// buffer (RT-safe: no allocations on the audio thread).
        ///
        /// Drop the handle to stop audio.
        pub fn start(mut self) -> Result<(AudioHandle, rtrb::Consumer<MeterUpdateNotification>)> {
            // Set sample rate from the actual input device config
            let actual_sample_rate = self.input_config.sample_rate.0 as f32;
            self.processor.set_sample_rate(actual_sample_rate);

            let mut processor = self.processor;
            let buffer_size = self.config.buffer_size as usize;
            let num_channels = self.input_config.channels as usize;
            let _param_bridge = Arc::clone(&self.param_bridge);

            // --- SPSC ring buffer for input→output audio transfer ---
            // Capacity: buffer_size * num_channels * 4 blocks of headroom.
            // Data format: interleaved f32 samples (matches cpal output).
            let ring_capacity = buffer_size * num_channels.max(2) * 4;
            let (mut ring_producer, mut ring_consumer) = rtrb::RingBuffer::new(ring_capacity);

            // --- SPSC ring buffer for meter data (audio → consumer task) ---
            // Capacity: 64 frames — sufficient for ~1s at 60 Hz update rate.
            // Uses rtrb (lock-free, zero-allocation) instead of tokio channels
            // to maintain real-time safety on the audio thread.
            let (mut meter_producer, meter_consumer) =
                rtrb::RingBuffer::<MeterUpdateNotification>::new(64);

            let mut frame_counter = 0u64;

            // Pre-allocate deinterleaved buffers BEFORE the audio callback.
            // These are moved into the closure and reused on every invocation,
            // avoiding heap allocations on the audio thread.
            let mut left_buf = vec![0.0f32; buffer_size];
            let mut right_buf = vec![0.0f32; buffer_size];

            // Pre-allocate interleave buffer for writing to the ring buffer.
            let mut interleave_buf = vec![0.0f32; buffer_size * 2];

            let input_stream = self
                .input_device
                .build_input_stream(
                    &self.input_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        frame_counter += 1;

                        let num_samples = data.len() / num_channels.max(1);
                        if num_samples == 0 || num_channels == 0 {
                            return;
                        }

                        let actual_samples = num_samples.min(left_buf.len());
                        let left = &mut left_buf[..actual_samples];
                        let right = &mut right_buf[..actual_samples];

                        // Zero-fill and deinterleave
                        left.fill(0.0);
                        right.fill(0.0);

                        for i in 0..actual_samples {
                            left[i] = data[i * num_channels];
                            if num_channels > 1 {
                                right[i] = data[i * num_channels + 1];
                            } else {
                                right[i] = left[i];
                            }
                        }

                        // Read parameter values at block boundary (RT-safe atomic reads).
                        // Currently the bridge is kept alive in the closure for future
                        // vtable v2 parameter injection. The infrastructure is in place.
                        let _ = &_param_bridge;

                        // Process through the user's DSP (stack-local channel array)
                        {
                            let mut channels: [&mut [f32]; 2] = [left, right];
                            processor.process(&mut channels);
                        }

                        // Re-borrow after process()
                        let left = &left_buf[..actual_samples];
                        let right = &right_buf[..actual_samples];

                        // Compute meters from processed output
                        let peak_left = left.iter().copied().fold(0.0f32, |a, b| a.max(b.abs()));
                        let rms_left =
                            (left.iter().map(|x| x * x).sum::<f32>() / left.len() as f32).sqrt();
                        let peak_right = right.iter().copied().fold(0.0f32, |a, b| a.max(b.abs()));
                        let rms_right =
                            (right.iter().map(|x| x * x).sum::<f32>() / right.len() as f32).sqrt();

                        // Send meter update approximately every other callback.
                        // At 44100 Hz / 512 samples per buffer ≈ 86 callbacks/sec,
                        // firing every 2nd callback gives ~43 Hz visual updates.
                        // The WebSocket/UI side already rate-limits display.
                        if frame_counter % 2 == 0 {
                            let notification = MeterUpdateNotification {
                                timestamp_us: frame_counter,
                                left_peak: peak_left,
                                left_rms: rms_left,
                                right_peak: peak_right,
                                right_rms: rms_right,
                            };
                            // Push to lock-free ring buffer — RT-safe, no allocation.
                            // If the consumer is slow, older frames are silently
                            // dropped (acceptable for metering data).
                            let _ = meter_producer.push(notification);
                        }

                        // Interleave processed audio and write to ring buffer.
                        // If the ring buffer is full, samples are silently dropped
                        // (acceptable — temporary glitch, RT-safe).
                        let interleave = &mut interleave_buf[..actual_samples * 2];
                        for i in 0..actual_samples {
                            interleave[i * 2] = left[i];
                            interleave[i * 2 + 1] = right[i];
                        }

                        // Write to SPSC ring buffer — non-blocking, lock-free.
                        // Push sample by sample; if full, remaining samples are dropped.
                        for &sample in interleave.iter() {
                            if ring_producer.push(sample).is_err() {
                                break;
                            }
                        }
                    },
                    |err| {
                        tracing::error!("Audio input stream error: {}", err);
                    },
                    None,
                )
                .context("Failed to build input stream")?;

            input_stream
                .play()
                .context("Failed to start input stream")?;
            tracing::info!("Input stream started");

            // --- Output stream (optional) ---
            let output_stream = if let (Some(output_device), Some(output_config)) =
                (self.output_device, self.output_config)
            {
                let stream = output_device
                    .build_output_stream(
                        &output_config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            // Read from SPSC ring buffer — non-blocking, lock-free.
                            // If underflow, fill with silence (zeros).
                            for sample in data.iter_mut() {
                                *sample = ring_consumer.pop().unwrap_or(0.0);
                            }
                        },
                        |err| {
                            tracing::error!("Audio output stream error: {}", err);
                        },
                        None,
                    )
                    .context("Failed to build output stream")?;

                stream.play().context("Failed to start output stream")?;
                tracing::info!("Output stream started");
                Some(stream)
            } else {
                tracing::info!("No output device — metering-only mode");
                None
            };

            let mode = if output_stream.is_some() {
                "full-duplex (input + output)"
            } else {
                "input-only (metering)"
            };
            tracing::info!("Audio server started in {} mode", mode);

            Ok((
                AudioHandle {
                    _input_stream: input_stream,
                    _output_stream: output_stream,
                },
                meter_consumer,
            ))
        }

        /// Returns true if an output device is available for audio playback.
        pub fn has_output(&self) -> bool {
            self.output_device.is_some()
        }
    }
}

#[cfg(feature = "audio")]
pub use implementation::*;
