//! Hot-reload infrastructure for `wavecraft start`
//!
//! This module provides file watching, build guarding, and rebuild pipeline
//! components for hot-reloading Rust plugins during development.

pub mod guard;
pub mod rebuild;
pub mod watcher;

pub use guard::BuildGuard;
pub use rebuild::{RebuildCallbacks, RebuildPipeline};
pub use watcher::{FileWatcher, WatchEvent};
