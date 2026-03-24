use std::sync::Arc;

use anyhow::{Context, Result};
use cpal::traits::StreamTrait;
use cpal::{Device, StreamConfig};
use wavecraft_protocol::MeterUpdateNotification;

use super::super::SharedHardwareInputRoutingSelection;
use super::super::SharedInputSourceSelection;
use super::super::atomic_params::AtomicParameterBridge;
use super::super::ffi_processor::DevAudioProcessor;
use super::{AudioHandle, device_setup};

pub(super) struct StartAudioIoContext<'a> {
    pub(super) input_device: &'a Device,
    pub(super) input_config: &'a StreamConfig,
    pub(super) output_device: &'a Device,
    pub(super) output_config: &'a StreamConfig,
    pub(super) processor: Box<dyn DevAudioProcessor>,
    pub(super) buffer_size: usize,
    pub(super) input_channels: usize,
    pub(super) output_channels: usize,
    pub(super) param_bridge: Arc<AtomicParameterBridge>,
    pub(super) input_source_selection: SharedInputSourceSelection,
    pub(super) hardware_input_routing_selection: SharedHardwareInputRoutingSelection,
}

pub(super) fn start_audio_io(
    context: StartAudioIoContext<'_>,
) -> Result<(AudioHandle, rtrb::Consumer<MeterUpdateNotification>)> {
    // --- SPSC ring buffer for input→output audio transfer ---
    // Capacity: buffer_size * num_channels * 4 blocks of headroom.
    // Data format: interleaved f32 samples (matches cpal output).
    let ring_capacity = context.buffer_size * 2 * 4;
    let (ring_producer, ring_consumer) = rtrb::RingBuffer::new(ring_capacity);

    // --- SPSC ring buffer for meter data (audio → consumer task) ---
    // Capacity: 64 frames — sufficient for ~1s at 60 Hz update rate.
    // Uses rtrb (lock-free, zero-allocation) instead of tokio channels
    // to maintain real-time safety on the audio thread.
    let (meter_producer, meter_consumer) = rtrb::RingBuffer::<MeterUpdateNotification>::new(64);

    let input_stream = device_setup::build_input_stream(
        context.input_device,
        context.input_config,
        device_setup::InputStreamBuildContext {
            processor: context.processor,
            buffer_size: context.buffer_size,
            input_channels: context.input_channels,
            param_bridge: context.param_bridge,
            input_source_selection: context.input_source_selection,
            hardware_input_routing_selection: context.hardware_input_routing_selection,
            ring_producer,
            meter_producer,
        },
    )?;

    input_stream
        .play()
        .context("Failed to start input stream")?;
    tracing::info!("Input stream started");

    // --- Output stream (required) ---
    let output_stream = device_setup::build_output_stream(
        context.output_device,
        context.output_config,
        context.output_channels,
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
    ))
}
