//! Oscillator — a simple sine-wave generator.

use wavecraft_dsp::{ParamRange, ParamSpec, Processor, ProcessorParams, Transport};

/// Available oscillator waveform shapes.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Waveform {
    #[default]
    Sine,
    Square,
    Saw,
    Triangle,
}

impl Waveform {
    /// Variant labels in declaration order (must match enum discriminant order).
    pub const VARIANTS: &'static [&'static str] = &["Sine", "Square", "Saw", "Triangle"];

    /// Convert a 0-based index to a `Waveform`.
    /// Out-of-range values default to `Sine`.
    pub fn from_index(index: f32) -> Self {
        match index.round() as u32 {
            0 => Self::Sine,
            1 => Self::Square,
            2 => Self::Saw,
            3 => Self::Triangle,
            _ => Self::Sine,
        }
    }
}

/// Generate a single sample for the given waveform at the given phase (0.0–1.0).
pub fn generate_waveform_sample(waveform: Waveform, phase: f32) -> f32 {
    match waveform {
        Waveform::Sine => (phase * std::f32::consts::TAU).sin(),
        Waveform::Square => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Saw => 2.0 * phase - 1.0,
        Waveform::Triangle => {
            if phase < 0.5 {
                4.0 * phase - 1.0
            } else {
                -4.0 * phase + 3.0
            }
        }
    }
}

#[inline]
fn advance_phase(phase: &mut f32, phase_delta: f32) {
    // Advance phase, wrapping at 1.0 to avoid floating-point drift.
    *phase += phase_delta;
    if *phase >= 1.0 {
        *phase -= 1.0;
    }
}

/// Oscillator parameters.
#[derive(Clone)]
pub struct OscillatorParams {
    /// Enable/disable oscillator output.
    pub enabled: bool,

    /// Waveform index mapped through [`Waveform::from_index`].
    pub waveform: f32,

    /// Frequency in Hz. `factor = 2.5` gives a logarithmic feel in the UI.
    pub frequency: f32,

    /// Output level as normalized amplitude (0.0 – 1.0).
    pub level: f32,
}

impl Default for OscillatorParams {
    fn default() -> Self {
        Self {
            enabled: false,
            waveform: 0.0,
            frequency: 440.0,
            level: 0.5,
        }
    }
}

impl ProcessorParams for OscillatorParams {
    fn param_specs() -> &'static [ParamSpec] {
        static SPECS: [ParamSpec; 4] = [
            ParamSpec {
                name: "Enabled",
                id_suffix: "enabled",
                range: ParamRange::Stepped { min: 0, max: 1 },
                default: 0.0,
                unit: "",
                group: None,
            },
            ParamSpec {
                name: "Waveform",
                id_suffix: "waveform",
                range: ParamRange::Enum {
                    variants: Waveform::VARIANTS,
                },
                default: 0.0,
                unit: "",
                group: None,
            },
            ParamSpec {
                name: "Frequency",
                id_suffix: "frequency",
                range: ParamRange::Skewed {
                    min: 20.0,
                    max: 20_000.0,
                    factor: 2.5,
                },
                default: 440.0,
                unit: "Hz",
                group: None,
            },
            ParamSpec {
                name: "Level",
                id_suffix: "level",
                range: ParamRange::Linear { min: 0.0, max: 1.0 },
                default: 0.5,
                unit: "%",
                group: None,
            },
        ];

        &SPECS
    }

    fn from_param_defaults() -> Self {
        Self::default()
    }

    fn apply_plain_values(&mut self, values: &[f32]) {
        if let Some(enabled) = values.first() {
            self.enabled = *enabled >= 0.5;
        }
        if let Some(waveform) = values.get(1) {
            self.waveform = *waveform;
        }
        if let Some(frequency) = values.get(2) {
            self.frequency = *frequency;
        }
        if let Some(level) = values.get(3) {
            self.level = *level;
        }
    }
}

/// A minimal oscillator that produces multiple waveforms.
#[derive(Default)]
pub struct Oscillator {
    /// Current sample rate provided by the host.
    sample_rate: f32,
    /// Phase position within one cycle (0.0 – 1.0).
    phase: f32,
}

