//! Tests for the wavecraft_processor! macro.

use wavecraft_core::wavecraft_processor;
use wavecraft_dsp::{Processor, Transport};

// Generate wrapper types for built-in processors
wavecraft_processor!(InputGain => Gain);
wavecraft_processor!(OutputGain => Gain);
wavecraft_processor!(Bypass => Passthrough);
wavecraft_processor!(ToneFilter => Filter);
wavecraft_processor!(SoftClip => Saturator);

#[test]
fn test_processor_macro_generates_default() {
    let _input_gain = InputGain::default();
    let _output_gain = OutputGain::default();
    let _bypass = Bypass::default();
    let _tone_filter = ToneFilter::default();
    let _soft_clip = SoftClip::default();
}

#[test]
fn test_processor_macro_implements_processor_trait() {
    let mut input_gain = InputGain::default();

    let mut left = [1.0_f32, 1.0_f32];
    let mut right = [1.0_f32, 1.0_f32];
    let mut buffer = [&mut left[..], &mut right[..]];

    let transport = Transport::default();
    let params = Default::default();

    // Should compile and run without panic
    input_gain.process(&mut buffer, &transport, &params);
}

#[test]
fn test_processor_types_are_distinct() {
    // Each generated type should be distinct, even if wrapping the same inner type
    fn takes_input_gain(_: InputGain) {}
    fn takes_output_gain(_: OutputGain) {}

    let input = InputGain::default();
    let output = OutputGain::default();

    takes_input_gain(input);
    takes_output_gain(output);

    // This should NOT compile (types are distinct):
    // takes_input_gain(output);  // Error: expected InputGain, found OutputGain
}

#[test]
fn test_passthrough_wrapper() {
    let mut bypass = Bypass::default();

    let mut left = [2.0_f32, 3.0_f32];
    let mut right = [4.0_f32, 5.0_f32];
    let mut buffer = [&mut left[..], &mut right[..]];

    let transport = Transport::default();
    let params = Default::default();

    bypass.process(&mut buffer, &transport, &params);

    // Passthrough should not modify the buffer
    assert_eq!(buffer[0][0], 2.0);
    assert_eq!(buffer[0][1], 3.0);
    assert_eq!(buffer[1][0], 4.0);
    assert_eq!(buffer[1][1], 5.0);
}

#[test]
fn test_filter_wrapper_supports_stateful_processor_methods() {
    let mut filter = ToneFilter::default();

    filter.set_sample_rate(48_000.0);

    let mut mono = [1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32];
    let mut buffer = [&mut mono[..]];
    let transport = Transport::default();
    let params = <ToneFilter as Processor>::Params::default();

    filter.process(&mut buffer, &transport, &params);
    filter.reset();

    assert!(buffer[0].iter().all(|sample| sample.is_finite()));
}

#[test]
fn test_saturator_wrapper_processes_audio() {
    let mut saturator = SoftClip::default();

    let mut mono = [2.0_f32, -2.0_f32];
    let mut buffer = [&mut mono[..]];
    let transport = Transport::default();
    let params = <SoftClip as Processor>::Params::default();

    saturator.process(&mut buffer, &transport, &params);

    assert!(buffer[0][0].abs() < 2.0);
    assert!(buffer[0][1].abs() < 2.0);
}
