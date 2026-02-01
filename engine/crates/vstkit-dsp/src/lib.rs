//! DSP crate - pure audio processing algorithms.
//!
//! This crate contains all DSP logic without any plugin framework dependencies,
//! making it fully testable in isolation.

pub mod gain;
pub mod processor;

pub use processor::Processor;
