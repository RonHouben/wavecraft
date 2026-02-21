use anyhow::{bail, Context, Result};
use console::style;
use std::fs;
use std::path::{Path, PathBuf};

use crate::project::{read_engine_package_name, ProjectMarkers};
use crate::template::{extract_template, variables::TemplateVariables};

#[path = "bundle/bundle_runner.rs"]
mod bundle_runner;
#[path = "bundle/engine_build.rs"]
mod engine_build;
#[path = "bundle/install.rs"]
mod install;
#[path = "bundle/metadata_refresh.rs"]
mod metadata_refresh;
#[path = "bundle/project_root.rs"]
mod project_root;
#[path = "bundle/ui_assets.rs"]
mod ui_assets;

/// Options for the `bundle` command.
#[derive(Debug)]
pub struct BundleCommand {
    /// Install generated bundles after build.
    pub install: bool,
}

const TEMP_BUNDLE_PROJECT_NAME: &str = "wavecraft_temp_bundle_plugin";
const TEMP_BUNDLE_DIR_NAME: &str = "wavecraft-temp-bundle-plugin";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BundleExecutionTarget {
    ExistingProject,
    TemporaryGeneratedProject,
}

impl BundleCommand {
    pub fn execute(&self) -> Result<()> {
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let project_root = project_root::resolve_project_root(&cwd, self.install)?;

        let project = ProjectMarkers::detect(&project_root)
            .context("Unable to validate Wavecraft project context")?;

        let execution_target = resolve_execution_target(&project, self.install, &cwd)?;

        match execution_target {
            BundleExecutionTarget::ExistingProject => self.run_pipeline(&project),
            BundleExecutionTarget::TemporaryGeneratedProject => {
                let temp_project_root = ensure_temporary_generated_project(&project_root)?;
                let temp_project =
                    ProjectMarkers::detect(&temp_project_root).with_context(|| {
                        format!(
                            "Failed to validate temporary bundle project at {}",
                            temp_project_root.display()
                        )
                    })?;

                if temp_project.sdk_mode {
                    bail!(
                        "Temporary bundle project resolved to SDK mode unexpectedly: {}",
                        temp_project_root.display()
                    );
                }

                println!(
                    "{} Temporary bundle project source: {}",
                    style("→").cyan(),
                    temp_project_root.display()
                );

                self.run_pipeline(&temp_project)
            }
        }
    }

    fn run_pipeline(&self, project: &ProjectMarkers) -> Result<()> {
        let package_name = read_engine_package_name(&project.engine_dir).ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to read engine package name from {}",
                project.engine_cargo_toml.display()
            )
        })?;

        metadata_refresh::refresh_generated_types(&project, &package_name)?;

        ui_assets::build_ui_assets(&project.ui_dir)?;
        ui_assets::sync_ui_dist_into_wavecraft_nih_plug(
            &project.ui_dir,
            &project.engine_cargo_toml,
            &project.engine_dir,
        )?;

        println!(
            "{} Building plugin package `{}`...",
            style("→").cyan(),
            package_name
        );
        engine_build::build_release_package(&project.engine_dir, &package_name)?;

        println!("{} Bundling plugin artifacts...", style("→").cyan());
        bundle_runner::run_nih_plug_bundle(&project.engine_dir, &package_name)?;

        if self.install {
            install::install_vst3_bundle(&project.engine_dir, &package_name)?;
            println!("{} Bundle/install completed", style("✓").green());
        } else {
            println!("{} Bundle completed", style("✓").green());
        }
        Ok(())
    }
}

fn resolve_execution_target(
    project: &ProjectMarkers,
    install: bool,
    cwd: &Path,
) -> Result<BundleExecutionTarget> {
    if !project.sdk_mode {
        return Ok(BundleExecutionTarget::ExistingProject);
    }

    if install {
        return Ok(BundleExecutionTarget::TemporaryGeneratedProject);
    }

    bail!(
        "`wavecraft bundle` must run from a generated plugin project, not the SDK monorepo root.\n\
         Current directory: {}\n\
         For SDK monorepo workflows, use:\n\
           wavecraft bundle --install\n\
         (this auto-generates/reuses a temporary plugin project under target/tmp)\n\
         Or run from your generated plugin root:\n\
           wavecraft bundle",
        cwd.display(),
    )
}

