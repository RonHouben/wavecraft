//! Test tone processor — a simple sine-wave tone generator.

use wavecraft_dsp::{ParamRange, ParamSpec, Processor, ProcessorParams, Transport};

/// Generate a single sine-wave sample for the given phase (0.0–1.0).
#[inline]
fn generate_sine_sample(phase: f32) -> f32 {
    (phase * std::f32::consts::TAU).sin()
}

#[inline]
fn advance_phase(phase: &mut f32, phase_delta: f32) {
    *phase += phase_delta;
    if *phase >= 1.0 {
        *phase -= 1.0;
    }
}

/// Test tone processor parameters.
#[derive(Clone)]
pub struct TestToneProcessorParams {
    /// Enable/disable test tone generation.
    pub enabled: bool,

    /// Frequency in Hz. `factor = 2.5` gives a logarithmic feel in the UI.
    pub frequency: f32,

    /// Output level as normalized amplitude (0.0 – 1.0).
    pub level: f32,
}

impl Default for TestToneProcessorParams {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: 440.0,
            level: 0.5,
        }
    }
}

impl ProcessorParams for TestToneProcessorParams {
    fn param_specs() -> &'static [ParamSpec] {
        static SPECS: [ParamSpec; 3] = [
            ParamSpec {
                name: "Enabled",
                id_suffix: "enabled",
                range: ParamRange::Stepped { min: 0, max: 1 },
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
        if let Some(frequency) = values.get(1) {
            self.frequency = *frequency;
        }
        if let Some(level) = values.get(2) {
            self.level = *level;
        }
    }
}

/// A minimal test tone source that produces a sine wave.
#[derive(Default)]
pub struct TestToneProcessor {
    /// Current sample rate provided by the host.
    sample_rate: f32,
    /// Phase position within one cycle (0.0 – 1.0).
    phase: f32,
}

impl Processor for TestToneProcessor {
    type Params = TestToneProcessorParams;

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

        if self.sample_rate == 0.0 {
            return;
        }

        let phase_delta = params.frequency / self.sample_rate;
        let start_phase = self.phase;

