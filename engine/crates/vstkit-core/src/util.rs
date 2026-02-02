//! Utility functions for common plugin operations.

use nih_plug::prelude::Buffer;

/// Calculate peak and RMS values for stereo buffer (real-time safe).
///
/// Returns (peak_l, peak_r, rms_l, rms_r) in linear scale.
///
/// # Example
///
/// ```rust,no_run
/// use vstkit_core::util::calculate_stereo_meters;
/// use vstkit_metering::MeterFrame;
/// # use nih_plug::prelude::*;
///
/// # fn process(buffer: &Buffer, meter_producer: &mut vstkit_metering::MeterProducer) {
/// let (peak_l, peak_r, rms_l, rms_r) = calculate_stereo_meters(buffer);
/// meter_producer.push(MeterFrame {
///     peak_l, peak_r, rms_l, rms_r,
///     timestamp: 0,
/// });
/// # }
/// ```
#[inline]
pub fn calculate_stereo_meters(buffer: &Buffer) -> (f32, f32, f32, f32) {
    let mut peak_l = 0.0f32;
    let mut peak_r = 0.0f32;
    let mut sum_sq_l = 0.0f32;
    let mut sum_sq_r = 0.0f32;

    let num_samples = buffer.samples() as f32;

    // Iterate over channels using nih-plug's safe API
    let channels = buffer.as_slice_immutable();
    if channels.len() >= 2 {
        let left = &channels[0];
        let right = &channels[1];

        for &sample in left.iter() {
            peak_l = peak_l.max(sample.abs());
            sum_sq_l += sample * sample;
        }

        for &sample in right.iter() {
            peak_r = peak_r.max(sample.abs());
            sum_sq_r += sample * sample;
        }
    }

    let rms_l = (sum_sq_l / num_samples).sqrt();
    let rms_r = (sum_sq_r / num_samples).sqrt();

    (peak_l, peak_r, rms_l, rms_r)
}
