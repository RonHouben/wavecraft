//! Wavecraft dev server library
//!
//! Exports modules for testing and integration.

pub mod app;
pub mod assets;
pub mod webview;
pub mod ws_server;

#[cfg(feature = "audio")]
pub mod audio_server;

pub use app::AppState;
