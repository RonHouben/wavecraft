//! Convenience re-exports for Wavecraft plugin development.
//!
//! This prelude module provides all commonly needed imports for building
//! Wavecraft audio plugins with nih-plug.
//!
//! # Usage
//!
//! ```rust,ignore
//! use wavecraft::prelude::*;  // via Cargo rename
//! ```

// Re-export nih_plug prelude for direct Plugin trait access
pub use nih_plug::prelude::*;

// Re-export wavecraft-core prelude (DSP types, parameter types)
pub use wavecraft_core::prelude::*;

// Re-export wavecraft-dsp types
pub use wavecraft_dsp::{Processor, ProcessorParams, Transport, builtins};

// Re-export wavecraft-protocol types
pub use wavecraft_protocol::{ParamId, ParameterInfo, ParameterType, db_to_linear};

// Re-export wavecraft-metering types
pub use wavecraft_metering::{MeterConsumer, MeterFrame, MeterProducer, create_meter_channel};

// Re-export editor (platform-dependent)
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use crate::editor::WavecraftEditor;

// Re-export utility functions
pub use crate::util::calculate_stereo_meters;

// Re-export macros
pub use crate::wavecraft_plugin;
pub use crate::wavecraft_processor;
