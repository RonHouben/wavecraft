//! DSP crate - pure audio processing algorithms.
//!
//! This crate contains all DSP logic without any plugin framework dependencies,
//! making it fully testable in isolation.

pub mod builtins;
pub mod combinators;
pub mod gain;
pub mod processor;
pub mod traits;

// Re-export the core trait for user implementations
pub use traits::{ParamRange, ParamSpec, Processor, ProcessorParams, Transport};

// Re-export the concrete gain processor (legacy)
pub use processor::GainProcessor;

// Re-export built-in processors
pub use builtins::*;

// Re-export combinators
pub use combinators::Chain;

// Re-export SignalChain! and Chain! macros
// Note: Chain! is deprecated in favor of SignalChain!
#[doc(inline)]
pub use crate::{Chain, SignalChain};
