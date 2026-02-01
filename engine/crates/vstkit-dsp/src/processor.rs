//! Audio processor - reference implementation with gain control.
//!
//! The `GainProcessor` struct is a reference implementation of a simple gain
//! processor. It demonstrates real-time safe audio processing patterns.

use crate::gain::db_to_linear;

/// Reference gain processor implementation for VstKit.
///
/// This struct maintains processing state and provides a simple gain effect.
/// All methods are designed to be real-time safe (no allocations, no locks, no syscalls).
///
/// This serves as a reference implementation for the `Processor` trait.
pub struct GainProcessor {
    sample_rate: f32,
}

impl GainProcessor {
    /// Create a new gain processor with the given sample rate.
    ///
    /// # Arguments
    /// * `sample_rate` - The audio sample rate in Hz (e.g., 44100.0)
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }

    /// Update the sample rate.
    ///
    /// Call this when the host changes sample rate (e.g., in `initialize()`).
    ///
    /// # Arguments
    /// * `sample_rate` - The new sample rate in Hz
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    /// Get the current sample rate.
    #[inline]
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Process stereo audio buffers in-place.
    ///
    /// # Arguments
    /// * `left` - Left channel audio buffer (modified in-place)
    /// * `right` - Right channel audio buffer (modified in-place)
    /// * `gain_db` - Gain to apply in decibels
    ///
    /// # Real-Time Safety
    /// This method is real-time safe:
    /// - No allocations
    /// - No locks
    /// - No syscalls
    /// - No panics (uses debug_assert only)
    #[inline]
    pub fn process(&self, left: &mut [f32], right: &mut [f32], gain_db: f32) {
        debug_assert_eq!(
            left.len(),
            right.len(),
            "Left and right buffers must have equal length"
        );

        let gain_linear = db_to_linear(gain_db);

        // Apply gain to left channel
        for sample in left.iter_mut() {
            *sample *= gain_linear;
        }

        // Apply gain to right channel
        for sample in right.iter_mut() {
            *sample *= gain_linear;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough_at_0db() {
        let processor = GainProcessor::new(44100.0);
        let mut left = [0.5, -0.5, 0.25, -0.25];
        let mut right = [0.3, -0.3, 0.1, -0.1];

        let left_original = left;
        let right_original = right;

        processor.process(&mut left, &mut right, 0.0);

        for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
            assert!(
                (l - left_original[i]).abs() < 1e-6,
                "Left sample {} changed at 0dB gain",
                i
            );
            assert!(
                (r - right_original[i]).abs() < 1e-6,
                "Right sample {} changed at 0dB gain",
                i
            );
        }
    }

    #[test]
    fn test_gain_applied() {
        let processor = GainProcessor::new(44100.0);
        let mut left = [1.0, 1.0, 1.0, 1.0];
        let mut right = [1.0, 1.0, 1.0, 1.0];

        // -6 dB ≈ 0.501187 linear gain
        processor.process(&mut left, &mut right, -6.0);

        let expected_gain = db_to_linear(-6.0);
        for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
            assert!(
                (l - expected_gain).abs() < 0.001,
                "Left sample {} has incorrect gain",
                i
            );
            assert!(
                (r - expected_gain).abs() < 0.001,
                "Right sample {} has incorrect gain",
                i
            );
        }
    }

    #[test]
    fn test_negative_gain() {
        let processor = GainProcessor::new(44100.0);
        let mut left = [1.0];
        let mut right = [1.0];

        processor.process(&mut left, &mut right, -12.0);

        let expected = db_to_linear(-12.0);
        assert!(
            (left[0] - expected).abs() < 0.001,
            "Attenuation not applied correctly"
        );
    }

    #[test]
    fn test_positive_gain() {
        let processor = GainProcessor::new(44100.0);
        let mut left = [0.5];
        let mut right = [0.5];

        processor.process(&mut left, &mut right, 6.0);

        let expected = 0.5 * db_to_linear(6.0);
        assert!(
            (left[0] - expected).abs() < 0.001,
            "Boost not applied correctly"
        );
    }

    #[test]
    fn test_sample_rate_update() {
        let mut processor = GainProcessor::new(44100.0);
        assert!((processor.sample_rate() - 44100.0).abs() < 1e-6);

        processor.set_sample_rate(48000.0);
        assert!((processor.sample_rate() - 48000.0).abs() < 1e-6);
    }
}
