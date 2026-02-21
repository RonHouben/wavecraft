//! Compatibility shim for legacy `editor/windows.rs` module paths.
//!
//! Canonical Windows editor implementation lives in `editor/windows/mod.rs`.
//! Keep this shim to avoid duplicate logic and module drift.

#[path = "windows/mod.rs"]
mod windows_impl;

pub use windows_impl::*;
