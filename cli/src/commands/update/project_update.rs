use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Result of project dependency updates.
pub(super) enum ProjectUpdateResult {
    /// Not in a project directory — deps skipped.
    NotInProject,
    /// Project deps updated (may include partial failures).
    Updated { errors: Vec<String> },
}

/// Update project dependencies (Rust crates + npm packages) if in a project directory.
pub(super) fn update_project_deps() -> ProjectUpdateResult {
    let (has_engine, has_ui) = detect_project(Path::new("."));

    if !has_engine && !has_ui {
        println!();
        println!("ℹ️  Not in a Wavecraft plugin project — skipping dependency updates.");
        println!(
            "   Run this command from a project root to also update Rust and npm dependencies."
        );
        return ProjectUpdateResult::NotInProject;
    }

    let mut errors = Vec::new();

    if has_engine {
        println!("📦 Updating Rust dependencies...");
        match update_rust_deps() {
            Ok(()) => println!("✅ Rust dependencies updated"),
            Err(e) => {
                eprintln!("❌ Rust update failed: {}", e);
                errors.push(format!("Rust: {}", e));
            }
        }
    }

    if has_ui {
        println!("📦 Updating npm dependencies...");
        match update_npm_deps() {
            Ok(()) => println!("✅ npm dependencies updated"),
            Err(e) => {
                eprintln!("❌ npm update failed: {}", e);
                errors.push(format!("npm: {}", e));
            }
        }
    }

    ProjectUpdateResult::Updated { errors }
}

/// Detect whether the given directory is a Wavecraft plugin project.
///
/// Returns `(has_engine, has_ui)` based on the presence of marker files
/// (`engine/Cargo.toml` and `ui/package.json`).
fn detect_project(root: &Path) -> (bool, bool) {
    let has_engine = root.join("engine/Cargo.toml").exists();
    let has_ui = root.join("ui/package.json").exists();
    (has_engine, has_ui)
}

fn update_rust_deps() -> Result<()> {
    let status = Command::new("cargo")
        .arg("update")
        .current_dir("engine")
        .status()
        .context("Failed to run 'cargo update'. Is cargo installed?")?;

    if !status.success() {
        bail!("cargo update exited with status {}", status);
    }

    Ok(())
}

fn update_npm_deps() -> Result<()> {
    let status = Command::new("npm")
        .arg("update")
        .current_dir("ui")
        .status()
        .context("Failed to run 'npm update'. Is npm installed?")?;

    if !status.success() {
        bail!("npm update exited with status {}", status);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_project_engine_only() {
        let temp = TempDir::new().unwrap();
        let engine_dir = temp.path().join("engine");
        fs::create_dir(&engine_dir).unwrap();
        fs::write(engine_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let (has_engine, has_ui) = detect_project(temp.path());
        assert!(has_engine);
        assert!(!has_ui);
    }

    #[test]
    fn test_detect_project_ui_only() {
        let temp = TempDir::new().unwrap();
        let ui_dir = temp.path().join("ui");
        fs::create_dir(&ui_dir).unwrap();
        fs::write(ui_dir.join("package.json"), "{}").unwrap();

        let (has_engine, has_ui) = detect_project(temp.path());
        assert!(!has_engine);
        assert!(has_ui);
    }

    #[test]
    fn test_detect_project_both() {
        let temp = TempDir::new().unwrap();

        let engine_dir = temp.path().join("engine");
        fs::create_dir(&engine_dir).unwrap();
        fs::write(engine_dir.join("Cargo.toml"), "[package]").unwrap();

        let ui_dir = temp.path().join("ui");
        fs::create_dir(&ui_dir).unwrap();
        fs::write(ui_dir.join("package.json"), "{}").unwrap();

        let (has_engine, has_ui) = detect_project(temp.path());
        assert!(has_engine);
        assert!(has_ui);
    }

    #[test]
    fn test_detect_project_no_markers() {
        let temp = TempDir::new().unwrap();

        let (has_engine, has_ui) = detect_project(temp.path());
        assert!(!has_engine);
        assert!(!has_ui);
    }
}
