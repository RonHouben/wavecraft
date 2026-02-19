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
mod output_modifiers;
mod output_routing;
mod startup_wiring;

use std::sync::Arc;

use anyhow::Result;
use cpal::{Device, Stream, StreamConfig};
use wavecraft_processors::OscilloscopeFrameConsumer;
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

        startup_wiring::start_audio_io(startup_wiring::StartAudioIoContext {
            input_device: &self.input_device,
            input_config: &self.input_config,
            output_device: &self.output_device,
            output_config: &self.output_config,
            processor,
            buffer_size,
            input_channels,
            output_channels,
            param_bridge,
            actual_sample_rate,
        })
    }

    /// Returns true if an output device is available for audio playback.
    pub fn has_output(&self) -> bool {
        true
    }
}
