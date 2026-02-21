use anyhow::{bail, Context, Result};
use console::style;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use crate::project::ProjectMarkers;

/// Options for the `bundle` command.
#[derive(Debug)]
pub struct BundleCommand {
    /// Install generated bundles after build.
    pub install: bool,
}

impl BundleCommand {
    pub fn execute(&self) -> Result<()> {
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let project_root = resolve_project_root(&cwd)?;

        let project = ProjectMarkers::detect(&project_root)
            .context("Unable to validate Wavecraft project context")?;

        if project.sdk_mode {
            bail!(
                "`wavecraft bundle --install` must run from a generated plugin project, not the SDK monorepo root.\n\
                 Current directory: {}\n\
                 Navigate to your generated plugin root and run:\n\
                   wavecraft bundle --install",
                cwd.display()
            );
        }

        let delegated_args = delegated_bundle_args(self.install);
        let delegated_display = delegated_command_display(delegated_args);

        println!(
            "{} Delegating to generated project: {}",
            style("→").cyan(),
            delegated_display
        );
        let delegated_cwd = project_root.join("engine");

        let status = Command::new("cargo")
            .args(delegated_args)
            .current_dir(&delegated_cwd)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("Failed to run delegated command `{}`", delegated_display))?;

        if !status.success() {
            bail!(
                "Delegated command failed: `{}` (exit: {}).",
                delegated_display,
                format_exit_status(status),
            );
        }

        if self.install {
            println!("{} Bundle/install completed", style("✓").green());
        } else {
            println!("{} Bundle completed", style("✓").green());
        }
        Ok(())
    }
}

fn delegated_bundle_args(install: bool) -> &'static [&'static str] {
    if install {
        &["xtask", "bundle", "--install"]
    } else {
        &["xtask", "bundle"]
    }
}

fn delegated_command_display(args: &[&str]) -> String {
    format!("cargo {}", args.join(" "))
}

fn format_exit_status(status: ExitStatus) -> String {
    status
        .code()
        .map_or_else(|| "terminated by signal".to_string(), |value| value.to_string())
}

fn resolve_project_root(start_dir: &Path) -> Result<PathBuf> {
    if let Some(root) = find_wavecraft_project_root(start_dir) {
        return Ok(root);
    }

    bail!(
        "Invalid project context for `wavecraft bundle --install`.\n\
         Current directory: {}\n\
         Expected a Wavecraft plugin project root containing:\n\
           - ui/package.json\n\
           - engine/Cargo.toml\n\
         Recovery:\n\
           1) cd <your-generated-plugin-root>\n\
           2) wavecraft bundle --install",
        start_dir.display()
    );
}

fn find_wavecraft_project_root(start_dir: &Path) -> Option<PathBuf> {
    start_dir
        .ancestors()
        .find(|path| is_wavecraft_project_root(path))
        .map(Path::to_path_buf)
}

fn is_wavecraft_project_root(path: &Path) -> bool {
    has_project_marker(path, "ui/package.json") && has_project_marker(path, "engine/Cargo.toml")
}

fn has_project_marker(path: &Path, marker: &str) -> bool {
    path.join(marker).is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn find_project_root_from_nested_directory() {
        let temp = TempDir::new().expect("temp dir should be created");
        let root = temp.path();

        fs::create_dir_all(root.join("ui")).expect("ui dir");
        fs::create_dir_all(root.join("engine")).expect("engine dir");
        fs::create_dir_all(root.join("ui/src/components")).expect("nested ui dir");
        fs::write(root.join("ui/package.json"), "{}").expect("ui package");
        fs::write(root.join("engine/Cargo.toml"), "[package]\nname='demo'").expect("engine cargo");

        let nested = root.join("ui/src/components");
        let detected = find_wavecraft_project_root(&nested).expect("project root should be found");

        assert_eq!(detected, root);
    }

    #[test]
    fn resolve_project_root_returns_actionable_error_when_missing_markers() {
        let temp = TempDir::new().expect("temp dir should be created");
        let result = resolve_project_root(temp.path());

        assert!(result.is_err());
        let message = result.expect_err("should fail").to_string();
        assert!(message.contains("Invalid project context"));
        assert!(message.contains("wavecraft bundle --install"));
        assert!(message.contains("ui/package.json"));
        assert!(message.contains("engine/Cargo.toml"));
    }
}
