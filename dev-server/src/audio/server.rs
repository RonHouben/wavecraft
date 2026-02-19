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

mod device_setup;
mod input_pipeline;
mod metering;
mod output_routing;
mod output_modifiers;

use std::sync::Arc;

use anyhow::{Context, Result};
use cpal::traits::StreamTrait;
use cpal::{Device, Stream, StreamConfig};
use wavecraft_processors::{
    OscilloscopeFrameConsumer, OscilloscopeTap, create_oscilloscope_channel,
};
use wavecraft_protocol::MeterUpdateNotification;

use super::atomic_params::AtomicParameterBridge;
use super::ffi_processor::DevAudioProcessor;

const GAIN_MULTIPLIER_MIN: f32 = 0.0;
const GAIN_MULTIPLIER_MAX: f32 = 2.0;

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
        let negotiated = device_setup::negotiate_default_devices_and_configs()?;

        Ok(Self {
            processor,
            config,
            input_device: negotiated.input_device,
            output_device: negotiated.output_device,
            input_config: negotiated.input_config,
            output_config: negotiated.output_config,
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
    pub fn start(
        mut self,
    ) -> Result<(
        AudioHandle,
        rtrb::Consumer<MeterUpdateNotification>,
        OscilloscopeFrameConsumer,
    )> {
        // Set sample rate from the actual input device config
        let actual_sample_rate = self.input_config.sample_rate.0 as f32;
        self.processor.set_sample_rate(actual_sample_rate);

        let processor = self.processor;
        let buffer_size = self.config.buffer_size as usize;
        let input_channels = self.input_config.channels as usize;
        let output_channels = self.output_config.channels as usize;
        let param_bridge = Arc::clone(&self.param_bridge);

        // --- SPSC ring buffer for input→output audio transfer ---
        // Capacity: buffer_size * num_channels * 4 blocks of headroom.
        // Data format: interleaved f32 samples (matches cpal output).
        let ring_capacity = buffer_size * 2 * 4;
        let (ring_producer, ring_consumer) = rtrb::RingBuffer::new(ring_capacity);

        // --- SPSC ring buffer for meter data (audio → consumer task) ---
        // Capacity: 64 frames — sufficient for ~1s at 60 Hz update rate.
        // Uses rtrb (lock-free, zero-allocation) instead of tokio channels
        // to maintain real-time safety on the audio thread.
        let (meter_producer, meter_consumer) = rtrb::RingBuffer::<MeterUpdateNotification>::new(64);
        let (oscilloscope_producer, oscilloscope_consumer) = create_oscilloscope_channel(8);
        let mut oscilloscope_tap = OscilloscopeTap::with_output(oscilloscope_producer);
        oscilloscope_tap.set_sample_rate_hz(actual_sample_rate);
        let input_stream = device_setup::build_input_stream(
            &self.input_device,
            &self.input_config,
            device_setup::InputStreamBuildContext {
                processor,
                buffer_size,
                input_channels,
                param_bridge,
                actual_sample_rate,
                ring_producer,
                meter_producer,
                oscilloscope_tap,
            },
        )?;

        input_stream
            .play()
            .context("Failed to start input stream")?;
        tracing::info!("Input stream started");

        // --- Output stream (required) ---
        let output_stream = device_setup::build_output_stream(
            &self.output_device,
            &self.output_config,
            output_channels,
            ring_consumer,
        )?;

        output_stream
            .play()
            .context("Failed to start output stream")?;
        tracing::info!("Output stream started");

        tracing::info!("Audio server started in full-duplex (input + output) mode");

        Ok((
            AudioHandle {
                _input_stream: input_stream,
                _output_stream: Some(output_stream),
            },
            meter_consumer,
            oscilloscope_consumer,
        ))
    }

    /// Returns true if an output device is available for audio playback.
    pub fn has_output(&self) -> bool {
        true
    }
}
