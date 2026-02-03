//! Gain processor - amplifies or attenuates audio signals.

use crate::traits::{Processor, ProcessorParams, Transport};

/// Parameter struct for gain processor.
#[derive(Debug, Default, Clone)]
pub struct GainParams {
    /// Gain level in linear amplitude (0.0 = silence, 1.0 = unity, >1.0 = boost).
    pub level: f32,
}

impl ProcessorParams for GainParams {
    fn param_specs() -> &'static [crate::traits::ParamSpec] {
        use crate::traits::{ParamRange, ParamSpec};
        
        static SPECS: [ParamSpec; 1] = [
            ParamSpec {
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
            },
        ];
        &SPECS
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
    
    fn process(&mut self, buffer: &mut [&mut [f32]], _transport: &Transport, params: &Self::Params) {
        let gain = params.level;
        
        for channel in buffer.iter_mut() {
            for sample in channel.iter_mut() {
                *sample *= gain;
            }
        }
    }
    
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self._sample_rate = sample_rate;
    }
    
    fn reset(&mut self) {
        // No state to reset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unity_gain() {
        let mut processor = GainDsp::default();
        let mut left = [0.5, -0.5, 0.25];
        let mut right = [0.3, -0.3, 0.1];
        let mut buffer = [&mut left[..], &mut right[..]];
        
        let params = GainParams { level: 1.0 };
        let transport = Transport::default();
        
        processor.process(&mut buffer, &transport, &params);
        
        assert!((left[0] - 0.5).abs() < 1e-6);
        assert!((left[1] + 0.5).abs() < 1e-6);
        assert!((right[0] - 0.3).abs() < 1e-6);
    }
    
    #[test]
    fn test_boost() {
        let mut processor = GainDsp::default();
        let mut left = [1.0];
        let mut buffer = [&mut left[..]];
        
        let params = GainParams { level: 2.0 };
        let transport = Transport::default();
        
        processor.process(&mut buffer, &transport, &params);
        
        assert!((left[0] - 2.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_attenuation() {
        let mut processor = GainDsp::default();
        let mut left = [1.0];
        let mut buffer = [&mut left[..]];
        
        let params = GainParams { level: 0.5 };
        let transport = Transport::default();
        
        processor.process(&mut buffer, &transport, &params);
        
        assert!((left[0] - 0.5).abs() < 1e-6);
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
}
