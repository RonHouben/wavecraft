//! Utility functions for Wavecraft plugins.
//!
//! This module provides helper functions used by the plugin framework.

use nih_plug::prelude::*;
use wavecraft_metering::MeterFrame;

/// Calculate stereo peak and RMS meters from a nih-plug buffer.
///
/// This function computes the peak and RMS values for left and right channels
/// from the given audio buffer.
///
/// # Arguments
///
/// * `buffer` - The nih-plug audio buffer to analyze
///
/// # Returns
///
/// A `MeterFrame` containing the peak and RMS values in linear scale.
#[inline]
pub fn calculate_stereo_meters(buffer: &Buffer) -> MeterFrame {
    let num_samples = buffer.samples();
    if num_samples == 0 {
        return MeterFrame::default();
    }

    let mut peak_l = 0.0f32;
    let mut peak_r = 0.0f32;
    let mut sum_sq_l = 0.0f32;
    let mut sum_sq_r = 0.0f32;

    let num_samples_f32 = num_samples as f32;

    // Iterate over channels using nih-plug's safe immutable API
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

    let rms_l = (sum_sq_l / num_samples_f32).sqrt();
    let rms_r = (sum_sq_r / num_samples_f32).sqrt();

    MeterFrame {
        peak_l,
        peak_r,
        rms_l,
        rms_r,
        timestamp: 0,
    }
}
