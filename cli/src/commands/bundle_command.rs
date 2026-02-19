use anyhow::{bail, Context, Result};
use console::style;

use crate::project::{read_engine_package_name, ProjectMarkers};

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

impl BundleCommand {
    pub fn execute(&self) -> Result<()> {
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let project_root = project_root::resolve_project_root(&cwd, self.install)?;

        let project = ProjectMarkers::detect(&project_root)
            .context("Unable to validate Wavecraft project context")?;

        if project.sdk_mode {
            bail!(
                "`wavecraft bundle` must run from a generated plugin project, not the SDK monorepo root.\n\
                 Current directory: {}\n\
                 Navigate to your generated plugin root and run:\n\
                   wavecraft bundle{}",
                cwd.display(),
                if self.install { " --install" } else { "" }
            );
        }

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
