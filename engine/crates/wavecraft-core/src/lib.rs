//! Wavecraft Core - Audio plugin SDK
//!
//! This crate provides the core Wavecraft SDK for building audio plugins.
//! It re-exports types from the sub-crates and provides the `wavecraft_processor!` macro.
//!
//! For full plugin building with nih-plug integration, use `wavecraft-nih_plug`
//! which depends on this crate and adds the host integration layer.
//!
//! # Quick Start
//!
//! ```text
//! // In your plugin crate, depend on wavecraft-nih_plug:
//! // [dependencies]
//! // wavecraft = { package = "wavecraft-nih_plug", git = "...", tag = "v0.8.0" }
//!
//! use wavecraft::prelude::*;
//!
//! wavecraft_processor!(MyGain => Gain);
//!
//! wavecraft_plugin! {
//!     name: "My Plugin",
//!     vendor: "My Company",
//!     signal: MyGain,
//! }
//! ```

// Public modules
pub mod macros;
pub mod prelude;

// Re-export helper crates used by macros
pub use paste;

// Re-export sub-crates for convenient access
pub use wavecraft_bridge;
pub use wavecraft_dsp;
pub use wavecraft_macros;
pub use wavecraft_metering;
pub use wavecraft_protocol;
