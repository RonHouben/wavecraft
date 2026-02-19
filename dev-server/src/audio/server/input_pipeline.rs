use std::sync::Arc;

use wavecraft_processors::OscilloscopeTap;
use wavecraft_protocol::MeterUpdateNotification;

use super::super::atomic_params::AtomicParameterBridge;
use super::super::ffi_processor::DevAudioProcessor;
use super::device_setup::InputStreamBuildContext;

pub(super) struct InputCallbackPipeline {
    frame_counter: u64,
    oscillator_phase: f32,
    left_buf: Vec<f32>,
    right_buf: Vec<f32>,
    interleave_buf: Vec<f32>,
    processor: Box<dyn DevAudioProcessor>,
    input_channels: usize,
    param_bridge: Arc<AtomicParameterBridge>,
    actual_sample_rate: f32,
    ring_producer: rtrb::Producer<f32>,
    meter_producer: rtrb::Producer<MeterUpdateNotification>,
    oscilloscope_tap: OscilloscopeTap,
}

impl InputCallbackPipeline {
    pub(super) fn new(context: InputStreamBuildContext) -> Self {
        Self {
            frame_counter: 0,
            oscillator_phase: 0.0,
            left_buf: vec![0.0f32; context.buffer_size],
            right_buf: vec![0.0f32; context.buffer_size],
            interleave_buf: vec![0.0f32; context.buffer_size * 2],
            processor: context.processor,
            input_channels: context.input_channels,
            param_bridge: context.param_bridge,
            actual_sample_rate: context.actual_sample_rate,
            ring_producer: context.ring_producer,
            meter_producer: context.meter_producer,
            oscilloscope_tap: context.oscilloscope_tap,
        }
    }

    pub(super) fn process_callback(&mut self, data: &[f32]) {
        self.frame_counter += 1;

        let num_samples = data.len() / self.input_channels.max(1);
        if num_samples == 0 || self.input_channels == 0 {
            return;
        }

        let actual_samples = num_samples.min(self.left_buf.len());
        let left = &mut self.left_buf[..actual_samples];
        let right = &mut self.right_buf[..actual_samples];

        // Zero-fill and deinterleave
        left.fill(0.0);
        right.fill(0.0);

        for i in 0..actual_samples {
            left[i] = data[i * self.input_channels];
            if self.input_channels > 1 {
                right[i] = data[i * self.input_channels + 1];
            } else {
                right[i] = left[i];
            }
        }

        // Process through the user's DSP (stack-local channel array)
        {
            let mut channels: [&mut [f32]; 2] = [left, right];
            self.processor.process(&mut channels);
        }

        // Apply runtime output modifiers from lock-free parameters.
        // This provides immediate control for source generators in
        // browser dev mode while FFI parameter injection is evolving.
        super::output_modifiers::apply_output_modifiers(
            left,
            right,
            &self.param_bridge,
            &mut self.oscillator_phase,
            self.actual_sample_rate,
        );

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
        for i in 0..actual_samples {
            interleave[i * 2] = left[i];
            interleave[i * 2 + 1] = right[i];
        }

        // Write to SPSC ring buffer — non-blocking, lock-free.
        for &sample in interleave.iter() {
            if self.ring_producer.push(sample).is_err() {
                break;
            }
        }
    }
}
