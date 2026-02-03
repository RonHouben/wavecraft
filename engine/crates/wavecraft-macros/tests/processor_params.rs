//! Tests for ProcessorParams derive macro

use wavecraft_dsp::{ParamRange, ProcessorParams};
use wavecraft_macros::ProcessorParams;

#[derive(ProcessorParams, Default)]
struct SimpleParams {
    #[param(range = "0.0..=1.0", default = 0.5)]
    value: f32,
}

#[test]
fn test_simple_param_specs() {
    let specs = SimpleParams::param_specs();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "Value");
    assert_eq!(specs[0].id_suffix, "value");
    assert_eq!(specs[0].default, 0.5);
    assert_eq!(specs[0].unit, "");
    
    match &specs[0].range {
        ParamRange::Linear { min, max } => {
            assert!((min - 0.0).abs() < 1e-6);
            assert!((max - 1.0).abs() < 1e-6);
        }
        _ => panic!("Expected Linear range"),
    }
}

#[derive(ProcessorParams, Default)]
struct MultiParamStruct {
    #[param(range = "-24.0..=24.0", default = 0.0, unit = "dB")]
    gain: f32,
    
    #[param(range = "20.0..=20000.0", factor = 2.5, unit = "Hz")]
    frequency: f32,
}

#[test]
fn test_multiple_params() {
    let specs = MultiParamStruct::param_specs();
    assert_eq!(specs.len(), 2);
    
    // Check first param (gain)
    assert_eq!(specs[0].name, "Gain");
    assert_eq!(specs[0].id_suffix, "gain");
    assert_eq!(specs[0].unit, "dB");
    assert_eq!(specs[0].default, 0.0);
    
    // Check second param (frequency)
    assert_eq!(specs[1].name, "Frequency");
    assert_eq!(specs[1].id_suffix, "frequency");
    assert_eq!(specs[1].unit, "Hz");
    
    // Should have skewed range due to factor
    match &specs[1].range {
        ParamRange::Skewed { min, max, factor } => {
            assert!((min - 20.0).abs() < 1e-6);
            assert!((max - 20000.0).abs() < 1e-6);
            assert!((factor - 2.5).abs() < 1e-6);
        }
        _ => panic!("Expected Skewed range for frequency"),
    }
}

#[derive(ProcessorParams, Default)]
struct NoDefaultParam {
    #[param(range = "0.0..=10.0")]
    level: f32,
}

#[test]
fn test_default_value_calculation() {
    let specs = NoDefaultParam::param_specs();
    assert_eq!(specs.len(), 1);
    // Default should be midpoint: (0 + 10) / 2 = 5.0
    assert_eq!(specs[0].default, 5.0);
}
