//! Auto-detection of SDK repository for development mode.
//!
//! When the CLI is run via `cargo run` from within the wavecraft monorepo,
//! this module detects the SDK source tree so generated projects can use
//! local path dependencies instead of git tags (which may not exist yet).
//!
//! Detection logic:
//! 1. Check if the running binary is inside a cargo `target/` directory
//!    (indicates `cargo run`, not `cargo install`)
//! 2. Walk up from the binary location to find the monorepo root
//!    (identified by `engine/crates/wavecraft-nih_plug/Cargo.toml`)
//! 3. If found, return the `engine/crates/` path for local path dependencies

use std::path::{Path, PathBuf};

/// Marker file that identifies the wavecraft SDK repository root.
const SDK_MARKER: &str = "engine/crates/wavecraft-nih_plug/Cargo.toml";
const SDK_CRATES_RELATIVE: &str = "engine/crates";

/// Attempts to detect whether the CLI is running from a source checkout
/// of the wavecraft SDK (i.e., via `cargo run`).
///
/// Returns `Some(path)` pointing to the canonicalized `engine/crates/` directory
/// if detected, or `None` if the CLI appears to be installed normally.
pub fn detect_sdk_repo() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;

    // Resolve symlinks to get the real binary location
    let exe_path = exe_path.canonicalize().ok()?;

    // Only proceed if the binary is in a cargo target/ directory
    // (i.e., running via `cargo run`, not `cargo install`)
    if !is_cargo_run_binary(&exe_path) {
        return None;
    }

    // Walk up from the binary to find the monorepo root
    let root = find_monorepo_root(&exe_path)?;
    let crates_dir = join_repo_path(&root, SDK_CRATES_RELATIVE);

    // Final validation: ensure the crates directory actually exists
    if crates_dir.is_dir() {
        crates_dir.canonicalize().ok()
    } else {
        None
    }
}

/// Checks if the binary appears to be running from a cargo build directory.
///
/// When built via `cargo run`, the binary lives under `target/debug/` or
/// `target/release/`. Installed binaries live in `~/.cargo/bin/` or similar.
fn is_cargo_run_binary(exe_path: &Path) -> bool {
    exe_path.components().any(|c| c.as_os_str() == "target")
}

/// Walks up from the given path looking for the wavecraft monorepo root.
///
/// The monorepo root is identified by the presence of the SDK marker file
/// (`engine/crates/wavecraft-nih_plug/Cargo.toml`).
fn find_monorepo_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.parent();
    while let Some(dir) = current {
        if has_sdk_marker(dir) {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn join_repo_path(root: &Path, relative: &str) -> PathBuf {
    root.join(relative)
}

fn has_sdk_marker(dir: &Path) -> bool {
    join_repo_path(dir, SDK_MARKER).is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_cargo_run_binary_debug() {
        let path = Path::new("/Users/dev/wavecraft/cli/target/debug/wavecraft");
        assert!(is_cargo_run_binary(path));
    }

    #[test]
    fn test_is_cargo_run_binary_release() {
        let path = Path::new("/Users/dev/wavecraft/cli/target/release/wavecraft");
        assert!(is_cargo_run_binary(path));
    }

    #[test]
    fn test_is_cargo_run_binary_workspace_target() {
        // When built from workspace root, binary is in <repo>/target/debug/
        let path = Path::new("/Users/dev/wavecraft/target/debug/wavecraft");
        assert!(is_cargo_run_binary(path));
    }

    #[test]
    fn test_is_not_cargo_run_binary_installed() {
        let path = Path::new("/Users/dev/.cargo/bin/wavecraft");
        assert!(!is_cargo_run_binary(path));
    }

    #[test]
    fn test_is_not_cargo_run_binary_usr_local() {
        let path = Path::new("/usr/local/bin/wavecraft");
        assert!(!is_cargo_run_binary(path));
    }

    #[test]
    fn test_find_monorepo_root_found() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Create SDK marker structure
        let marker_dir = root.join("engine/crates/wavecraft-nih_plug");
        fs::create_dir_all(&marker_dir).unwrap();
        fs::write(marker_dir.join("Cargo.toml"), "[package]").unwrap();

        // Simulate binary deep inside the repo
        let binary_path = root.join("cli/target/debug/wavecraft");
        fs::create_dir_all(binary_path.parent().unwrap()).unwrap();
        fs::write(&binary_path, "").unwrap();

        let result = find_monorepo_root(&binary_path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), root);
    }

    #[test]
    fn test_find_monorepo_root_workspace_target() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Create SDK marker structure
        let marker_dir = root.join("engine/crates/wavecraft-nih_plug");
        fs::create_dir_all(&marker_dir).unwrap();
        fs::write(marker_dir.join("Cargo.toml"), "[package]").unwrap();

        // Binary at workspace root target (not cli/target)
        let binary_path = root.join("target/debug/wavecraft");
        fs::create_dir_all(binary_path.parent().unwrap()).unwrap();
        fs::write(&binary_path, "").unwrap();

        let result = find_monorepo_root(&binary_path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), root);
    }

    #[test]
    fn test_find_monorepo_root_not_found() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // No SDK marker — just a random project
        let binary_path = root.join("target/debug/my-app");
        fs::create_dir_all(binary_path.parent().unwrap()).unwrap();
        fs::write(&binary_path, "").unwrap();

        let result = find_monorepo_root(&binary_path);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_monorepo_root_with_home_dir() {
        // Simulates installed binary — no monorepo structure above
        let path = Path::new("/Users/dev/.cargo/bin/wavecraft");
        let result = find_monorepo_root(path);
        assert!(result.is_none());
    }
}