        for channel in buffer.iter_mut() {
            self.phase = start_phase;
            for sample in channel.iter_mut() {
                *sample += generate_sine_sample(self.phase) * params.level;
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
    use wavecraft_dsp::Bypassed;

    fn test_params() -> TestToneProcessorParams {
        TestToneProcessorParams {
            enabled: true,
            frequency: 440.0,
            level: 0.5,
        }
    }

    fn test_params_with_level(level: f32) -> TestToneProcessorParams {
        TestToneProcessorParams {
            enabled: true,
            frequency: 440.0,
            level,
        }
    }

    #[test]
    fn sine_wave_zero_crossing_and_peak() {
        assert!((generate_sine_sample(0.0)).abs() < 1e-5);
        assert!((generate_sine_sample(0.25) - 1.0).abs() < 1e-5);
        assert!((generate_sine_sample(0.5)).abs() < 1e-5);
        assert!((generate_sine_sample(0.75) + 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_tone_processor_preserves_passthrough_when_level_is_zero() {
        let mut test_tone = TestToneProcessor::default();
        test_tone.set_sample_rate(48_000.0);

        let mut left = [0.25_f32; 64];
        let mut right = [-0.5_f32; 64];
        let left_in = left;
        let right_in = right;
        let mut buffer = [&mut left[..], &mut right[..]];

        test_tone.process(
            &mut buffer,
            &Transport::default(),
            &test_params_with_level(0.0),
        );

        for (actual, expected) in left.iter().zip(left_in.iter()) {
            assert!((actual - expected).abs() <= f32::EPSILON);
        }

        for (actual, expected) in right.iter().zip(right_in.iter()) {
            assert!((actual - expected).abs() <= f32::EPSILON);
        }
    }

    #[test]
    fn test_tone_processor_generates_signal_on_silent_input() {
        let mut test_tone = TestToneProcessor::default();
        test_tone.set_sample_rate(48_000.0);

        let mut left = [0.0_f32; 128];
        let mut right = [0.0_f32; 128];
        let mut buffer = [&mut left[..], &mut right[..]];

        test_tone.process(&mut buffer, &Transport::default(), &test_params());

        let peak_left = left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
        let peak_right = right
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        assert!(
            peak_left > 0.01,
            "expected audible test tone output on left"
        );
        assert!(
            peak_right > 0.01,
            "expected audible test tone output on right"
        );
    }

    #[test]
    fn test_tone_processor_adds_signal_without_removing_input() {
        let mut test_tone_mixed = TestToneProcessor::default();
        test_tone_mixed.set_sample_rate(48_000.0);

        let mut left_mixed = [0.2_f32; 128];
        let mut right_mixed = [-0.15_f32; 128];
        let left_input = left_mixed;
        let right_input = right_mixed;
        let mut mixed_buffer = [&mut left_mixed[..], &mut right_mixed[..]];

        test_tone_mixed.process(&mut mixed_buffer, &Transport::default(), &test_params());

        let mut test_tone_only = TestToneProcessor::default();
        test_tone_only.set_sample_rate(48_000.0);

        let mut left_tone_only = [0.0_f32; 128];
        let mut right_tone_only = [0.0_f32; 128];
        let mut tone_only_buffer = [&mut left_tone_only[..], &mut right_tone_only[..]];

        test_tone_only.process(&mut tone_only_buffer, &Transport::default(), &test_params());

        for i in 0..left_mixed.len() {
            let additive_component_left = left_mixed[i] - left_input[i];
            let additive_component_right = right_mixed[i] - right_input[i];

            assert!((additive_component_left - left_tone_only[i]).abs() < 1e-6);
            assert!((additive_component_right - right_tone_only[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_tone_processor_bypass_wrapper_mutes_generator_output() {
        let mut wrapped = Bypassed::new(TestToneProcessor::default());
        wrapped.set_sample_rate(48_000.0);

        type WrappedParams = <Bypassed<TestToneProcessor> as Processor>::Params;
        let bypassed_params = WrappedParams {
            inner: test_params(),
            bypassed: true,
        };

        for _ in 0..4 {
            let mut left = [0.0_f32; 128];
            let mut right = [0.0_f32; 128];
            let mut buffer = [&mut left[..], &mut right[..]];

            wrapped.process(&mut buffer, &Transport::default(), &bypassed_params);
        }

        let mut left = [0.0_f32; 128];
        let mut right = [0.0_f32; 128];
        let mut buffer = [&mut left[..], &mut right[..]];
        wrapped.process(&mut buffer, &Transport::default(), &bypassed_params);

        let peak_left = left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
        let peak_right = right
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        assert!(
            peak_left <= 1e-6,
            "expected bypassed test tone to contribute no left-channel signal"
        );
        assert!(
            peak_right <= 1e-6,
            "expected bypassed test tone to contribute no right-channel signal"
        );
    }

    #[test]
    fn apply_plain_values_updates_all_fields() {
        let mut params = TestToneProcessorParams::default();
        params.apply_plain_values(&[1.0, 1760.0, 0.9]);

        assert!(params.enabled);
        assert!((params.frequency - 1760.0).abs() < f32::EPSILON);
        assert!((params.level - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_tone_processor_disabled_by_default() {
        let mut test_tone = TestToneProcessor::default();
        test_tone.set_sample_rate(48_000.0);

        let mut left = [0.0_f32; 128];
        let mut right = [0.0_f32; 128];
        let mut buffer = [&mut left[..], &mut right[..]];

        let params = TestToneProcessorParams::default();
        test_tone.process(&mut buffer, &Transport::default(), &params);

        assert!(left.iter().all(|sample| sample.abs() <= f32::EPSILON));
        assert!(right.iter().all(|sample| sample.abs() <= f32::EPSILON));
    }

    #[test]
    fn frequency_param_uses_full_audible_range() {
        let specs = TestToneProcessorParams::param_specs();
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
