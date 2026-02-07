//! Wavecraft project detection utilities.
//!
//! Determines if the current directory (or a specified path) is a valid
//! Wavecraft plugin project by checking for required structure.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// Markers that identify a Wavecraft plugin project.
#[derive(Debug)]
#[allow(dead_code)] // Fields retained for future use (e.g., version detection)
pub struct ProjectMarkers {
    /// Path to ui/ directory
    pub ui_dir: PathBuf,
    /// Path to engine/ directory
    pub engine_dir: PathBuf,
    /// Path to ui/package.json
    pub ui_package_json: PathBuf,
    /// Path to engine/Cargo.toml
    pub engine_cargo_toml: PathBuf,
}

impl ProjectMarkers {
    /// Detect project markers starting from the given directory.
    ///
    /// Returns `Ok(ProjectMarkers)` if this is a valid Wavecraft project,
    /// or an error describing what's missing.
    pub fn detect(start_dir: &Path) -> Result<Self> {
        let ui_dir = start_dir.join("ui");
        let engine_dir = start_dir.join("engine");
        let ui_package_json = ui_dir.join("package.json");
        let engine_cargo_toml = engine_dir.join("Cargo.toml");

        // Check required directories
        if !ui_dir.is_dir() {
            bail!(
                "Not a Wavecraft project: missing 'ui/' directory.\n\
                 Run this command from a plugin project created with `wavecraft create`."
            );
        }

        if !engine_dir.is_dir() {
            bail!(
                "Not a Wavecraft project: missing 'engine/' directory.\n\
                 Run this command from a plugin project created with `wavecraft create`."
            );
        }

        // Check required files
        if !ui_package_json.is_file() {
            bail!("Invalid project structure: missing 'ui/package.json'");
        }

        if !engine_cargo_toml.is_file() {
            bail!("Invalid project structure: missing 'engine/Cargo.toml'");
        }

        Ok(Self {
            ui_dir,
            engine_dir,
            ui_package_json,
            engine_cargo_toml,
        })
    }
}

/// Check if UI dependencies are installed.
pub fn has_node_modules(project: &ProjectMarkers) -> bool {
    project.ui_dir.join("node_modules").is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_project_detection_valid() {
        let tmp = TempDir::new().unwrap();

        // Create valid project structure
        fs::create_dir_all(tmp.path().join("ui")).unwrap();
        fs::create_dir_all(tmp.path().join("engine")).unwrap();
        fs::write(tmp.path().join("ui/package.json"), "{}").unwrap();
        fs::write(tmp.path().join("engine/Cargo.toml"), "[package]").unwrap();

        let result = ProjectMarkers::detect(tmp.path());
        assert!(result.is_ok());

        let markers = result.unwrap();
        assert_eq!(markers.ui_dir, tmp.path().join("ui"));
        assert_eq!(markers.engine_dir, tmp.path().join("engine"));
    }

    #[test]
    fn test_project_detection_missing_ui() {
        let tmp = TempDir::new().unwrap();

        // Only create engine/
        fs::create_dir_all(tmp.path().join("engine")).unwrap();
        fs::write(tmp.path().join("engine/Cargo.toml"), "[package]").unwrap();

        let result = ProjectMarkers::detect(tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing 'ui/'"));
    }

    #[test]
    fn test_project_detection_missing_engine() {
        let tmp = TempDir::new().unwrap();

        // Only create ui/
        fs::create_dir_all(tmp.path().join("ui")).unwrap();
        fs::write(tmp.path().join("ui/package.json"), "{}").unwrap();

        let result = ProjectMarkers::detect(tmp.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing 'engine/'"));
    }

    #[test]
    fn test_project_detection_missing_package_json() {
        let tmp = TempDir::new().unwrap();

        // Create directories but skip package.json
        fs::create_dir_all(tmp.path().join("ui")).unwrap();
        fs::create_dir_all(tmp.path().join("engine")).unwrap();
        fs::write(tmp.path().join("engine/Cargo.toml"), "[package]").unwrap();

        let result = ProjectMarkers::detect(tmp.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing 'ui/package.json'"));
    }

    #[test]
    fn test_has_node_modules_false() {
        let tmp = TempDir::new().unwrap();

        // Create valid project without node_modules
        fs::create_dir_all(tmp.path().join("ui")).unwrap();
        fs::create_dir_all(tmp.path().join("engine")).unwrap();
        fs::write(tmp.path().join("ui/package.json"), "{}").unwrap();
        fs::write(tmp.path().join("engine/Cargo.toml"), "[package]").unwrap();

        let project = ProjectMarkers::detect(tmp.path()).unwrap();
        assert!(!has_node_modules(&project));
    }

    #[test]
    fn test_has_node_modules_true() {
        let tmp = TempDir::new().unwrap();

        // Create valid project with node_modules
        fs::create_dir_all(tmp.path().join("ui/node_modules")).unwrap();
        fs::create_dir_all(tmp.path().join("engine")).unwrap();
        fs::write(tmp.path().join("ui/package.json"), "{}").unwrap();
        fs::write(tmp.path().join("engine/Cargo.toml"), "[package]").unwrap();

        let project = ProjectMarkers::detect(tmp.path()).unwrap();
        assert!(has_node_modules(&project));
    }
}
