//! Project detection and utilities for Wavecraft plugin projects.

// Internal module boundaries

pub mod detection;
pub mod dylib;
pub mod param_extract;
pub mod ts_codegen;

// Public re-exports
pub use detection::{has_node_modules, ProjectMarkers};
pub use dylib::{find_plugin_dylib, read_engine_package_name, resolve_debug_dir};
