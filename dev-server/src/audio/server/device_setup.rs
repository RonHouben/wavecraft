use std::sync::Arc;

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Stream, StreamConfig};
use wavecraft_processors::OscilloscopeTap;
use wavecraft_protocol::MeterUpdateNotification;

use super::super::atomic_params::AtomicParameterBridge;
use super::super::ffi_processor::DevAudioProcessor;

pub(super) struct NegotiatedAudioDeviceConfig {
    pub(super) input_device: Device,
    pub(super) output_device: Device,
    pub(super) input_config: StreamConfig,
    pub(super) output_config: StreamConfig,
}

pub(super) struct InputStreamBuildContext {
    pub(super) processor: Box<dyn DevAudioProcessor>,
    pub(super) buffer_size: usize,
    pub(super) input_channels: usize,
    pub(super) param_bridge: Arc<AtomicParameterBridge>,
    pub(super) actual_sample_rate: f32,
    pub(super) ring_producer: rtrb::Producer<f32>,
    pub(super) meter_producer: rtrb::Producer<MeterUpdateNotification>,
    pub(super) oscilloscope_tap: OscilloscopeTap,
}

pub(super) fn negotiate_default_devices_and_configs() -> Result<NegotiatedAudioDeviceConfig> {
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

    Ok(NegotiatedAudioDeviceConfig {
        input_device,
        output_device,
        input_config,
        output_config,
    })
}

pub(super) fn build_input_stream(
    input_device: &Device,
    input_config: &StreamConfig,
    context: InputStreamBuildContext,
) -> Result<Stream> {
    let mut input_pipeline = super::input_pipeline::InputCallbackPipeline::new(context);

    let input_stream = input_device
        .build_input_stream(
            input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                input_pipeline.process_callback(data);
            },
            |err| {
                tracing::error!("Audio input stream error: {}", err);
            },
            None,
        )
        .context("Failed to build input stream")?;

    Ok(input_stream)
}

pub(super) fn build_output_stream(
    output_device: &Device,
    output_config: &StreamConfig,
    output_channels: usize,
    mut ring_consumer: rtrb::Consumer<f32>,
) -> Result<Stream> {
    let output_stream = output_device
        .build_output_stream(
            output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                super::output_routing::route_output_callback(
                    data,
                    output_channels,
                    &mut ring_consumer,
                );
            },
            |err| {
                tracing::error!("Audio output stream error: {}", err);
            },
            None,
        )
        .context("Failed to build output stream")?;

    Ok(output_stream)
}
