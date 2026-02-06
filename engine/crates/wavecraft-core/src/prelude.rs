//! Prelude module - Common imports for Wavecraft SDK users.
//!
//! This module re-exports the most commonly used types and traits for building
//! plugins with Wavecraft. Import this to get started quickly:
//!
//! ```rust,ignore
//! use wavecraft_core::prelude::*;
//! ```
//!
//! For full plugin development with nih-plug integration, use the
//! `wavecraft-nih_plug` crate's prelude instead:
//!
//! ```rust,ignore
//! use wavecraft::prelude::*;  // via wavecraft-nih_plug
//! ```

// Re-export Wavecraft DSP traits and types
pub use wavecraft_dsp::{Chain, ParamRange, ParamSpec, Processor, ProcessorParams, Transport};

// Re-export built-in processors
pub use wavecraft_dsp::builtins::{GainDsp, PassthroughDsp};

// Re-export Wavecraft protocol types
pub use wavecraft_protocol::{ParamId, ParameterInfo, ParameterType, db_to_linear};

// Re-export metering types
pub use wavecraft_metering::{MeterConsumer, MeterFrame, MeterProducer, create_meter_channel};

// Re-export ProcessorParams derive macro
pub use wavecraft_macros::ProcessorParams as DeriveProcessorParams;

// Re-export wavecraft_processor! declarative macro (exported at crate root due to #[macro_export])
pub use crate::wavecraft_processor;
