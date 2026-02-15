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

use std::sync::Arc;

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use wavecraft_protocol::MeterUpdateNotification;

use super::atomic_params::AtomicParameterBridge;
use super::ffi_processor::DevAudioProcessor;

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
    output_device: Device,
    input_config: StreamConfig,
    output_config: StreamConfig,
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

        // Output device (required): dev mode expects audible output by default.
        let output_device = host
            .default_output_device()
            .context("No output device available")?;

        match output_device.name() {
            Ok(name) => tracing::info!("Using output device: {}", name),
            Err(_) => tracing::info!("Using output device: (unnamed)"),
        }

        let supported_output = output_device
            .default_output_config()
            .context("Failed to get default output config")?;
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
        let output_config: StreamConfig = supported_output.into();

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
        let input_channels = self.input_config.channels as usize;
        let output_channels = self.output_config.channels as usize;
        let param_bridge = Arc::clone(&self.param_bridge);

        // --- SPSC ring buffer for input→output audio transfer ---
        // Capacity: buffer_size * num_channels * 4 blocks of headroom.
        // Data format: interleaved f32 samples (matches cpal output).
        let ring_capacity = buffer_size * 2 * 4;
        let (mut ring_producer, mut ring_consumer) = rtrb::RingBuffer::new(ring_capacity);

        // --- SPSC ring buffer for meter data (audio → consumer task) ---
        // Capacity: 64 frames — sufficient for ~1s at 60 Hz update rate.
        // Uses rtrb (lock-free, zero-allocation) instead of tokio channels
        // to maintain real-time safety on the audio thread.
        let (mut meter_producer, meter_consumer) =
            rtrb::RingBuffer::<MeterUpdateNotification>::new(64);

        let mut frame_counter = 0u64;
        let mut oscillator_phase = 0.0_f32;

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

                    let num_samples = data.len() / input_channels.max(1);
                    if num_samples == 0 || input_channels == 0 {
                        return;
                    }

                    let actual_samples = num_samples.min(left_buf.len());
                    let left = &mut left_buf[..actual_samples];
                    let right = &mut right_buf[..actual_samples];

                    // Zero-fill and deinterleave
                    left.fill(0.0);
                    right.fill(0.0);

                    for i in 0..actual_samples {
                        left[i] = data[i * input_channels];
                        if input_channels > 1 {
                            right[i] = data[i * input_channels + 1];
                        } else {
                            right[i] = left[i];
                        }
                    }

                    // Process through the user's DSP (stack-local channel array)
                    {
                        let mut channels: [&mut [f32]; 2] = [left, right];
                        processor.process(&mut channels);
                    }

                    // Apply runtime output modifiers from lock-free parameters.
                    // This provides immediate control for source generators in
                    // browser dev mode while FFI parameter injection is evolving.
                    apply_output_modifiers(
                        left,
                        right,
                        &param_bridge,
                        &mut oscillator_phase,
                        actual_sample_rate,
                    );

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
                    if frame_counter.is_multiple_of(2) {
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

                    // Interleave processed stereo audio and write to ring buffer.
                    // If the ring buffer is full, samples are silently dropped
                    // (acceptable — temporary glitch, RT-safe).
                    let interleave = &mut interleave_buf[..actual_samples * 2];
                    for i in 0..actual_samples {
                        interleave[i * 2] = left[i];
                        interleave[i * 2 + 1] = right[i];
                    }

                    // Write to SPSC ring buffer — non-blocking, lock-free.
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

        // --- Output stream (required) ---
        let output_stream = self
            .output_device
            .build_output_stream(
                &self.output_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    if output_channels == 0 {
                        data.fill(0.0);
                        return;
                    }

                    // Route stereo frames from the ring into the device layout.
                    // Underflow is filled with silence.
                    for frame in data.chunks_mut(output_channels) {
                        let left = ring_consumer.pop().unwrap_or(0.0);
                        let right = ring_consumer.pop().unwrap_or(0.0);

                        if output_channels == 1 {
                            frame[0] = 0.5 * (left + right);
                            continue;
                        }

                        frame[0] = left;
                        frame[1] = right;

                        for channel in frame.iter_mut().skip(2) {
                            *channel = 0.0;
                        }
                    }
                },
                |err| {
                    tracing::error!("Audio output stream error: {}", err);
                },
                None,
            )
            .context("Failed to build output stream")?;

        output_stream
            .play()
            .context("Failed to start output stream")?;
        tracing::info!("Output stream started");

        let mode = "full-duplex (input + output)";
        tracing::info!("Audio server started in {} mode", mode);

        Ok((
            AudioHandle {
                _input_stream: input_stream,
                _output_stream: Some(output_stream),
            },
            meter_consumer,
        ))
    }

    /// Returns true if an output device is available for audio playback.
    pub fn has_output(&self) -> bool {
        true
    }
}