impl Processor for Oscillator {
    type Params = OscillatorParams;

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        buffer: &mut [&mut [f32]],
        _transport: &Transport,
        params: &Self::Params,
    ) {
        if !params.enabled {
            return;
        }

        // Guard: if set_sample_rate() hasn't been called yet, leave buffer unchanged.
        if self.sample_rate == 0.0 {
            return;
        }

        let waveform = Waveform::from_index(params.waveform);

        // How far the phase advances per sample.
        let phase_delta = params.frequency / self.sample_rate;

        // Save the starting phase so every channel receives the same waveform.
        let start_phase = self.phase;

        for channel in buffer.iter_mut() {
            self.phase = start_phase;
            for sample in channel.iter_mut() {
                *sample += generate_waveform_sample(waveform, self.phase) * params.level;
                advance_phase(&mut self.phase, phase_delta);
            }
        }
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_params(enabled: bool) -> OscillatorParams {
        OscillatorParams {
            enabled,
            waveform: 0.0,
            frequency: 440.0,
            level: 0.5,
        }
    }

    fn test_params_with_waveform(enabled: bool, waveform: f32) -> OscillatorParams {
        OscillatorParams {
            enabled,
            waveform,
            frequency: 440.0,
            level: 0.5,
        }
    }

    #[test]
    fn waveform_from_index_maps_correctly() {
        assert_eq!(Waveform::from_index(0.0), Waveform::Sine);
        assert_eq!(Waveform::from_index(1.0), Waveform::Square);
        assert_eq!(Waveform::from_index(2.0), Waveform::Saw);
        assert_eq!(Waveform::from_index(3.0), Waveform::Triangle);
    }

    #[test]
    fn waveform_from_index_out_of_range_defaults_to_sine() {
        assert_eq!(Waveform::from_index(-1.0), Waveform::Sine);
        assert_eq!(Waveform::from_index(4.0), Waveform::Sine);
        assert_eq!(Waveform::from_index(100.0), Waveform::Sine);
    }

    #[test]
    fn waveform_from_index_rounds_floats() {
        assert_eq!(Waveform::from_index(0.4), Waveform::Sine);
        assert_eq!(Waveform::from_index(0.6), Waveform::Square);
        assert_eq!(Waveform::from_index(1.5), Waveform::Saw);
        assert_eq!(Waveform::from_index(2.7), Waveform::Triangle);
    }

    #[test]
    fn sine_wave_zero_crossing_and_peak() {
        assert!((generate_waveform_sample(Waveform::Sine, 0.0)).abs() < 1e-5);
        assert!((generate_waveform_sample(Waveform::Sine, 0.25) - 1.0).abs() < 1e-5);
        assert!((generate_waveform_sample(Waveform::Sine, 0.5)).abs() < 1e-5);
        assert!((generate_waveform_sample(Waveform::Sine, 0.75) + 1.0).abs() < 1e-5);
    }

    #[test]
    fn square_wave_values() {
        assert_eq!(generate_waveform_sample(Waveform::Square, 0.0), 1.0);
        assert_eq!(generate_waveform_sample(Waveform::Square, 0.25), 1.0);
        assert_eq!(generate_waveform_sample(Waveform::Square, 0.5), -1.0);
        assert_eq!(generate_waveform_sample(Waveform::Square, 0.75), -1.0);
    }

    #[test]
    fn saw_wave_values() {
        assert!((generate_waveform_sample(Waveform::Saw, 0.0) + 1.0).abs() < 1e-5);
        assert!((generate_waveform_sample(Waveform::Saw, 0.5)).abs() < 1e-5);
        assert!((generate_waveform_sample(Waveform::Saw, 1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn triangle_wave_values() {
        assert!((generate_waveform_sample(Waveform::Triangle, 0.0) + 1.0).abs() < 1e-5);
        assert!((generate_waveform_sample(Waveform::Triangle, 0.25)).abs() < 1e-5);
        assert!((generate_waveform_sample(Waveform::Triangle, 0.5) - 1.0).abs() < 1e-5);
        assert!((generate_waveform_sample(Waveform::Triangle, 0.75)).abs() < 1e-5);
    }

    #[test]
    fn oscillator_preserves_passthrough_when_disabled() {
        let mut osc = Oscillator::default();
        osc.set_sample_rate(48_000.0);

        let mut left = [0.25_f32; 64];
        let mut right = [-0.5_f32; 64];
        let left_in = left;
        let right_in = right;
        let mut buffer = [&mut left[..], &mut right[..]];

        osc.process(&mut buffer, &Transport::default(), &test_params(false));

        for (actual, expected) in left.iter().zip(left_in.iter()) {
            assert!((actual - expected).abs() <= f32::EPSILON);
        }

        for (actual, expected) in right.iter().zip(right_in.iter()) {
            assert!((actual - expected).abs() <= f32::EPSILON);
        }
    }

    #[test]
    fn oscillator_generates_signal_when_enabled_on_silent_input() {
        let mut osc = Oscillator::default();
        osc.set_sample_rate(48_000.0);

        let mut left = [0.0_f32; 128];
        let mut right = [0.0_f32; 128];
        let mut buffer = [&mut left[..], &mut right[..]];

        osc.process(&mut buffer, &Transport::default(), &test_params(true));

        let peak_left = left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
        let peak_right = right
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        assert!(
            peak_left > 0.01,
            "expected audible oscillator output on left"
        );
        assert!(
            peak_right > 0.01,
            "expected audible oscillator output on right"
        );
    }

    #[test]
    fn oscillator_enabled_adds_signal_without_removing_input() {
        let mut osc_mixed = Oscillator::default();
        osc_mixed.set_sample_rate(48_000.0);

        let mut left_mixed = [0.2_f32; 128];
        let mut right_mixed = [-0.15_f32; 128];
        let left_input = left_mixed;
        let right_input = right_mixed;
        let mut mixed_buffer = [&mut left_mixed[..], &mut right_mixed[..]];

        osc_mixed.process(&mut mixed_buffer, &Transport::default(), &test_params(true));

        let mut osc_only = Oscillator::default();
        osc_only.set_sample_rate(48_000.0);

        let mut left_osc_only = [0.0_f32; 128];
        let mut right_osc_only = [0.0_f32; 128];
        let mut osc_only_buffer = [&mut left_osc_only[..], &mut right_osc_only[..]];

        osc_only.process(
            &mut osc_only_buffer,
            &Transport::default(),
            &test_params(true),
        );

        for i in 0..left_mixed.len() {
            let additive_component_left = left_mixed[i] - left_input[i];
            let additive_component_right = right_mixed[i] - right_input[i];

            assert!((additive_component_left - left_osc_only[i]).abs() < 1e-6);
            assert!((additive_component_right - right_osc_only[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn all_waveforms_produce_signal_when_enabled() {
        for waveform_index in 0..4 {
            let mut osc = Oscillator::default();
            osc.set_sample_rate(48_000.0);

            let mut left = [0.0_f32; 128];
            let mut right = [0.0_f32; 128];
            let mut buffer = [&mut left[..], &mut right[..]];

            osc.process(
                &mut buffer,
                &Transport::default(),
                &test_params_with_waveform(true, waveform_index as f32),
            );

            let peak = left
                .iter()
                .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
            assert!(
                peak > 0.01,
                "waveform index {waveform_index} should produce signal"
            );
        }
    }

    #[test]
    fn apply_plain_values_updates_all_fields() {
        let mut params = OscillatorParams::default();
        params.apply_plain_values(&[1.0, 2.0, 1760.0, 0.9]);

        assert!(params.enabled);
        assert!((params.waveform - 2.0).abs() < f32::EPSILON);
        assert!((params.frequency - 1760.0).abs() < f32::EPSILON);
        assert!((params.level - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn frequency_param_uses_full_audible_range() {
        let specs = OscillatorParams::param_specs();
        let frequency = specs
            .iter()
            .find(|spec| spec.id_suffix == "frequency")
            .expect("frequency spec should exist");

        match frequency.range {
            ParamRange::Skewed { min, max, .. } => {
                assert!((min - 20.0).abs() < f64::EPSILON);
                assert!((max - 20_000.0).abs() < f64::EPSILON);
            }
            _ => panic!("frequency should use a skewed range"),
        }
    }
}
