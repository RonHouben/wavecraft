//! Test that Chain parameter extraction doesn't hang during initialization.
//!
//! This test verifies that nested SignalChain types can extract their
//! parameter specs without hanging, which was an issue when using OnceLock<Vec<_>>
//! in the ChainParams::param_specs() implementation.

use wavecraft_dsp::{
    Processor, ProcessorParams, SignalChain,
    builtins::{GainDsp, PassthroughDsp},
};

#[test]
fn test_simple_chain_param_extraction() {
    // SignalChain![A, B] expands to Chain<A, B>
    type SimpleChain = SignalChain![GainDsp, GainDsp];

    // This should complete without hanging
    let specs = <SimpleChain as Processor>::Params::param_specs();

    // Should have 2 gain parameters (one from each GainDsp)
    assert_eq!(specs.len(), 2);
    assert_eq!(specs[0].name, "Level");
    assert_eq!(specs[1].name, "Level");
}

#[test]
fn test_nested_chain_param_extraction() {
    // SignalChain![A, B, C] expands to Chain<A, Chain<B, C>>
    type NestedChain = SignalChain![GainDsp, GainDsp, PassthroughDsp];

    // This should complete without hanging
    let specs = <NestedChain as Processor>::Params::param_specs();

    // Should have 2 gain parameters (PassthroughDsp has no params)
    assert_eq!(specs.len(), 2);
}

#[test]
fn test_deeply_nested_chain_param_extraction() {
    // Deep nesting: SignalChain![A, B, C, D] -> Chain<A, Chain<B, Chain<C, D>>>
    type DeepChain = SignalChain![GainDsp, PassthroughDsp, GainDsp, PassthroughDsp];

    // This should complete without hanging (previously caused dlopen timeout)
    let specs = <DeepChain as Processor>::Params::param_specs();

    // Should have 2 gain parameters
    assert_eq!(specs.len(), 2);
    assert_eq!(specs[0].name, "Level");
    assert_eq!(specs[1].name, "Level");
}

#[test]
fn test_repeated_param_extraction() {
    // With the current implementation (Box::leak on every call), each call
    // allocates new memory. This is acceptable because param_specs() is only
    // called once at plugin load time.
    type TestChain = SignalChain![GainDsp, GainDsp];

    let specs1 = <TestChain as Processor>::Params::param_specs();
    let specs2 = <TestChain as Processor>::Params::param_specs();

    // Both should have the same content (even if not the same pointer)
    assert_eq!(specs1.len(), specs2.len());
    assert_eq!(specs1[0].name, specs2[0].name);
}
