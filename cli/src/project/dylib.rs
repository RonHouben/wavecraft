//! Dylib discovery utilities for finding built plugin libraries.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Find the plugin dylib in the engine's target directory.
///
/// This function handles:
/// - Platform-specific extensions (.dylib, .so, .dll)
/// - Crate name matching (lib[crate_name].ext)
/// - Workspace vs project-local target directories
/// - Multiple candidates (picks most recent)
pub fn find_plugin_dylib(engine_dir: &Path) -> Result<PathBuf> {
    let debug_dir = resolve_debug_dir(engine_dir)?;

    // Look for library files with platform-specific extensions
    #[cfg(target_os = "macos")]
    let extension = "dylib";
    #[cfg(target_os = "linux")]
    let extension = "so";
    #[cfg(target_os = "windows")]
    let extension = "dll";

    // Find library files (skip deps/ subdirectory)
    let entries = std::fs::read_dir(&debug_dir).context("Failed to read debug directory")?;

    let candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|ext| ext == extension)
                && p.file_name().is_some_and(|n| {
                    let name = n.to_string_lossy();
                    if cfg!(target_os = "windows") {
                        !name.starts_with("lib")
                    } else {
                        name.starts_with("lib")
                    }
                })
        })
        .collect();

    if candidates.is_empty() {
        anyhow::bail!(
            "No plugin library found in {}.\n\
             Make sure the engine crate has `crate-type = [\"cdylib\"]` in Cargo.toml.",
            debug_dir.display()
        );
    }

    // Prefer the dylib that matches the engine crate name
    if let Some(crate_name) = read_engine_crate_name(engine_dir) {
        let expected_stem = crate_name.replace('-', "_");
        if let Some(matched) = candidates
            .iter()
            .find(|p| library_matches_name(p, &expected_stem, extension))
        {
            return Ok(matched.to_path_buf());
        }
    }

    if candidates.len() == 1 {
        return Ok(candidates.into_iter().next().unwrap());
    }

    // Multiple libraries - pick the one most recently modified
    let mut sorted = candidates;
    sorted.sort_by_key(|p| {
        std::fs::metadata(p)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    Ok(sorted.pop().unwrap())
}

/// Resolve the debug directory, checking both project-local and workspace locations.
pub fn resolve_debug_dir(engine_dir: &Path) -> Result<PathBuf> {
    let engine_debug = engine_dir.join("target").join("debug");
    if engine_debug.exists() {
        return Ok(engine_debug);
    }

    let workspace_debug = engine_dir.parent().map(|p| p.join("target").join("debug"));

    if let Some(debug_dir) = workspace_debug {
        if debug_dir.exists() {
            return Ok(debug_dir);
        }
    }

    anyhow::bail!(
        "Build output directory not found. Tried:\n  - {}\n  - {}\n\
         Run `cargo build` first.",
        engine_debug.display(),
        engine_dir
            .parent()
            .map(|p| p.join("target").join("debug").display().to_string())
            .unwrap_or_else(|| "<workspace root unavailable>".to_string())
    );
}

/// Read the package name from the engine's Cargo.toml.
///
/// Returns the `[package] name` field, which is used for `--package` flags
/// in Cargo commands to target the correct crate when building with features.
pub fn read_engine_package_name(engine_dir: &Path) -> Option<String> {
    let cargo_toml_path = engine_dir.join("Cargo.toml");
    let contents = std::fs::read_to_string(cargo_toml_path).ok()?;
    let manifest: toml::Value = toml::from_str(&contents).ok()?;

    manifest
        .get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(|name| name.as_str())
        .map(|name| name.to_string())
}

/// Read the crate name from the engine's Cargo.toml.
fn read_engine_crate_name(engine_dir: &Path) -> Option<String> {
    let cargo_toml_path = engine_dir.join("Cargo.toml");
    let contents = std::fs::read_to_string(cargo_toml_path).ok()?;
    let manifest: toml::Value = toml::from_str(&contents).ok()?;

    let lib_name = manifest
        .get("lib")
        .and_then(|lib| lib.get("name"))
        .and_then(|name| name.as_str())
        .map(|name| name.to_string());

    if lib_name.is_some() {
        return lib_name;
    }

    manifest
        .get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(|name| name.as_str())
        .map(|name| name.to_string())
}

/// Check if a library path matches the expected crate name.
fn library_matches_name(path: &Path, expected_stem: &str, extension: &str) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    if cfg!(target_os = "windows") {
        file_name.eq_ignore_ascii_case(&format!("{}.{}", expected_stem, extension))
    } else {
        file_name.eq_ignore_ascii_case(&format!("lib{}.{}", expected_stem, extension))
    }
}
