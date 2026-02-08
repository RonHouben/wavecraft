use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Update all project dependencies (Rust crates + npm packages).
pub fn run() -> Result<()> {
    // Detect workspace structure
    let has_engine = Path::new("engine/Cargo.toml").exists();
    let has_ui = Path::new("ui/package.json").exists();

    if !has_engine && !has_ui {
        bail!(
            "Not a Wavecraft plugin project.\n\
             Expected to find 'engine/Cargo.toml' or 'ui/package.json'.\n\
             Run this command from the root of a Wavecraft plugin project."
        );
    }

    let mut errors = Vec::new();

    // Update Rust dependencies
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

    // Update npm dependencies
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

    if errors.is_empty() {
        println!("\n✨ All dependencies updated successfully");
        Ok(())
    } else {
        bail!(
            "Failed to update some dependencies:\n  {}",
            errors.join("\n  ")
        );
    }
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
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detects_engine_only() {
        let temp = TempDir::new().unwrap();
        let engine_dir = temp.path().join("engine");
        fs::create_dir(&engine_dir).unwrap();
        fs::write(engine_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let has_engine = temp.path().join("engine/Cargo.toml").exists();
        let has_ui = temp.path().join("ui/package.json").exists();

        assert!(has_engine);
        assert!(!has_ui);
    }

    #[test]
    fn test_detects_ui_only() {
        let temp = TempDir::new().unwrap();
        let ui_dir = temp.path().join("ui");
        fs::create_dir(&ui_dir).unwrap();
        fs::write(ui_dir.join("package.json"), "{}").unwrap();

        let has_engine = temp.path().join("engine/Cargo.toml").exists();
        let has_ui = temp.path().join("ui/package.json").exists();

        assert!(!has_engine);
        assert!(has_ui);
    }

    #[test]
    fn test_detects_both() {
        let temp = TempDir::new().unwrap();

        let engine_dir = temp.path().join("engine");
        fs::create_dir(&engine_dir).unwrap();
        fs::write(engine_dir.join("Cargo.toml"), "[package]").unwrap();

        let ui_dir = temp.path().join("ui");
        fs::create_dir(&ui_dir).unwrap();
        fs::write(ui_dir.join("package.json"), "{}").unwrap();

        let has_engine = temp.path().join("engine/Cargo.toml").exists();
        let has_ui = temp.path().join("ui/package.json").exists();

        assert!(has_engine);
        assert!(has_ui);
    }
}
