//! Soft-clip saturator processor.

use wavecraft_dsp::{ParamRange, ParamSpec, Processor, ProcessorParams, Transport};
use wavecraft_protocol::db_to_linear;

const MIN_DRIVE_DB: f32 = 0.0;
const MAX_DRIVE_DB: f32 = 30.0;
const MIN_OUTPUT_DB: f32 = -24.0;
const MAX_OUTPUT_DB: f32 = 24.0;
const MIN_UNIT: f32 = 0.0;
const MAX_UNIT: f32 = 1.0;

/// Parameters for the soft-clip saturator.
#[derive(Debug, Clone)]
pub struct SaturatorParams {
    /// Input drive in dB before saturation.
    pub drive_db: f32,
    /// Output level in dB after saturation.
    pub output_db: f32,
    /// Dry/wet blend where 0 = dry and 1 = fully saturated.
    pub mix: f32,
    /// Tonal warmth control where 0 = warm and 1 = bright.
    pub tone: f32,
}

impl Default for SaturatorParams {
    fn default() -> Self {
        Self::from_param_defaults()
    }
}

impl ProcessorParams for SaturatorParams {
    fn param_specs() -> &'static [ParamSpec] {
        static SPECS: [ParamSpec; 4] = [
            ParamSpec {
                name: "Drive",
                id_suffix: "drive_db",
                range: ParamRange::Linear {
                    min: MIN_DRIVE_DB as f64,
                    max: MAX_DRIVE_DB as f64,
                },
                default: 12.0,
                unit: "dB",
                group: Some("Saturator"),
            },
            ParamSpec {
                name: "Output",
                id_suffix: "output_db",
                range: ParamRange::Linear {
                    min: MIN_OUTPUT_DB as f64,
                    max: MAX_OUTPUT_DB as f64,
                },
                default: 0.0,
                unit: "dB",
                group: Some("Saturator"),
            },
            ParamSpec {
                name: "Mix",
                id_suffix: "mix",
                range: ParamRange::Linear {
                    min: MIN_UNIT as f64,
                    max: MAX_UNIT as f64,
                },
                default: 1.0,
                unit: "%",
                group: Some("Saturator"),
            },
            ParamSpec {
                name: "Tone",
                id_suffix: "tone",
                range: ParamRange::Linear {
                    min: MIN_UNIT as f64,
                    max: MAX_UNIT as f64,
                },
                default: 0.55,
                unit: "%",
                group: Some("Saturator"),
            },
        ];

        &SPECS
    }

    fn from_param_defaults() -> Self {
        Self {
            drive_db: 12.0,
            output_db: 0.0,
            mix: 1.0,
            tone: 0.55,
        }
    }

    fn apply_plain_values(&mut self, values: &[f32]) {
        if let Some(drive_db) = values.first() {
            self.drive_db = *drive_db;
        }
        if let Some(output_db) = values.get(1) {
            self.output_db = *output_db;
        }
        if let Some(mix) = values.get(2) {
            self.mix = *mix;
        }
        if let Some(tone) = values.get(3) {
            self.tone = *tone;
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
        let output = db_to_linear(params.output_db);
        let mix = params.mix.clamp(MIN_UNIT, MAX_UNIT);
        let tone = params.tone.clamp(MIN_UNIT, MAX_UNIT);

        for channel in buffer.iter_mut() {
            for sample in channel.iter_mut() {
                let dry = *sample;
                let driven = dry * drive;
                let wet = warm_soft_clip(driven, tone) * output;
                *sample = dry + (wet - dry) * mix;
            }
        }
    }
}

#[inline]
fn warm_soft_clip(input: f32, tone: f32) -> f32 {
    let tone = tone.clamp(MIN_UNIT, MAX_UNIT);

    // Lower tone values increase damping in the nonlinear stage for a warmer response.
    let pre_emphasis = lerp(0.85, 1.2, tone);
    let emphasized = input * pre_emphasis;
    let clipped = emphasized / (1.0 + emphasized.abs());

    // Cubic damping adds warmth without introducing any state or allocations.
    let warmth_amount = 0.35 * (1.0 - tone);
    let warmed = clipped - warmth_amount * clipped * clipped * clipped;

    warmed / pre_emphasis
}

#[inline]
fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process_mono(drive_db: f32, output_db: f32, mix: f32, tone: f32, input: &[f32]) -> Vec<f32> {
        let mut processor = SaturatorDsp;
        let params = SaturatorParams {
            drive_db,
            output_db,
            mix,
            tone,
        };

        let mut mono = input.to_vec();
        let mut buffer = [&mut mono[..]];
        processor.process(&mut buffer, &Transport::default(), &params);
        mono
    }

    #[test]
    fn param_specs_use_db_suffixes_and_group() {
        let specs = SaturatorParams::param_specs();
        assert_eq!(specs.len(), 4);
        assert_eq!(specs[0].id_suffix, "drive_db");
        assert_eq!(specs[1].id_suffix, "output_db");
        assert_eq!(specs[2].id_suffix, "mix");
        assert_eq!(specs[3].id_suffix, "tone");
        assert_eq!(specs[0].group, Some("Saturator"));
        assert_eq!(specs[1].unit, "dB");
    }

    #[test]
    fn soft_clip_is_bounded() {
        let output = process_mono(30.0, 0.0, 1.0, 0.5, &[10.0, -10.0, 100.0, -100.0]);

        for sample in output {
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn higher_drive_pushes_toward_saturation() {
        let input = [0.5_f32, -0.5_f32];
        let low_drive = process_mono(0.0, 0.0, 1.0, 0.5, &input);
        let high_drive = process_mono(18.0, 0.0, 1.0, 0.5, &input);

        assert!(high_drive[0].abs() > low_drive[0].abs());
        assert!(high_drive[1].abs() > low_drive[1].abs());
    }

    #[test]
    fn output_level_reduces_level() {
        let input = [0.8_f32, -0.8_f32];
        let unity = process_mono(12.0, 0.0, 1.0, 0.5, &input);
        let reduced = process_mono(12.0, -12.0, 1.0, 0.5, &input);

        assert!(reduced[0].abs() < unity[0].abs());
        assert!(reduced[1].abs() < unity[1].abs());
    }

    #[test]
    fn mix_blends_dry_and_wet() {
        let input = [0.35_f32, -0.35_f32];
        let dry = process_mono(18.0, 0.0, 0.0, 0.5, &input);
        let wet = process_mono(18.0, 0.0, 1.0, 0.5, &input);
        let blended = process_mono(18.0, 0.0, 0.5, 0.5, &input);

        assert_eq!(dry, input);
        assert!(wet[0] != input[0]);
        assert!(blended[0] != dry[0]);
        assert!(blended[0] != wet[0]);
    }

    #[test]
    fn tone_changes_warmth_curve() {
        let input = [0.75_f32, -0.75_f32];
        let warm = process_mono(14.0, 0.0, 1.0, 0.0, &input);
        let bright = process_mono(14.0, 0.0, 1.0, 1.0, &input);

        assert_ne!(warm[0], bright[0]);
        assert_ne!(warm[1], bright[1]);
    }
}