fn apply_output_modifiers(
    left: &mut [f32],
    right: &mut [f32],
    param_bridge: &AtomicParameterBridge,
    oscillator_phase: &mut f32,
    sample_rate: f32,
) {
    // Temporary dedicated control for sdk-template oscillator source.
    // 1.0 = on, 0.0 = off.
    if let Some(enabled) = param_bridge.read("oscillator_enabled")
        && enabled < 0.5
    {
        left.fill(0.0);
        right.fill(0.0);
        return;
    }

    // Focused dev-mode bridge for sdk-template oscillator parameters while
    // full generic FFI parameter injection is still being implemented.
    let Some(frequency) = param_bridge.read("oscillator_frequency") else {
        return;
    };
    let Some(level) = param_bridge.read("oscillator_level") else {
        return;
    };

    if !sample_rate.is_finite() || sample_rate <= 0.0 {
        return;
    }

    let clamped_frequency = if frequency.is_finite() {
        frequency.clamp(20.0, 5000.0)
    } else {
        440.0
    };
    let clamped_level = if level.is_finite() {
        level.clamp(0.0, 1.0)
    } else {
        0.0
    };

    let phase_delta = clamped_frequency / sample_rate;
    let mut phase = if oscillator_phase.is_finite() {
        *oscillator_phase
    } else {
        0.0
    };

    for (left_sample, right_sample) in left.iter_mut().zip(right.iter_mut()) {
        let sample = (phase * std::f32::consts::TAU).sin() * clamped_level;
        *left_sample = sample;
        *right_sample = sample;

        phase += phase_delta;
        if phase >= 1.0 {
            phase -= phase.floor();
        }
    }

    *oscillator_phase = phase;
}

#[cfg(test)]
mod tests {
    use super::apply_output_modifiers;
    use crate::audio::atomic_params::AtomicParameterBridge;
    use wavecraft_protocol::{ParameterInfo, ParameterType};

    fn bridge_with_enabled(default_value: f32) -> AtomicParameterBridge {
        AtomicParameterBridge::new(&[ParameterInfo {
            id: "oscillator_enabled".to_string(),
            name: "Enabled".to_string(),
            param_type: ParameterType::Float,
            value: default_value,
            default: default_value,
            unit: Some("%".to_string()),
            min: 0.0,
            max: 1.0,
            group: Some("Oscillator".to_string()),
        }])
    }

    #[test]
    fn output_modifiers_mute_when_oscillator_disabled() {
        let bridge = bridge_with_enabled(1.0);
        bridge.write("oscillator_enabled", 0.0);

        let mut left = [0.25_f32, -0.5, 0.75];
        let mut right = [0.2_f32, -0.4, 0.6];
        let mut phase = 0.0;
        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        assert!(left.iter().all(|s| s.abs() <= f32::EPSILON));
        assert!(right.iter().all(|s| s.abs() <= f32::EPSILON));
    }

    #[test]
    fn output_modifiers_keep_signal_when_oscillator_enabled() {
        let bridge = bridge_with_enabled(1.0);

        let mut left = [0.25_f32, -0.5, 0.75];
        let mut right = [0.2_f32, -0.4, 0.6];
        let mut phase = 0.0;
        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        assert_eq!(left, [0.25, -0.5, 0.75]);
        assert_eq!(right, [0.2, -0.4, 0.6]);
    }

    fn oscillator_bridge(frequency: f32, level: f32, enabled: f32) -> AtomicParameterBridge {
        AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "oscillator_enabled".to_string(),
                name: "Enabled".to_string(),
                param_type: ParameterType::Float,
                value: enabled,
                default: enabled,
                unit: Some("%".to_string()),
                min: 0.0,
                max: 1.0,
                group: Some("Oscillator".to_string()),
            },
            ParameterInfo {
                id: "oscillator_frequency".to_string(),
                name: "Frequency".to_string(),
                param_type: ParameterType::Float,
                value: frequency,
                default: frequency,
                min: 20.0,
                max: 5_000.0,
                unit: Some("Hz".to_string()),
                group: Some("Oscillator".to_string()),
            },
            ParameterInfo {
                id: "oscillator_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: level,
                default: level,
                unit: Some("%".to_string()),
                min: 0.0,
                max: 1.0,
                group: Some("Oscillator".to_string()),
            },
        ])
    }

    #[test]
    fn output_modifiers_generate_runtime_oscillator_from_frequency_and_level() {
        let bridge = oscillator_bridge(880.0, 0.75, 1.0);
        let mut left = [0.0_f32; 128];
        let mut right = [0.0_f32; 128];
        let mut phase = 0.0;

        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        let peak_left = left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
        let peak_right = right
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        assert!(peak_left > 0.2, "expected audible generated oscillator");
        assert!(peak_right > 0.2, "expected audible generated oscillator");
        assert_eq!(left, right, "expected in-phase stereo oscillator output");
        assert!(phase > 0.0, "phase should advance after generation");
    }

    #[test]
    fn output_modifiers_level_zero_produces_silence() {
        let bridge = oscillator_bridge(440.0, 0.0, 1.0);
        let mut left = [0.1_f32; 64];
        let mut right = [0.1_f32; 64];
        let mut phase = 0.0;

        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        assert!(left.iter().all(|s| s.abs() <= f32::EPSILON));
        assert!(right.iter().all(|s| s.abs() <= f32::EPSILON));
    }

    #[test]
    fn output_modifiers_frequency_change_changes_waveform() {
        let low_freq_bridge = oscillator_bridge(220.0, 0.5, 1.0);
        let high_freq_bridge = oscillator_bridge(1760.0, 0.5, 1.0);

        let mut low_left = [0.0_f32; 256];
        let mut low_right = [0.0_f32; 256];
        let mut high_left = [0.0_f32; 256];
        let mut high_right = [0.0_f32; 256];

        let mut low_phase = 0.0;
        let mut high_phase = 0.0;

        apply_output_modifiers(
            &mut low_left,
            &mut low_right,
            &low_freq_bridge,
            &mut low_phase,
            48_000.0,
        );
        apply_output_modifiers(
            &mut high_left,
            &mut high_right,
            &high_freq_bridge,
            &mut high_phase,
            48_000.0,
        );

        assert_ne!(
            low_left, high_left,
            "frequency updates should alter waveform"
        );
        assert_eq!(low_left, low_right);
        assert_eq!(high_left, high_right);
    }
}
