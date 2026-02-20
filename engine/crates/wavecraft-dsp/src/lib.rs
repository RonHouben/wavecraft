//! DSP crate - pure audio processing algorithms.
//!
//! This crate contains all DSP logic without any plugin framework dependencies,
//! making it fully testable in isolation.

pub mod builtins;
pub mod combinators;
pub mod gain;
pub mod processor;
pub mod traits;

// Core DSP contracts.
pub use traits::{ParamRange, ParamSpec, Processor, ProcessorParams, Transport};

// Built-in processors and combinators.
pub use builtins::*;
pub use combinators::Chain;
pub use processor::GainProcessor;

// Note: SignalChain! and Chain! macros are automatically exported at crate root
// via #[macro_export] in combinators/mod.rs
