//! Audio processing modules for the development server.
//!
//! This module provides real-time audio capture, FFI processing, and
//! lock-free parameter bridging for the `wavecraft start` dev workflow.
//!
//! All types in this module are feature-gated behind `audio` (enabled by default).

pub mod atomic_params;
pub mod ffi_processor;
pub mod server;

pub use atomic_params::AtomicParameterBridge;
pub use ffi_processor::{DevAudioProcessor, FfiProcessor};
pub use server::{AudioConfig, AudioHandle, AudioServer};