fn ensure_temporary_generated_project(sdk_root: &Path) -> Result<PathBuf> {
    let temp_project_root = temporary_bundle_project_root(sdk_root);

    if is_generated_plugin_root(&temp_project_root) {
        println!(
            "{} Reusing temporary bundle project at {}",
            style("→").cyan(),
            temp_project_root.display()
        );
        return Ok(temp_project_root);
    }

    if temp_project_root.exists() {
        bail!(
            "Refusing to overwrite existing non-plugin directory at {}.\n\
             Recovery: remove or rename this path, then rerun `wavecraft bundle --install`.",
            temp_project_root.display()
        );
    }

    println!(
        "{} SDK monorepo context detected. Generating temporary plugin project at {}",
        style("→").cyan(),
        temp_project_root.display()
    );

    create_temporary_generated_project(sdk_root, &temp_project_root)?;

    Ok(temp_project_root)
}

fn temporary_bundle_project_root(sdk_root: &Path) -> PathBuf {
    sdk_root
        .join("target")
        .join("tmp")
        .join(TEMP_BUNDLE_DIR_NAME)
}

fn is_generated_plugin_root(path: &Path) -> bool {
    path.join("ui").join("package.json").is_file()
        && path.join("engine").join("Cargo.toml").is_file()
}

fn create_temporary_generated_project(sdk_root: &Path, target_dir: &Path) -> Result<()> {
    let sdk_crates_dir = sdk_root
        .join("engine")
        .join("crates")
        .canonicalize()
        .with_context(|| {
            format!(
                "Failed to resolve local SDK crates path at {}",
                sdk_root.join("engine").join("crates").display()
            )
        })?;

    let vars = TemplateVariables::new(
        TEMP_BUNDLE_PROJECT_NAME.to_string(),
        "Wavecraft SDK".to_string(),
        "sdk@wavecraft.local".to_string(),
        "https://github.com/RonHouben/wavecraft".to_string(),
        format!("wavecraft-cli-v{}", env!("CARGO_PKG_VERSION")),
        Some(sdk_crates_dir),
    );

    if let Some(parent) = target_dir.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create temporary bundle parent directory at {}",
                parent.display()
            )
        })?;
    }

    extract_template(target_dir, &vars).with_context(|| {
        format!(
            "Failed to generate temporary plugin project at {}",
            target_dir.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fake_project(sdk_mode: bool) -> ProjectMarkers {
        let root = PathBuf::from("/tmp/fake-project");
        ProjectMarkers {
            ui_dir: root.join("ui"),
            engine_dir: root.join("engine"),
            ui_package_json: root.join("ui/package.json"),
            engine_cargo_toml: root.join("engine/Cargo.toml"),
            sdk_mode,
        }
    }

    #[test]
    fn routing_uses_existing_project_outside_sdk_mode() {
        let project = fake_project(false);
        let target = resolve_execution_target(&project, false, Path::new("/tmp/fake-project"))
            .expect("routing should succeed");
        assert_eq!(target, BundleExecutionTarget::ExistingProject);
    }

    #[test]
    fn routing_uses_temporary_project_for_sdk_install() {
        let project = fake_project(true);
        let target = resolve_execution_target(&project, true, Path::new("/tmp/fake-sdk"))
            .expect("routing should succeed");
        assert_eq!(target, BundleExecutionTarget::TemporaryGeneratedProject);
    }

    #[test]
    fn routing_rejects_sdk_without_install() {
        let project = fake_project(true);
        let error = resolve_execution_target(&project, false, Path::new("/tmp/fake-sdk"))
            .expect_err("sdk mode without --install should fail");
        let message = error.to_string();
        assert!(message.contains("generated plugin project"));
        assert!(message.contains("wavecraft bundle --install"));
    }

    #[test]
    fn ensure_temporary_generated_project_reuses_existing_plugin_root() {
        let temp = TempDir::new().expect("temp dir");
        let sdk_root = temp.path();
        let project_root = temporary_bundle_project_root(sdk_root);

        std::fs::create_dir_all(project_root.join("ui")).expect("ui dir");
        std::fs::create_dir_all(project_root.join("engine")).expect("engine dir");
        std::fs::write(project_root.join("ui/package.json"), "{}").expect("ui package.json");
        std::fs::write(
            project_root.join("engine/Cargo.toml"),
            "[package]\nname='demo'",
        )
        .expect("engine Cargo.toml");

        let resolved = ensure_temporary_generated_project(sdk_root).expect("should reuse");
        assert_eq!(resolved, project_root);
    }
}
