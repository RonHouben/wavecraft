//! DSP crate - pure audio processing algorithms.
//!
//! This crate contains all DSP logic without any plugin framework dependencies,
//! making it fully testable in isolation.

pub mod gain;
pub mod processor;
pub mod traits;

// Re-export the core trait for user implementations
pub use traits::{Processor, Transport};

// Re-export the concrete gain processor
pub use processor::GainProcessor;
