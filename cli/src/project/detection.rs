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
    /// Path to engine/ directory (in SDK mode: points to sdk-template/engine)
    pub engine_dir: PathBuf,
    /// Path to ui/package.json
    pub ui_package_json: PathBuf,
    /// Path to engine/Cargo.toml (in SDK mode: sdk-template/engine/Cargo.toml)
    pub engine_cargo_toml: PathBuf,
    /// True when running from SDK repo, false for normal plugin projects
    pub sdk_mode: bool,
}

impl ProjectMarkers {
    /// Detect project markers starting from the given directory.
    ///
    /// Returns `Ok(ProjectMarkers)` if this is a valid Wavecraft project,
    /// or an error describing what's missing.
    ///
    /// This also checks if we're in the SDK repo itself (library crates only,
    /// no plugin implementation) and returns an appropriate error.
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

        // Check if this is the SDK workspace (has [workspace] in engine/Cargo.toml)
        if is_sdk_repo(&engine_cargo_toml)? {
            // SDK mode: redirect to canonical sdk-template project
            let template_dir = start_dir.join("sdk-template");
            let template_engine = template_dir.join("engine");
            let template_engine_cargo = template_engine.join("Cargo.toml");
            let template_ui = template_dir.join("ui");
            let template_ui_package_json = template_ui.join("package.json");

            if !template_dir.is_dir() {
                bail!(
                    "SDK mode detected but 'sdk-template/' directory is missing.\n\
                     This is required to run the dev server from the SDK repo."
                );
            }

            if !template_engine_cargo.is_file() {
                bail!(
                    "SDK mode detected but 'sdk-template/engine/Cargo.toml' not found.\n\
                     Run the setup script first:\n\n\
                     	./scripts/setup-dev-template.sh\n\n\
                     This processes .template files for local development."
                );
            }

            if !template_ui_package_json.is_file() {
                bail!(
                    "SDK mode detected but 'sdk-template/ui/package.json' is missing.\n\
                     Ensure the sdk-template/ directory is complete."
                );
            }

            return Ok(Self {
                ui_dir: template_ui,
                engine_dir: template_engine,
                ui_package_json: template_ui_package_json,
                engine_cargo_toml: template_engine_cargo,
                sdk_mode: true,
            });
        }

        Ok(Self {
            ui_dir,
            engine_dir,
            ui_package_json,
            engine_cargo_toml,
            sdk_mode: false,
        })
    }
}

