//! Soft-clip saturator processor.

use wavecraft_dsp::{ParamRange, ParamSpec, Processor, ProcessorParams, Transport};
use wavecraft_protocol::db_to_linear;

const MIN_GAIN_DB: f32 = -24.0;
const MAX_GAIN_DB: f32 = 24.0;

/// Parameters for the soft-clip saturator.
#[derive(Debug, Clone)]
pub struct SaturatorParams {
    /// Input drive in dB before saturation.
    pub drive_db: f32,
    /// Output trim in dB after saturation.
    pub output_trim_db: f32,
}

impl Default for SaturatorParams {
    fn default() -> Self {
        Self::from_param_defaults()
    }
}

impl ProcessorParams for SaturatorParams {
    fn param_specs() -> &'static [ParamSpec] {
        static SPECS: [ParamSpec; 2] = [
            ParamSpec {
                name: "Drive",
                id_suffix: "drive_db",
                range: ParamRange::Linear {
                    min: MIN_GAIN_DB as f64,
                    max: MAX_GAIN_DB as f64,
                },
                default: 0.0,
                unit: "dB",
                group: Some("Saturator"),
            },
            ParamSpec {
                name: "Output Trim",
                id_suffix: "output_trim_db",
                range: ParamRange::Linear {
                    min: MIN_GAIN_DB as f64,
                    max: MAX_GAIN_DB as f64,
                },
                default: 0.0,
                unit: "dB",
                group: Some("Saturator"),
            },
        ];

        &SPECS
    }

    fn from_param_defaults() -> Self {
        Self {
            drive_db: 0.0,
            output_trim_db: 0.0,
        }
    }

    fn apply_plain_values(&mut self, values: &[f32]) {
        if let Some(drive_db) = values.first() {
            self.drive_db = *drive_db;
        }
        if let Some(output_trim_db) = values.get(1) {
            self.output_trim_db = *output_trim_db;
        }
    }
}

/// Soft-clip saturator DSP processor.
#[derive(Debug, Default)]
pub struct SaturatorDsp;

impl Processor for SaturatorDsp {
    type Params = SaturatorParams;

    fn process(
        &mut self,
        buffer: &mut [&mut [f32]],
        _transport: &Transport,
        params: &Self::Params,
    ) {
        let drive = db_to_linear(params.drive_db);
        let output_trim = db_to_linear(params.output_trim_db);

        for channel in buffer.iter_mut() {
            for sample in channel.iter_mut() {
                let driven = *sample * drive;
                *sample = soft_clip(driven) * output_trim;
            }
        }
    }
}

#[inline]
fn soft_clip(input: f32) -> f32 {
    input / (1.0 + input.abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process_mono(drive_db: f32, output_trim_db: f32, input: &[f32]) -> Vec<f32> {
        let mut processor = SaturatorDsp;
        let params = SaturatorParams {
            drive_db,
            output_trim_db,
        };

        let mut mono = input.to_vec();
        let mut buffer = [&mut mono[..]];
        processor.process(&mut buffer, &Transport::default(), &params);
        mono
    }

    #[test]
    fn param_specs_use_db_suffixes_and_group() {
        let specs = SaturatorParams::param_specs();
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].id_suffix, "drive_db");
        assert_eq!(specs[1].id_suffix, "output_trim_db");
        assert_eq!(specs[0].group, Some("Saturator"));
        assert_eq!(specs[1].unit, "dB");
    }

    #[test]
    fn soft_clip_is_bounded() {
        let output = process_mono(24.0, 0.0, &[10.0, -10.0, 100.0, -100.0]);

        for sample in output {
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn higher_drive_pushes_toward_saturation() {
        let input = [0.5_f32, -0.5_f32];
        let low_drive = process_mono(0.0, 0.0, &input);
        let high_drive = process_mono(18.0, 0.0, &input);

        assert!(high_drive[0].abs() > low_drive[0].abs());
        assert!(high_drive[1].abs() > low_drive[1].abs());
    }

    #[test]
    fn output_trim_reduces_level() {
        let input = [0.8_f32, -0.8_f32];
        let untrimmed = process_mono(12.0, 0.0, &input);
        let trimmed = process_mono(12.0, -12.0, &input);

        assert!(trimmed[0].abs() < untrimmed[0].abs());
        assert!(trimmed[1].abs() < untrimmed[1].abs());
    }
}
