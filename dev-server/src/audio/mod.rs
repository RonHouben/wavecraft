//! Audio processing modules for the development server.
//!
//! This module provides real-time audio capture, FFI processing, and
//! lock-free parameter bridging for the `wavecraft start` dev workflow.
//!
//! All types in this module are feature-gated behind `audio` (enabled by default).

// Internal module boundaries

pub mod atomic_params;
pub mod ffi_processor;
pub mod server;
pub mod status;

// Public re-exports
pub use atomic_params::AtomicParameterBridge;
pub use ffi_processor::{DevAudioProcessor, FfiProcessor};
pub use server::{AudioConfig, AudioHandle, AudioServer};
pub use status::{status, status_with_diagnostic};
