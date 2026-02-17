use anyhow::{bail, Context, Result};
use console::style;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::project::{read_engine_package_name, ProjectMarkers};

/// Options for the `bundle` command.
#[derive(Debug)]
pub struct BundleCommand {
    /// Install generated bundles after build.
    pub install: bool,
}

impl BundleCommand {
    pub fn execute(&self) -> Result<()> {
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let project_root = resolve_project_root(&cwd, self.install)?;

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

        build_ui_assets(&project.ui_dir)?;

        println!(
            "{} Building plugin package `{}`...",
            style("→").cyan(),
            package_name
        );

        let build_status = Command::new("cargo")
            .args(["build", "--release", "-p", package_name.as_str()])
            .current_dir(&project.engine_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("Failed to run `cargo build --release`")?;

        if !build_status.success() {
            let code = build_status.code().map_or_else(
                || "terminated by signal".to_string(),
                |value| value.to_string(),
            );

            bail!("Build failed (exit: {}).", code);
        }

        println!("{} Bundling plugin artifacts...", style("→").cyan());
        run_nih_plug_bundle(&project.engine_dir, &package_name)?;

        if self.install {
            install_vst3_bundle(&project.engine_dir, &package_name)?;
            println!("{} Bundle/install completed", style("✓").green());
        } else {
            println!("{} Bundle completed", style("✓").green());
        }
        Ok(())
    }
}

fn build_ui_assets(ui_dir: &Path) -> Result<()> {
    println!("{} Building UI assets...", style("→").cyan());

    if !ui_dir.join("node_modules").is_dir() {
        let install_status = Command::new("npm")
            .args(["install"])
            .current_dir(ui_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("Failed to run `npm install`. Is npm installed and in your PATH?")?;

        if !install_status.success() {
            let code = install_status.code().map_or_else(
                || "terminated by signal".to_string(),
                |value| value.to_string(),
            );

            bail!("UI dependency install failed (exit: {}).", code);
        }
    }

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(ui_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run `npm run build`. Is npm installed and in your PATH?")?;

    if !status.success() {
        let code = status.code().map_or_else(
            || "terminated by signal".to_string(),
            |value| value.to_string(),
        );

        bail!("UI build failed (exit: {}).", code);
    }

    Ok(())
}

fn run_nih_plug_bundle(engine_dir: &Path, package_name: &str) -> Result<()> {
    let original_cwd = env::current_dir().context("Failed to capture current directory")?;
    let original_manifest_dir = env::var("CARGO_MANIFEST_DIR").ok();

    env::set_current_dir(engine_dir)
        .with_context(|| format!("Failed to enter engine directory: {}", engine_dir.display()))?;

    // `nih_plug_xtask` prefers CARGO_MANIFEST_DIR when set.
    // When this CLI is launched via Cargo, that env var points at this repo's manifest,
    // causing bundling to run against the SDK workspace instead of the generated project.
    // Clear it temporarily so workspace discovery uses the current directory we set above.
    env::remove_var("CARGO_MANIFEST_DIR");

    let args = vec![
        "bundle".to_string(),
        package_name.to_string(),
        "--release".to_string(),
    ];

    let bundle_result = nih_plug_xtask::main_with_args("wavecraft", args)
        .map_err(|error| anyhow::anyhow!("Bundle command failed: {}", error));

    if let Some(value) = original_manifest_dir {
        env::set_var("CARGO_MANIFEST_DIR", value);
    } else {
        env::remove_var("CARGO_MANIFEST_DIR");
    }

    env::set_current_dir(&original_cwd).with_context(|| {
        format!(
            "Failed to restore current directory to {}",
            original_cwd.display()
        )
    })?;

    bundle_result
}

fn bundled_dir_candidates(engine_dir: &Path) -> [PathBuf; 2] {
    [
        engine_dir.join("target").join("bundled"),
        engine_dir
            .parent()
            .unwrap_or(engine_dir)
            .join("target")
            .join("bundled"),
    ]
}

fn resolve_vst3_bundle_path(engine_dir: &Path, package_name: &str) -> Result<PathBuf> {
    for candidate in bundled_dir_candidates(engine_dir) {
        let bundle_path = candidate.join(format!("{}.vst3", package_name));
        if bundle_path.exists() {
            return Ok(bundle_path);
        }
    }

    let candidates = bundled_dir_candidates(engine_dir);
    bail!(
        "Install failed: expected bundled VST3 artifact `{}` was not found in:\n  - {}\n  - {}\nRecovery: run `wavecraft bundle --install` from the plugin project root after bundle succeeds.",
        format_args!("{}.vst3", package_name),
        candidates[0].display(),
        candidates[1].display(),
    );
}

fn macos_vst3_install_dir() -> Result<PathBuf> {
    if !cfg!(target_os = "macos") {
        bail!("Install failed: `wavecraft bundle --install` is currently supported on macOS only.");
    }

    let home = env::var("HOME")
        .context("Install failed: could not resolve HOME environment variable for install path")?;

    Ok(Path::new(&home)
        .join("Library")
        .join("Audio")
        .join("Plug-Ins")
        .join("VST3"))
}

fn install_vst3_bundle(engine_dir: &Path, package_name: &str) -> Result<()> {
    let src = resolve_vst3_bundle_path(engine_dir, package_name)?;
    let dest_dir = macos_vst3_install_dir()?;
    let installed_path = install_vst3_bundle_at_path(&RealFileSystem, &src, &dest_dir)?;

    println!("{} Plugin installed successfully", style("✓").green());
    println!("  Installed VST3: {}", installed_path.display());
    Ok(())
}

fn install_vst3_bundle_at_path(
    fs_ops: &dyn FileSystemOps,
    src: &Path,
    dest_dir: &Path,
) -> Result<PathBuf> {
    if !src.exists() {
        bail!(
            "Install failed: expected bundled VST3 artifact was not found at {}.\nRecovery: run `wavecraft bundle --install` after a successful bundle.",
            src.display()
        );
    }

    fs_ops.create_dir_all(dest_dir).map_err(|error| {
        anyhow::anyhow!(
            "Install failed while creating destination directory {}: {}. Recovery: check filesystem permissions and retry.",
            dest_dir.display(),
            error
        )
    })?;

    let file_name = src.file_name().ok_or_else(|| {
        anyhow::anyhow!("Install failed: invalid VST3 source path {}", src.display())
    })?;
    let dest = dest_dir.join(file_name);

    if dest.exists() {
        fs_ops.remove_dir_all(&dest).map_err(|error| {
            anyhow::anyhow!(
                "Install failed while replacing existing plugin at {}: {}. Recovery: close your DAW, verify write permissions, and retry.",
                dest.display(),
                error
            )
        })?;
    }

    fs_ops.copy_dir_recursive(src, &dest).map_err(|error| {
        anyhow::anyhow!(
            "Install failed while copying VST3 bundle from {} to {}: {}. Recovery: close your DAW, verify write permissions, and retry.",
            src.display(),
            dest.display(),
            error
        )
    })?;

    Ok(dest)
}

trait FileSystemOps {
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn remove_dir_all(&self, path: &Path) -> io::Result<()>;
    fn copy_dir_recursive(&self, src: &Path, dest: &Path) -> io::Result<()>;
}

struct RealFileSystem;

impl FileSystemOps for RealFileSystem {
    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::remove_dir_all(path)
    }

    fn copy_dir_recursive(&self, src: &Path, dest: &Path) -> io::Result<()> {
        copy_dir_recursive_impl(src, dest)
    }
}

fn copy_dir_recursive_impl(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive_impl(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

fn resolve_project_root(start_dir: &Path, install: bool) -> Result<PathBuf> {
    if let Some(root) = find_wavecraft_project_root(start_dir) {
        return Ok(root);
    }

    let command_suffix = if install { " --install" } else { "" };

    bail!(
        "Invalid project context for `wavecraft bundle{}`.\n\
         Current directory: {}\n\
         Expected a Wavecraft plugin project root containing:\n\
           - ui/package.json\n\
           - engine/Cargo.toml\n\
         Recovery:\n\
           1) cd <your-generated-plugin-root>\n\
           2) wavecraft bundle{}",
        command_suffix,
        start_dir.display(),
        command_suffix
    );
}

fn find_wavecraft_project_root(start_dir: &Path) -> Option<PathBuf> {
    start_dir
        .ancestors()
        .find(|path| is_wavecraft_project_root(path))
        .map(Path::to_path_buf)
}

fn is_wavecraft_project_root(path: &Path) -> bool {
    path.join("ui").join("package.json").is_file()
        && path.join("engine").join("Cargo.toml").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::TempDir;

    #[derive(Default)]
    struct MockFileSystem {
        create_dir_all_error: Option<(io::ErrorKind, &'static str)>,
        remove_dir_all_error: Option<(io::ErrorKind, &'static str)>,
        copy_dir_recursive_error: Option<(io::ErrorKind, &'static str)>,
    }

    impl MockFileSystem {
        fn io_error(kind: io::ErrorKind, message: &'static str) -> io::Error {
            io::Error::new(kind, message)
        }
    }

    impl FileSystemOps for MockFileSystem {
        fn create_dir_all(&self, _path: &Path) -> io::Result<()> {
            if let Some((kind, message)) = self.create_dir_all_error {
                return Err(Self::io_error(kind, message));
            }
            Ok(())
        }

        fn remove_dir_all(&self, _path: &Path) -> io::Result<()> {
            if let Some((kind, message)) = self.remove_dir_all_error {
                return Err(Self::io_error(kind, message));
            }
            Ok(())
        }

        fn copy_dir_recursive(&self, _src: &Path, _dest: &Path) -> io::Result<()> {
            if let Some((kind, message)) = self.copy_dir_recursive_error {
                return Err(Self::io_error(kind, message));
            }
            Ok(())
        }
    }

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
        let result = resolve_project_root(temp.path(), false);

        assert!(result.is_err());
        let message = result.expect_err("should fail").to_string();
        assert!(message.contains("Invalid project context"));
        assert!(message.contains("wavecraft bundle`"));
        assert!(!message.contains("wavecraft bundle --install"));
        assert!(message.contains("ui/package.json"));
        assert!(message.contains("engine/Cargo.toml"));
    }

    #[test]
    fn resolve_project_root_install_returns_install_specific_error_when_missing_markers() {
        let temp = TempDir::new().expect("temp dir should be created");
        let result = resolve_project_root(temp.path(), true);

        assert!(result.is_err());
        let message = result.expect_err("should fail").to_string();
        assert!(message.contains("Invalid project context"));
        assert!(message.contains("wavecraft bundle --install"));
        assert!(message.contains("ui/package.json"));
        assert!(message.contains("engine/Cargo.toml"));
    }

    #[test]
    fn install_reports_directory_create_failure_with_diagnostics() {
        let src = create_fake_bundle("demo_plugin");
        let dest_dir = Path::new("/simulated/install/dir");
        let fs_ops = MockFileSystem {
            create_dir_all_error: Some((
                io::ErrorKind::PermissionDenied,
                "permission denied (simulated)",
            )),
            ..Default::default()
        };

        let error =
            install_vst3_bundle_at_path(&fs_ops, &src, dest_dir).expect_err("install should fail");
        let message = error.to_string();

        assert!(message.contains("creating destination directory"));
        assert!(message.contains(&dest_dir.display().to_string()));
        assert!(message.contains("permission denied (simulated)"));
        assert!(message.contains("Recovery: check filesystem permissions and retry."));

        cleanup_temp_path(&src);
    }

    #[test]
    fn install_reports_copy_failure_with_diagnostics() {
        let src = create_fake_bundle("demo_plugin");
        let dest_dir = make_unique_temp_path("vst3-install-dest");
        fs::create_dir_all(&dest_dir).expect("failed to create destination directory fixture");

        let fs_ops = MockFileSystem {
            copy_dir_recursive_error: Some((
                io::ErrorKind::PermissionDenied,
                "resource busy (simulated lock)",
            )),
            ..Default::default()
        };

        let error =
            install_vst3_bundle_at_path(&fs_ops, &src, &dest_dir).expect_err("install should fail");
        let message = error.to_string();

        assert!(message.contains("copying VST3 bundle"));
        assert!(message.contains(&src.display().to_string()));
        assert!(message.contains("resource busy (simulated lock)"));
        assert!(message.contains("Recovery: close your DAW, verify write permissions, and retry."));

        cleanup_temp_path(&src);
        cleanup_temp_path(&dest_dir);
    }

    #[test]
    fn install_reports_replace_failure_with_diagnostics() {
        let package_name = "demo_plugin";
        let src = create_fake_bundle(package_name);
        let dest_dir = make_unique_temp_path("vst3-install-replace");
        let dest_bundle = dest_dir.join(format!("{}.vst3", package_name));
        fs::create_dir_all(&dest_bundle)
            .expect("failed to create existing destination bundle fixture");

        let fs_ops = MockFileSystem {
            remove_dir_all_error: Some((
                io::ErrorKind::PermissionDenied,
                "operation not permitted (simulated lock)",
            )),
            ..Default::default()
        };

        let error =
            install_vst3_bundle_at_path(&fs_ops, &src, &dest_dir).expect_err("install should fail");
        let message = error.to_string();

        assert!(message.contains("replacing existing plugin"));
        assert!(message.contains(&dest_bundle.display().to_string()));
        assert!(message.contains("operation not permitted (simulated lock)"));
        assert!(message.contains("Recovery: close your DAW, verify write permissions, and retry."));

        cleanup_temp_path(&src);
        cleanup_temp_path(&dest_dir);
    }

    #[test]
    fn build_ui_assets_reports_failure_exit_code() {
        let temp = TempDir::new().expect("temp dir should be created");
        let ui_dir = temp.path().join("ui");
        fs::create_dir_all(&ui_dir).expect("ui dir");
        fs::create_dir_all(ui_dir.join("node_modules")).expect("node_modules dir");
        fs::write(
            ui_dir.join("package.json"),
            "{\"name\":\"build-ui-fixture\",\"scripts\":{\"build\":\"exit 1\"}}\n",
        )
        .expect("ui package");

        let error = build_ui_assets(&ui_dir).expect_err("build-ui should fail in fixture");
        assert!(error.to_string().contains("UI build failed"));
    }

    fn create_fake_bundle(package_name: &str) -> PathBuf {
        let bundle_dir =
            make_unique_temp_path("vst3-source-bundle").join(format!("{}.vst3", package_name));
        fs::create_dir_all(bundle_dir.join("Contents"))
            .expect("failed to create fake bundle directories");
        fs::write(
            bundle_dir.join("Contents").join("Info.plist"),
            "<plist><dict><key>CFBundleName</key><string>Test</string></dict></plist>",
        )
        .expect("failed to write fake bundle file");
        bundle_dir
    }

    fn make_unique_temp_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock moved backwards")
            .as_nanos();
        std::env::temp_dir().join(format!("{}-{}", prefix, nanos))
    }

    fn cleanup_temp_path(path: &Path) {
        let target = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };

        if target.exists() {
            let _ = fs::remove_dir_all(target);
        }
    }
}
