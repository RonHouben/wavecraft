//! Project detection and utilities for Wavecraft plugin projects.

pub mod detection;
pub mod dylib;

pub use detection::{has_node_modules, ProjectMarkers};
pub use dylib::{find_plugin_dylib, read_engine_package_name, resolve_debug_dir};
