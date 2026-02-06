//! Synthetic meter generator for development UI testing
//!
//! Generates animated meter data that looks realistic without
//! requiring actual audio processing. Useful for testing meter
//! components and visualizations.

use wavecraft_protocol::MeterFrame;

/// Generator for synthetic metering data
///
/// Creates smooth, animated meter values that look natural
/// for UI development and testing purposes.
pub struct MeterGenerator {
    /// Current animation phase (0.0 - 2π)
    phase: f64,
    /// Animation speed (radians per tick)
    #[allow(dead_code)] // Reserved for animated meter feature
    speed: f64,
    /// Base level (dB)
    base_level: f64,
    /// Modulation depth
    modulation: f64,
}

impl MeterGenerator {
    /// Create a new meter generator with default settings
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            speed: 0.05, // ~60 updates to complete one cycle
            base_level: -12.0,
            modulation: 6.0,
        }
    }

    /// Advance the animation by one tick
    ///
    /// Call this at your desired update rate (e.g., 60 Hz)
    #[allow(dead_code)] // Reserved for animated meter feature
    pub fn tick(&mut self) {
        self.phase += self.speed;
        if self.phase > std::f64::consts::TAU {
            self.phase -= std::f64::consts::TAU;
        }
    }

    /// Get the current meter frame
    ///
    /// Returns synthetic peak/RMS values that animate smoothly
    pub fn frame(&self) -> MeterFrame {
        // Generate smooth oscillating levels
        let sine = self.phase.sin();
        let cosine = (self.phase * 1.3).cos(); // Slightly different rate for L/R

        // Convert dB to linear
        let left_db = self.base_level + sine * self.modulation;
        let right_db = self.base_level + cosine * self.modulation;

        let left_linear = db_to_linear(left_db);
        let right_linear = db_to_linear(right_db);

        // RMS is slightly lower than peak
        let rms_factor = 0.7;

        MeterFrame {
            peak_l: left_linear as f32,
            rms_l: (left_linear * rms_factor) as f32,
            peak_r: right_linear as f32,
            rms_r: (right_linear * rms_factor) as f32,
            timestamp: 0, // Synthetic meters don't need timestamps
        }
    }
}

impl Default for MeterGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert decibels to linear amplitude
fn db_to_linear(db: f64) -> f64 {
    10.0_f64.powf(db / 20.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meter_generator_creates_valid_frames() {
        let mut gen = MeterGenerator::new();

        for _ in 0..100 {
            let frame = gen.frame();

            // Values should be valid (non-negative, reasonable range)
            assert!(frame.peak_l >= 0.0);
            assert!(frame.peak_r >= 0.0);
            assert!(frame.rms_l >= 0.0);
            assert!(frame.rms_r >= 0.0);

            // RMS should be <= peak
            assert!(frame.rms_l <= frame.peak_l + f32::EPSILON);
            assert!(frame.rms_r <= frame.peak_r + f32::EPSILON);

            gen.tick();
        }
    }

    #[test]
    fn test_meter_generator_animates() {
        let mut gen = MeterGenerator::new();

        let frame1 = gen.frame();
        gen.tick();
        let frame2 = gen.frame();

        // Values should change between ticks
        assert!(
            (frame1.peak_l - frame2.peak_l).abs() > f32::EPSILON
                || (frame1.peak_r - frame2.peak_r).abs() > f32::EPSILON
        );
    }

    #[test]
    fn test_db_to_linear() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 0.0001);
        assert!((db_to_linear(-6.0) - 0.5012).abs() < 0.001);
        assert!((db_to_linear(-20.0) - 0.1).abs() < 0.001);
    }
}
