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
fn generate_sample(waveform: Waveform, phase: f32) -> f32 {
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

/// Oscillator parameters.
#[derive(Clone)]
pub struct OscillatorParams {
    /// Enable/disable oscillator output.
    pub enabled: bool,

    /// Waveform index mapped through [`Waveform::from_index`].
    pub waveform: f32,

    /// Frequency in Hz. `factor = 2.5` gives a logarithmic feel in the UI.
    pub frequency: f32,

    /// Output level (0 % – 100 %).
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
                    max: 5000.0,
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
            for channel in buffer.iter_mut() {
                channel.fill(0.0);
            }
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
                *sample = generate_sample(waveform, self.phase) * params.level;

                // Advance phase, wrapping at 1.0 to avoid floating-point drift.
                self.phase += phase_delta;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
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
        assert!((generate_sample(Waveform::Sine, 0.0)).abs() < 1e-5);
        assert!((generate_sample(Waveform::Sine, 0.25) - 1.0).abs() < 1e-5);
        assert!((generate_sample(Waveform::Sine, 0.5)).abs() < 1e-5);
        assert!((generate_sample(Waveform::Sine, 0.75) + 1.0).abs() < 1e-5);
    }

    #[test]
    fn square_wave_values() {
        assert_eq!(generate_sample(Waveform::Square, 0.0), 1.0);
        assert_eq!(generate_sample(Waveform::Square, 0.25), 1.0);
        assert_eq!(generate_sample(Waveform::Square, 0.5), -1.0);
        assert_eq!(generate_sample(Waveform::Square, 0.75), -1.0);
    }

    #[test]
    fn saw_wave_values() {
        assert!((generate_sample(Waveform::Saw, 0.0) + 1.0).abs() < 1e-5);
        assert!((generate_sample(Waveform::Saw, 0.5)).abs() < 1e-5);
        assert!((generate_sample(Waveform::Saw, 1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn triangle_wave_values() {
        assert!((generate_sample(Waveform::Triangle, 0.0) + 1.0).abs() < 1e-5);
        assert!((generate_sample(Waveform::Triangle, 0.25)).abs() < 1e-5);
        assert!((generate_sample(Waveform::Triangle, 0.5) - 1.0).abs() < 1e-5);
        assert!((generate_sample(Waveform::Triangle, 0.75)).abs() < 1e-5);
    }

    #[test]
    fn oscillator_outputs_silence_when_disabled() {
        let mut osc = Oscillator::default();
        osc.set_sample_rate(48_000.0);

        let mut left = [1.0_f32; 64];
        let mut right = [1.0_f32; 64];
        let mut buffer = [&mut left[..], &mut right[..]];

        osc.process(&mut buffer, &Transport::default(), &test_params(false));

        assert!(left.iter().all(|s| s.abs() <= f32::EPSILON));
        assert!(right.iter().all(|s| s.abs() <= f32::EPSILON));
    }

    #[test]
    fn oscillator_outputs_signal_when_enabled() {
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

            let peak = left.iter().fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
            assert!(
                peak > 0.01,
                "waveform index {waveform_index} should produce signal"
            );
        }
    }
}
