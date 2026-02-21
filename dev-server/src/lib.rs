//! Wavecraft unified development server
//!
//! This crate provides the complete development server infrastructure for
//! `wavecraft start`, including:
//!
//! - **WebSocket server** (`ws`) — IPC bridge between browser UI and Rust engine
//! - **Audio processing** (`audio`) — Optional real-time audio via OS devices
//! - **Hot-reload** (`reload`) — File watching, rebuild pipeline, and parameter reload
//! - **Parameter hosting** (`host`) — In-memory parameter storage with atomic audio bridge
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────┐
//! │   CLI (wavecraft start) │
//! │   thin wrapper          │
//! └───────────┬────────────┘
//!             │
//!             ▼
//! ┌────────────────────────────────────────────┐
//! │          dev-server crate                   │
//! │                                             │
//! │  ┌──────────┐  ┌──────────┐  ┌───────────┐ │
//! │  │ WsServer │  │ Audio    │  │ Reload    │ │
//! │  │ (ws/)    │  │ (audio/) │  │ (reload/) │ │
//! │  └────┬─────┘  └────┬─────┘  └─────┬─────┘ │
//! │       │              │              │       │
//! │       └──────┬───────┴──────────────┘       │
//! │              ▼                               │
//! │     ┌─────────────────┐                      │
//! │     │ DevServerHost   │                      │
//! │     │ (host.rs)       │                      │
//! │     └─────────────────┘                      │
//! └────────────────────────────────────────────┘
//! ```

pub mod host;
pub mod reload;
pub mod session;
pub mod ws;

#[cfg(feature = "audio")]
pub mod audio;

// Core server orchestration exports.
pub use host::DevServerHost;
pub use reload::guard::BuildGuard;
pub use reload::rebuild::{RebuildCallbacks, RebuildPipeline};
pub use reload::watcher::{FileWatcher, WatchEvent};
pub use session::DevSession;
pub use ws::{WsHandle, WsServer};

// Optional audio-runtime exports (enabled by `audio` feature).
#[cfg(feature = "audio")]
pub use audio::{
    atomic_params::AtomicParameterBridge,
    ffi_processor::{DevAudioProcessor, FfiProcessor},
    server::{AudioConfig, AudioHandle, AudioServer},
    status::{status as audio_status, status_with_diagnostic as audio_status_with_diagnostic},
};