/// Check if the given Cargo.toml defines a workspace (SDK repo) vs a package (plugin project).
///
/// Uses a combination of checks to robustly identify the SDK repo:
/// 1. Parse the Cargo.toml to confirm it's a valid workspace
/// 2. Check for SDK-specific markers (cli/Cargo.toml and engine/crates/wavecraft-core)
fn is_sdk_repo(cargo_toml_path: &Path) -> Result<bool> {
    // Parse TOML to check for [workspace] table
    let content = std::fs::read_to_string(cargo_toml_path)?;
    let parsed: toml::Value = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse Cargo.toml: {}", e))?;

    // Must have a [workspace] table
    if parsed.get("workspace").is_none() {
        return Ok(false);
    }

    // Additional check: confirm this is the Wavecraft SDK by looking for marker files
    let repo_root = cargo_toml_path
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("Invalid cargo_toml_path"))?;

    let has_cli = repo_root.join("cli").join("Cargo.toml").is_file();
    let has_core_crate = repo_root
        .join("engine")
        .join("crates")
        .join("wavecraft-core")
        .is_dir();

    Ok(has_cli && has_core_crate)
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

    #[test]
    fn test_sdk_repo_detection() {
        let tmp = TempDir::new().unwrap();

        // Create SDK-like structure with [workspace] in Cargo.toml + marker files
        fs::create_dir_all(tmp.path().join("ui")).unwrap();
        fs::create_dir_all(tmp.path().join("sdk-template/engine/src")).unwrap();
        fs::create_dir_all(tmp.path().join("sdk-template/ui")).unwrap();
        fs::create_dir_all(tmp.path().join("engine/crates/wavecraft-core/src")).unwrap();
        fs::create_dir_all(tmp.path().join("cli")).unwrap();

        fs::write(tmp.path().join("ui/package.json"), "{}").unwrap();
        fs::write(
            tmp.path().join("engine/Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]",
        )
        .unwrap();

        // Create SDK marker files
        fs::write(
            tmp.path().join("cli/Cargo.toml"),
            "[package]\nname = \"wavecraft\"",
        )
        .unwrap();
        fs::write(
            tmp.path().join("engine/crates/wavecraft-core/Cargo.toml"),
            "[package]\nname = \"wavecraft-core\"",
        )
        .unwrap();

        fs::write(
            tmp.path().join("sdk-template/engine/Cargo.toml"),
            "[package]\nname = \"wavecraft-dev-template\"",
        )
        .unwrap();
        fs::write(tmp.path().join("sdk-template/ui/package.json"), "{}").unwrap();

        let result = ProjectMarkers::detect(tmp.path());
        assert!(result.is_ok());

        let markers = result.unwrap();
        assert!(markers.sdk_mode);
        assert_eq!(markers.engine_dir, tmp.path().join("sdk-template/engine"));
        assert_eq!(
            markers.engine_cargo_toml,
            tmp.path().join("sdk-template/engine/Cargo.toml")
        );
        assert_eq!(markers.ui_dir, tmp.path().join("sdk-template/ui"));
        assert_eq!(
            markers.ui_package_json,
            tmp.path().join("sdk-template/ui/package.json")
        );
    }

    #[test]
    fn test_sdk_mode_missing_template_engine_manifest() {
        let tmp = TempDir::new().unwrap();

        // Create SDK structure WITH marker files but WITHOUT processed sdk-template engine Cargo.toml
        fs::create_dir_all(tmp.path().join("ui")).unwrap();
        fs::create_dir_all(tmp.path().join("engine")).unwrap();
        fs::create_dir_all(tmp.path().join("engine/crates/wavecraft-core")).unwrap();
        fs::create_dir_all(tmp.path().join("cli")).unwrap();
        fs::create_dir_all(tmp.path().join("sdk-template/engine")).unwrap();
        fs::create_dir_all(tmp.path().join("sdk-template/ui")).unwrap();

        fs::write(tmp.path().join("ui/package.json"), "{}").unwrap();
        fs::write(tmp.path().join("sdk-template/ui/package.json"), "{}").unwrap();
        fs::write(
            tmp.path().join("engine/Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]",
        )
        .unwrap();

        // Create SDK marker files
        fs::write(
            tmp.path().join("cli/Cargo.toml"),
            "[package]\nname = \"wavecraft\"",
        )
        .unwrap();
        fs::write(
            tmp.path().join("engine/crates/wavecraft-core/Cargo.toml"),
            "[package]\nname = \"wavecraft-core\"",
        )
        .unwrap();

        // Note: NOT creating sdk-template/engine/Cargo.toml

        let result = ProjectMarkers::detect(tmp.path());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("setup-dev-template.sh"));
    }

    #[test]
    fn test_plugin_project_detection() {
        let tmp = TempDir::new().unwrap();

        // Create plugin project structure with [package] in Cargo.toml
        fs::create_dir_all(tmp.path().join("ui")).unwrap();
        fs::create_dir_all(tmp.path().join("engine")).unwrap();
        fs::write(tmp.path().join("ui/package.json"), "{}").unwrap();
        fs::write(
            tmp.path().join("engine/Cargo.toml"),
            "[package]\nname = \"test_plugin\"",
        )
        .unwrap();

        let result = ProjectMarkers::detect(tmp.path());
        assert!(result.is_ok());

        let markers = result.unwrap();
        assert!(!markers.sdk_mode);
    }
}
