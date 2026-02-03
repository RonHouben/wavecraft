//! Tests for the wavecraft_processor! macro.

use wavecraft_core::wavecraft_processor;
use wavecraft_dsp::{Processor, Transport};

// Generate wrapper types for built-in processors
wavecraft_processor!(InputGain => Gain);
wavecraft_processor!(OutputGain => Gain);
wavecraft_processor!(Bypass => Passthrough);

#[test]
fn test_processor_macro_generates_default() {
    let _input_gain = InputGain::default();
    let _output_gain = OutputGain::default();
    let _bypass = Bypass::default();
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
