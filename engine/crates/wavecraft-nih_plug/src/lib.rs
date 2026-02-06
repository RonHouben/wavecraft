//! Wavecraft nih-plug integration layer.
//!
//! This crate provides the nih-plug specific integration for Wavecraft plugins.
//! It is used internally and not published to crates.io.
//!
//! Plugin developers using Wavecraft should depend on this crate via git:
//! ```toml
//! [dependencies]
//! wavecraft = { package = "wavecraft-nih_plug", git = "https://github.com/wavecraft-audio/wavecraft", tag = "v0.8.0" }
//! ```

pub mod editor;
pub mod macros;
pub mod prelude;
pub mod util;

// Re-export key types for convenience
pub use wavecraft_core::prelude as core_prelude;
pub use wavecraft_dsp::{ParamRange, Processor, ProcessorParams, Transport};
pub use wavecraft_metering::{MeterConsumer, MeterFrame, MeterProducer, create_meter_channel};
pub use wavecraft_protocol::ParameterInfo;

// Re-export the wavecraft_processor! macro from wavecraft_core
pub use wavecraft_core::wavecraft_processor;

// Re-export the wavecraft_plugin! proc-macro from wavecraft_macros
pub use wavecraft_macros::wavecraft_plugin;

/// Hidden module for macro-generated code.
///
/// This module re-exports nih_plug types needed by the `wavecraft_plugin!` macro.
/// Plugin developers should not use this module directly.
#[doc(hidden)]
pub mod __nih {
    // Re-export all nih_plug prelude types for macro-generated code
    pub use nih_plug::prelude::*;

    // Re-export specific types that macros explicitly reference
    pub use nih_plug::prelude::{
        // Audio processing types
        AsyncExecutor,
        AudioIOLayout,
        AuxiliaryBuffers,
        Buffer,
        BufferConfig,
        // Format-specific types
        ClapFeature,
        // Core plugin traits
        ClapPlugin,
        Editor,
        // Parameter types
        Enum,
        EnumParam,
        FloatParam,
        FloatRange,
        InitContext,
        IntParam,
        IntRange,
        MidiConfig,
        Param,
        ParamPtr,
        Params,
        Plugin,
        ProcessContext,
        ProcessStatus,
        Vst3Plugin,
        Vst3SubCategory,
    };

    // Export macros that are used in generated code
    pub use nih_plug::{nih_export_clap, nih_export_vst3};

    // Re-export the editor module for WavecraftEditor
    pub use super::editor;
}

/// Internal types used by generated code (not part of public API).
///
/// This module provides types and functions needed by FFI exports generated
/// by the `wavecraft_plugin!` macro for parameter discovery via `wavecraft start`.
#[doc(hidden)]
pub mod __internal {
    pub use serde_json;
    pub use wavecraft_protocol::ParameterInfo;
    pub use wavecraft_protocol::ParameterType;

    use wavecraft_dsp::ParamSpec;

    /// Convert ParamSpec to ParameterInfo for JSON serialization.
    ///
    /// This function bridges the DSP layer's ParamSpec to the protocol's
    /// ParameterInfo, enabling FFI export of parameter metadata.
    pub fn param_spec_to_info(spec: &ParamSpec, id_prefix: &str) -> ParameterInfo {
        ParameterInfo {
            id: format!("{}_{}", id_prefix, spec.id_suffix),
            name: spec.name.to_string(),
            param_type: ParameterType::Float,
            value: spec.default as f32,
            default: spec.default as f32,
            unit: if spec.unit.is_empty() {
                None
            } else {
                Some(spec.unit.to_string())
            },
            group: spec.group.map(|s| s.to_string()),
        }
    }
}
