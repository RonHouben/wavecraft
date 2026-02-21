//! Tests for the SignalChain! macro.

use wavecraft_dsp::combinators::{Bypassed, Chain}; // Import struct types, not macro
use wavecraft_dsp::{Processor, Transport};

// Import macro separately to avoid deprecation warning on struct import
use wavecraft_dsp::SignalChain;

#[derive(Default)]
struct TestGain;

impl Processor for TestGain {
    type Params = ();

    fn process(
        &mut self,
        buffer: &mut [&mut [f32]],
        _transport: &Transport,
        _params: &Self::Params,
    ) {
        for channel in buffer.iter_mut() {
            for sample in channel.iter_mut() {
                *sample *= 1.0;
            }
        }
    }
}

#[derive(Default)]
struct TestPassthrough;

impl Processor for TestPassthrough {
    type Params = ();

    fn process(
        &mut self,
        _buffer: &mut [&mut [f32]],
        _transport: &Transport,
        _params: &Self::Params,
    ) {
    }
}

#[test]
fn test_chain_macro_single_processor() {
    // SignalChain![T] compiles to Bypassed<T>
    type SingleChain = SignalChain![TestGain];

    let _processor: SingleChain = Bypassed::new(TestGain);
}

#[test]
fn test_chain_macro_two_processors() {
    // SignalChain![A, B] compiles to Chain<Bypassed<A>, Bypassed<B>>
    type TwoChain = SignalChain![TestGain, TestPassthrough];

    let mut chain: TwoChain = Chain {
        first: Bypassed::new(TestGain),
        second: Bypassed::new(TestPassthrough),
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
    // SignalChain![A, B, C] compiles to
    // Chain<Bypassed<A>, Chain<Bypassed<B>, Bypassed<C>>>
    type ThreeChain = SignalChain![TestGain, TestPassthrough, TestGain];

    let chain: ThreeChain = Chain {
        first: Bypassed::new(TestGain),
        second: Chain {
            first: Bypassed::new(TestPassthrough),
            second: Bypassed::new(TestGain),
        },
    };

    // Type check that it's properly nested
    let _: &Bypassed<TestGain> = &chain.first;
    let _: &Bypassed<TestPassthrough> = &chain.second.first;
    let _: &Bypassed<TestGain> = &chain.second.second;
}
