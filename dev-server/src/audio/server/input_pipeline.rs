use std::sync::Arc;

use wavecraft_processors::OscilloscopeTap;
use wavecraft_protocol::MeterUpdateNotification;

use super::super::atomic_params::AtomicParameterBridge;
use super::super::ffi_processor::DevAudioProcessor;

const DEV_FFI_V1_COMPAT_ENV: &str = "WAVECRAFT_DEV_FFI_V1_COMPAT";
use super::device_setup::InputStreamBuildContext;

pub(super) struct InputCallbackPipeline {
    frame_counter: u64,
    left_buf: Vec<f32>,
    right_buf: Vec<f32>,
    interleave_buf: Vec<f32>,
    plain_values_buf: Vec<f32>,
    processor: Box<dyn DevAudioProcessor>,
    input_channels: usize,
    param_bridge: Arc<AtomicParameterBridge>,
    actual_sample_rate: f32,
    ring_producer: rtrb::Producer<f32>,
    meter_producer: rtrb::Producer<MeterUpdateNotification>,
    oscilloscope_tap: OscilloscopeTap,
    v1_compat_oscillator_phase: Option<f32>,
}

impl InputCallbackPipeline {
    pub(super) fn new(context: InputStreamBuildContext) -> Self {
        Self {
            frame_counter: 0,
            left_buf: vec![0.0f32; context.buffer_size],
            right_buf: vec![0.0f32; context.buffer_size],
            interleave_buf: vec![0.0f32; context.buffer_size * 2],
            plain_values_buf: vec![0.0f32; context.param_bridge.parameter_count()],
            processor: context.processor,
            input_channels: context.input_channels,
            param_bridge: context.param_bridge,
            actual_sample_rate: context.actual_sample_rate,
            ring_producer: context.ring_producer,
            meter_producer: context.meter_producer,
            oscilloscope_tap: context.oscilloscope_tap,
            v1_compat_oscillator_phase: is_v1_compat_mode_enabled().then_some(0.0),
        }
    }

    pub(super) fn process_callback(&mut self, data: &[f32]) {
        self.frame_counter += 1;

        let Some(actual_samples) =
            callback_sample_count(data.len(), self.input_channels, self.left_buf.len())
        else {
            return;
        };

        let left = &mut self.left_buf[..actual_samples];
        let right = &mut self.right_buf[..actual_samples];

        // Zero-fill and deinterleave
        deinterleave_input(data, self.input_channels, left, right);

        // Process through the user's DSP (stack-local channel array)
        let plain_values_len = self.param_bridge.copy_all_to(&mut self.plain_values_buf);
        // Compatibility mode is authoritative when explicitly enabled:
        // skip v2 plain-value injection and rely on the compat output path only.
        if self.v1_compat_oscillator_phase.is_none() {
            self.processor
                .apply_plain_values(&self.plain_values_buf[..plain_values_len]);
        }

        {
            let mut channels: [&mut [f32]; 2] = [left, right];
            self.processor.process(&mut channels);
        }

        // Temporary explicit compatibility path for v1-style dev DSP behavior.
        // Default path (compat disabled) uses FFI parameter injection only.
        if let Some(phase) = self.v1_compat_oscillator_phase.as_mut() {
            super::output_modifiers::apply_v1_compat_output_modifiers(
                left,
                right,
                &self.param_bridge,
                phase,
                self.actual_sample_rate,
            );
        }

        // Re-borrow after process()
        let left = &self.left_buf[..actual_samples];
        let right = &self.right_buf[..actual_samples];

        // Observation-only waveform capture for oscilloscope UI.
        self.oscilloscope_tap.capture_stereo(left, right);

        if let Some(notification) =
            super::metering::maybe_build_meter_update(self.frame_counter, left, right)
        {
            // Push to lock-free ring buffer — RT-safe, no allocation.
            // If the consumer is slow, older frames are silently
            // dropped (acceptable for metering data).
            let _ = self.meter_producer.push(notification);
        }

        // Interleave processed stereo audio and write to ring buffer.
        // If the ring buffer is full, samples are silently dropped
        // (acceptable — temporary glitch, RT-safe).
        let interleave = &mut self.interleave_buf[..actual_samples * 2];
        interleave_stereo(left, right, interleave);

        // Write to SPSC ring buffer — non-blocking, lock-free.
        push_samples_to_ring(&mut self.ring_producer, interleave);
    }
}

fn callback_sample_count(
    data_len: usize,
    input_channels: usize,
    max_samples: usize,
) -> Option<usize> {
    let num_samples = data_len / input_channels.max(1);
    if num_samples == 0 || input_channels == 0 {
        return None;
    }

    Some(num_samples.min(max_samples))
}

fn deinterleave_input(data: &[f32], input_channels: usize, left: &mut [f32], right: &mut [f32]) {
    left.fill(0.0);
    right.fill(0.0);

    for i in 0..left.len() {
        left[i] = data[i * input_channels];
        if input_channels > 1 {
            right[i] = data[i * input_channels + 1];
        } else {
            right[i] = left[i];
        }
    }
}

fn interleave_stereo(left: &[f32], right: &[f32], interleave: &mut [f32]) {
    for i in 0..left.len() {
        interleave[i * 2] = left[i];
        interleave[i * 2 + 1] = right[i];
    }
}

fn push_samples_to_ring(ring_producer: &mut rtrb::Producer<f32>, samples: &[f32]) {
    for &sample in samples {
        if ring_producer.push(sample).is_err() {
            break;
        }
    }
}

fn is_v1_compat_mode_enabled() -> bool {
    std::env::var(DEV_FFI_V1_COMPAT_ENV)
        .map(|value| value == "1")
        .unwrap_or(false)
}
