//! Prelude module - Common imports for VstKit SDK users.
//!
//! This module re-exports the most commonly used types and traits for building
//! plugins with VstKit. Import this to get started quickly:
//!
//! ```rust
//! use vstkit_core::prelude::*;
//! ```

// Re-export nih-plug essentials (everything from nih_plug::prelude)
pub use nih_plug::prelude::*;

// Re-export VstKit DSP traits
pub use vstkit_dsp::{Processor, Transport};

// Re-export VstKit protocol types
pub use vstkit_protocol::{ParamId, ParameterInfo, ParameterType, db_to_linear};

// Re-export metering
pub use vstkit_metering::{MeterConsumer, MeterFrame, MeterProducer, create_meter_channel};

// Re-export editor (platform-specific)
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use crate::editor::VstKitEditor;

// Re-export utility functions
pub use crate::util::calculate_stereo_meters;
