//! Gain processor - amplifies or attenuates audio signals.

use wavecraft_dsp::{ParamRange, ParamSpec, Processor, ProcessorParams, Transport};

/// Parameter struct for gain processor.
#[derive(Debug, Default, Clone)]
pub struct GainParams {
    /// Gain level in linear amplitude (0.0 = silence, 1.0 = unity, >1.0 = boost).
    pub level: f32,
}

impl ProcessorParams for GainParams {
    fn param_specs() -> &'static [ParamSpec] {
        static SPECS: [ParamSpec; 1] = [ParamSpec {
            name: "Level",
            id_suffix: "level",
            range: ParamRange::Skewed {
                min: 0.0,
                max: 2.0,
                factor: 2.5, // Logarithmic feel
            },
            default: 1.0,
            unit: "x",
            group: None,
        }];
        &SPECS
    }

    fn from_param_defaults() -> Self {
        Self { level: 1.0 }
    }

    fn apply_plain_values(&mut self, values: &[f32]) {
        if let Some(level) = values.first() {
            self.level = *level;
        }
    }
}

/// Gain processor - applies amplitude scaling to audio.
///
/// This is a simple but essential DSP building block that multiplies
/// all samples by a gain factor.
#[derive(Debug, Default)]
pub struct GainDsp {
    _sample_rate: f32,
}

impl Processor for GainDsp {
    type Params = GainParams;

    fn process(
        &mut self,
        buffer: &mut [&mut [f32]],
        _transport: &Transport,
        params: &Self::Params,
    ) {
        let gain = params.level;

        for channel in buffer.iter_mut() {
            apply_gain_to_channel(channel, gain);
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self._sample_rate = sample_rate;
    }

    fn reset(&mut self) {
        // No state to reset
    }
}

#[inline]
fn apply_gain_to_channel(channel: &mut [f32], gain: f32) {
    for sample in channel.iter_mut() {
        *sample *= gain;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process_with_gain(buffer: &mut [&mut [f32]], level: f32) {
        let mut processor = GainDsp::default();
        let params = GainParams { level };
        let transport = Transport::default();
        processor.process(buffer, &transport, &params);
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 1e-6);
    }

    #[test]
    fn test_unity_gain() {
        let mut left = [0.5, -0.5, 0.25];
        let mut right = [0.3, -0.3, 0.1];
        let mut buffer = [&mut left[..], &mut right[..]];

        process_with_gain(&mut buffer, 1.0);

        assert_close(left[0], 0.5);
        assert_close(left[1], -0.5);
        assert_close(right[0], 0.3);
    }

    #[test]
    fn test_boost() {
        let mut left = [1.0];
        let mut buffer = [&mut left[..]];

        process_with_gain(&mut buffer, 2.0);

        assert_close(left[0], 2.0);
    }

    #[test]
    fn test_attenuation() {
        let mut left = [1.0];
        let mut buffer = [&mut left[..]];

        process_with_gain(&mut buffer, 0.5);

        assert_close(left[0], 0.5);
    }

    #[test]
    fn test_param_specs() {
        let specs = GainParams::param_specs();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "Level");
        assert_eq!(specs[0].id_suffix, "level");
        assert_eq!(specs[0].default, 1.0);
        assert_eq!(specs[0].unit, "x");
    }

    #[test]
    fn test_from_param_defaults_uses_spec_default() {
        let defaults = GainParams::from_param_defaults();
        assert!((defaults.level - 1.0).abs() < 1e-6);
    }
}
