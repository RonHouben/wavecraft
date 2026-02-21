use std::sync::Arc;

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Stream, StreamConfig};
use wavecraft_processors::OscilloscopeTap;
use wavecraft_protocol::MeterUpdateNotification;

use super::super::atomic_params::AtomicParameterBridge;
use super::super::ffi_processor::DevAudioProcessor;

const INPUT_STREAM_LABEL: &str = "input";
const OUTPUT_STREAM_LABEL: &str = "output";

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
    log_required_device_name(INPUT_STREAM_LABEL, &input_device)?;

    let input_supported_config = input_device
        .default_input_config()
        .context("Failed to get default input config")?;
    let input_sample_rate = input_supported_config.sample_rate().0;
    log_sample_rate(INPUT_STREAM_LABEL, input_sample_rate);
    let input_config: StreamConfig = input_supported_config.into();

    // Output device (required): dev mode expects audible output by default.
    let output_device = host
        .default_output_device()
        .context("No output device available")?;

    log_device_name_or_fallback(OUTPUT_STREAM_LABEL, &output_device);

    let output_supported_config = output_device
        .default_output_config()
        .context("Failed to get default output config")?;
    let output_sample_rate = output_supported_config.sample_rate().0;
    log_sample_rate(OUTPUT_STREAM_LABEL, output_sample_rate);
    if output_sample_rate != input_sample_rate {
        tracing::warn!(
            "Input/output sample rate mismatch ({} vs {}). \
             Processing at input rate; output device may resample.",
            input_sample_rate,
            output_sample_rate
        );
    }
    let output_config: StreamConfig = output_supported_config.into();

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
            |err| log_stream_error(INPUT_STREAM_LABEL, err),
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
            |err| log_stream_error(OUTPUT_STREAM_LABEL, err),
            None,
        )
        .context("Failed to build output stream")?;

    Ok(output_stream)
}

fn log_required_device_name(stream_label: &str, device: &Device) -> Result<()> {
    tracing::info!("Using {} device: {}", stream_label, device.name()?);
    Ok(())
}

fn log_device_name_or_fallback(stream_label: &str, device: &Device) {
    match device.name() {
        Ok(name) => tracing::info!("Using {} device: {}", stream_label, name),
        Err(_) => tracing::info!("Using {} device: (unnamed)", stream_label),
    }
}

fn log_sample_rate(stream_label: &str, sample_rate_hz: u32) {
    tracing::info!(
        "{} sample rate: {} Hz",
        stream_label_display_name(stream_label),
        sample_rate_hz
    );
}

fn log_stream_error(stream_label: &str, err: cpal::StreamError) {
    tracing::error!("Audio {} stream error: {}", stream_label, err);
}

fn stream_label_display_name(stream_label: &str) -> &'static str {
    match stream_label {
        INPUT_STREAM_LABEL => "Input",
        OUTPUT_STREAM_LABEL => "Output",
        _ => "Stream",
    }
}
