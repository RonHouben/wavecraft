//! Prelude module - Common imports for Wavecraft SDK users.
//!
//! This module re-exports the most commonly used types and traits for building
//! plugins with Wavecraft. Import this to get started quickly:
//!
//! ```rust
//! use wavecraft_core::prelude::*;
//! ```

// Re-export nih-plug essentials (everything from nih_plug::prelude)
pub use nih_plug::prelude::*;

// Re-export nih-plug export macros (needed at crate root)
pub use nih_plug::{nih_export_clap, nih_export_vst3};

// Re-export Wavecraft DSP traits and types
pub use wavecraft_dsp::{
    Chain, ParamRange, ParamSpec, Processor, ProcessorParams, Transport,
};

// Re-export built-in processors
pub use wavecraft_dsp::builtins::{GainDsp, PassthroughDsp};

// Re-export Wavecraft protocol types
pub use wavecraft_protocol::{ParamId, ParameterInfo, ParameterType, db_to_linear};

// Re-export metering
pub use wavecraft_metering::{MeterConsumer, MeterFrame, MeterProducer, create_meter_channel};

// Re-export editor (platform-specific)
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use crate::editor::WavecraftEditor;

// Re-export utility functions
pub use crate::util::calculate_stereo_meters;

// Re-export DSL macros
pub use wavecraft_macros::{wavecraft_plugin, ProcessorParams as DeriveProcessorParams};

// Re-export wavecraft_processor! declarative macro (exported at crate root due to #[macro_export])
pub use crate::wavecraft_processor;
