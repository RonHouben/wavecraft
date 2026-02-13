//! Dylib discovery utilities for finding built plugin libraries.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

fn debug_dir_candidates(engine_dir: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::with_capacity(3);
    dirs.push(engine_dir.join("target").join("debug"));

    if let Some(parent) = engine_dir.parent() {
        dirs.push(parent.join("target").join("debug"));

        if let Some(grand_parent) = parent.parent() {
            dirs.push(grand_parent.join("target").join("debug"));
        }
    }

    dirs
}

/// Find the plugin dylib in the engine's target directory.
///
/// This function handles:
/// - Platform-specific extensions (.dylib, .so, .dll)
/// - Crate name matching (lib[crate_name].ext)
/// - Workspace vs project-local target directories
/// - Multiple candidates (picks most recent)
pub fn find_plugin_dylib(engine_dir: &Path) -> Result<PathBuf> {
    // Look for library files with platform-specific extensions
    #[cfg(target_os = "macos")]
    let extension = "dylib";
    #[cfg(target_os = "linux")]
    let extension = "so";
    #[cfg(target_os = "windows")]
    let extension = "dll";

    let debug_dirs: Vec<PathBuf> = debug_dir_candidates(engine_dir)
        .into_iter()
        .filter(|dir| dir.exists())
        .collect();

    if debug_dirs.is_empty() {
        anyhow::bail!(
            "Build output directory not found. Tried:\n{}\nRun `cargo build` first.",
            debug_dir_candidates(engine_dir)
                .into_iter()
                .map(|p| format!("  - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    for debug_dir in &debug_dirs {
        let entries = std::fs::read_dir(debug_dir)
            .with_context(|| format!("Failed to read debug directory: {}", debug_dir.display()))?;

        candidates.extend(
            entries
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
                }),
        );
    }

    if candidates.is_empty() {
        anyhow::bail!(
            "No plugin library found in any debug directory. Checked:\n{}\n\
             Make sure the engine crate has `crate-type = [\"cdylib\"]` in Cargo.toml.",
            debug_dirs
                .into_iter()
                .map(|p| format!("  - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n")
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
///
/// For SDK mode (engine_dir = sdk-template/engine), checks three levels:
/// 1. sdk-template/engine/target/debug (crate-local)
/// 2. sdk-template/target/debug (one parent up)
/// 3. target/debug (two parents up - repository root)
pub fn resolve_debug_dir(engine_dir: &Path) -> Result<PathBuf> {
    for candidate in debug_dir_candidates(engine_dir) {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    anyhow::bail!(
        "Build output directory not found. Tried:\n{}\nRun `cargo build` first.",
        debug_dir_candidates(engine_dir)
            .into_iter()
            .map(|p| format!("  - {}", p.display()))
            .collect::<Vec<_>>()
            .join("\n")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn platform_extension() -> &'static str {
        #[cfg(target_os = "macos")]
        {
            "dylib"
        }
        #[cfg(target_os = "linux")]
        {
            "so"
        }
        #[cfg(target_os = "windows")]
        {
            "dll"
        }
    }

    fn platform_library_name(stem: &str) -> String {
        if cfg!(target_os = "windows") {
            format!("{}.{}", stem, platform_extension())
        } else {
            format!("lib{}.{}", stem, platform_extension())
        }
    }

    #[test]
    fn finds_plugin_library_in_parent_target_when_engine_target_is_empty() {
        let temp = tempfile::tempdir().expect("temp dir");
        let sdk_template_dir = temp.path().join("sdk-template");
        let engine_dir = sdk_template_dir.join("engine");

        let engine_debug_dir = engine_dir.join("target").join("debug");
        let sdk_debug_dir = sdk_template_dir.join("target").join("debug");

        fs::create_dir_all(&engine_debug_dir).expect("create engine debug dir");
        fs::create_dir_all(&sdk_debug_dir).expect("create sdk debug dir");

        fs::write(
            engine_dir.join("Cargo.toml"),
            "[package]\nname = \"wavecraft-dev-template\"\n[lib]\nname = \"wavecraft_dev_template\"\n",
        )
        .expect("write Cargo.toml");

        let dylib_path = sdk_debug_dir.join(platform_library_name("wavecraft_dev_template"));
        fs::write(&dylib_path, b"test").expect("write dylib placeholder");

        let found = find_plugin_dylib(&engine_dir).expect("should find plugin dylib");
        assert_eq!(found, dylib_path);
    }

    #[test]
    fn prefers_library_matching_crate_name_across_candidate_directories() {
        let temp = tempfile::tempdir().expect("temp dir");
        let sdk_template_dir = temp.path().join("sdk-template");
        let engine_dir = sdk_template_dir.join("engine");

        let engine_debug_dir = engine_dir.join("target").join("debug");
        let sdk_debug_dir = sdk_template_dir.join("target").join("debug");

        fs::create_dir_all(&engine_debug_dir).expect("create engine debug dir");
        fs::create_dir_all(&sdk_debug_dir).expect("create sdk debug dir");

        fs::write(
            engine_dir.join("Cargo.toml"),
            "[package]\nname = \"wavecraft-dev-template\"\n[lib]\nname = \"wavecraft_dev_template\"\n",
        )
        .expect("write Cargo.toml");

        let other_lib = engine_debug_dir.join(platform_library_name("other_plugin"));
        fs::write(&other_lib, b"test").expect("write other dylib placeholder");

        let expected = sdk_debug_dir.join(platform_library_name("wavecraft_dev_template"));
        fs::write(&expected, b"test").expect("write expected dylib placeholder");

        let found = find_plugin_dylib(&engine_dir).expect("should find plugin dylib");
        assert_eq!(found, expected);
    }
}
