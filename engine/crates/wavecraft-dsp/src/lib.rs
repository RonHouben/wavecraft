//! DSP crate - pure audio processing algorithms.
//!
//! This crate contains all DSP logic without any plugin framework dependencies,
//! making it fully testable in isolation.

pub mod combinators;
pub mod gain;
pub mod traits;

// Core DSP contracts.
pub use traits::{ParamRange, ParamSpec, Processor, ProcessorParams, Transport};

// Combinators and helpers.
pub use combinators::{Bypassed, Chain};

// Note: SignalChain! and Chain! macros are automatically exported at crate root
// via #[macro_export] in combinators/mod.rs
