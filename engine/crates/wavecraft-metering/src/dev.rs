//! Development utilities for synthetic meter data.

use wavecraft_protocol::{MeterFrame, db_to_linear};

/// Generator for synthetic metering data.
///
/// Creates smooth, animated meter values for UI development and testing.
pub struct MeterGenerator {
    phase: f32,
    speed: f32,
    base_level_db: f32,
    modulation_db: f32,
}

impl MeterGenerator {
    /// Create a new meter generator with default settings.
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            speed: 0.05,
            base_level_db: -12.0,
            modulation_db: 6.0,
        }
    }

    /// Advance the animation by one tick.
    pub fn tick(&mut self) {
        self.phase += self.speed;
        if self.phase > std::f32::consts::TAU {
            self.phase -= std::f32::consts::TAU;
        }
    }

    /// Get the current meter frame.
    pub fn frame(&self) -> MeterFrame {
        let sine = self.phase.sin();
        let cosine = (self.phase * 1.3).cos();

        let left_db = self.base_level_db + sine * self.modulation_db;
        let right_db = self.base_level_db + cosine * self.modulation_db;

        let left_linear = db_to_linear(left_db);
        let right_linear = db_to_linear(right_db);

        let rms_factor = 0.7;

        MeterFrame {
            peak_l: left_linear,
            rms_l: left_linear * rms_factor,
            peak_r: right_linear,
            rms_r: right_linear * rms_factor,
            timestamp: 0,
        }
    }
}

impl Default for MeterGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meter_generator_creates_valid_frames() {
        let mut meter_gen = MeterGenerator::new();

        for _ in 0..100 {
            let frame = meter_gen.frame();

            assert!(frame.peak_l >= 0.0);
            assert!(frame.peak_r >= 0.0);
            assert!(frame.rms_l >= 0.0);
            assert!(frame.rms_r >= 0.0);

            assert!(frame.rms_l <= frame.peak_l + f32::EPSILON);
            assert!(frame.rms_r <= frame.peak_r + f32::EPSILON);

            meter_gen.tick();
        }
    }

    #[test]
    fn test_meter_generator_animates() {
        let mut meter_gen = MeterGenerator::new();

        let frame1 = meter_gen.frame();
        meter_gen.tick();
        let frame2 = meter_gen.frame();

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
