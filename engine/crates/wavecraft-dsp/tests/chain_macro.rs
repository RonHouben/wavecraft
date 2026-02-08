//! Tests for the SignalChain! macro.

use wavecraft_dsp::builtins::{GainDsp, PassthroughDsp};
use wavecraft_dsp::combinators::Chain; // Import struct type, not macro
use wavecraft_dsp::{Processor, Transport};

// Import macro separately to avoid deprecation warning on struct import
use wavecraft_dsp::SignalChain;

#[test]
fn test_chain_macro_single_processor() {
    // SignalChain![T] should compile to just T (zero overhead)
    type SingleChain = SignalChain![GainDsp];

    // This should be exactly GainDsp, not wrapped
    let _processor: SingleChain = GainDsp::default();
}

#[test]
fn test_chain_macro_two_processors() {
    // SignalChain![A, B] should compile to Chain<A, B>
    type TwoChain = SignalChain![GainDsp, PassthroughDsp];

    let mut chain: TwoChain = Chain {
        first: GainDsp::default(),
        second: PassthroughDsp,
    };

    // Verify it's a valid Processor
    let mut left = [1.0_f32];
    let mut buffer = [&mut left[..]];
    let transport = Transport::default();
    let params = Default::default();

    chain.process(&mut buffer, &transport, &params);
}

#[test]
fn test_chain_macro_three_processors() {
    // SignalChain![A, B, C] should compile to Chain<A, Chain<B, C>>
    type ThreeChain = SignalChain![GainDsp, PassthroughDsp, GainDsp];

    let chain: ThreeChain = Chain {
        first: GainDsp::default(),
        second: Chain {
            first: PassthroughDsp,
            second: GainDsp::default(),
        },
    };

    // Type check that it's properly nested
    let _: &GainDsp = &chain.first;
    let _: &PassthroughDsp = &chain.second.first;
    let _: &GainDsp = &chain.second.second;
}
