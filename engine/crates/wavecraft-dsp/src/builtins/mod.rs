//! Built-in DSP processors ready to use.
//!
//! This module provides common audio processors with ProcessorParams
//! implementations, enabling them to work with the declarative DSL.

pub mod gain;
pub mod passthrough;

pub use gain::{GainDsp, GainParams};
pub use passthrough::{PassthroughDsp, PassthroughParams};
