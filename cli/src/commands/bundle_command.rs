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
        sync_ui_dist_into_wavecraft_nih_plug(
            &project.ui_dir,
            &project.engine_cargo_toml,
            &project.engine_dir,
        )?;

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

fn sync_ui_dist_into_wavecraft_nih_plug(
    ui_dir: &Path,
    engine_cargo_toml: &Path,
    engine_dir: &Path,
) -> Result<()> {
    let ui_dist = ui_dir.join("dist");
    if !ui_dist.is_dir() {
        bail!(
            "UI build completed but expected output directory was not found: {}\nRecovery: ensure your UI build generates `ui/dist` before running `wavecraft bundle`.",
            ui_dist.display()
        );
    }

    let wavecraft_nih_plug_dir = resolve_wavecraft_nih_plug_path(engine_cargo_toml)?;
    let assets_dir = wavecraft_nih_plug_dir.join("assets").join("ui-dist");

    stage_ui_dist(&ui_dist, &assets_dir)?;

    println!(
        "{} Staged UI dist into {}",
        style("→").cyan(),
        assets_dir.display()
    );

    clean_wavecraft_nih_plug(engine_dir)?;
    Ok(())
}

fn resolve_wavecraft_nih_plug_path(engine_cargo_toml: &Path) -> Result<PathBuf> {
    let contents = fs::read_to_string(engine_cargo_toml).with_context(|| {
        format!(
            "Failed to read engine manifest while resolving Wavecraft dependency: {}",
            engine_cargo_toml.display()
        )
    })?;

    let manifest: toml::Value = toml::from_str(&contents).with_context(|| {
        format!(
            "Failed to parse engine manifest as TOML: {}",
            engine_cargo_toml.display()
        )
    })?;

    let dependencies = manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Engine manifest is missing a `[dependencies]` table: {}",
                engine_cargo_toml.display()
            )
        })?;

    for (dependency_name, dependency_value) in dependencies {
        let table = match dependency_value.as_table() {
            Some(table) => table,
            None => continue,
        };

        let package_name = table
            .get("package")
            .and_then(toml::Value::as_str)
            .unwrap_or(dependency_name.as_str());

        if package_name != "wavecraft-nih_plug" {
            continue;
        }

        let path_value = table.get("path").and_then(toml::Value::as_str);
        let Some(path_value) = path_value else {
            bail!(
                "`wavecraft bundle --install` requires a local SDK path dependency for `wavecraft-nih_plug`, but `{}` in {} does not define `path`.\nRecovery: generate the project in local SDK mode (`wavecraft create --local-sdk`) or update engine/Cargo.toml to use a local `path` dependency for `wavecraft-nih_plug`.",
                dependency_name,
                engine_cargo_toml.display()
            );
        };

        let base_dir = engine_cargo_toml.parent().ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to resolve base directory for engine manifest: {}",
                engine_cargo_toml.display()
            )
        })?;

        let resolved = if Path::new(path_value).is_absolute() {
            PathBuf::from(path_value)
        } else {
            base_dir.join(path_value)
        };

        if !resolved.is_dir() {
            bail!(
                "Resolved `wavecraft-nih_plug` dependency path does not exist or is not a directory: {}\nRecovery: ensure engine/Cargo.toml points to a valid local SDK checkout.",
                resolved.display()
            );
        }

        return Ok(resolved);
    }

    bail!(
        "Unable to find a dependency entry for package `wavecraft-nih_plug` in {}.\nRecovery: generate the project in local SDK mode (`wavecraft create --local-sdk`) or add `wavecraft = {{ package = \"wavecraft-nih_plug\", path = \"/path/to/wavecraft/engine/crates/wavecraft-nih_plug\" }}` to engine/Cargo.toml.",
        engine_cargo_toml.display()
    )
}

fn stage_ui_dist(ui_dist: &Path, assets_dir: &Path) -> Result<()> {
    if assets_dir.exists() {
        fs::remove_dir_all(assets_dir).with_context(|| {
            format!(
                "Failed to clear previous embedded UI assets at {}",
                assets_dir.display()
            )
        })?;
    }

    fs::create_dir_all(assets_dir).with_context(|| {
        format!(
            "Failed to create embedded UI asset directory at {}",
            assets_dir.display()
        )
    })?;

    copy_dir_recursive_impl(ui_dist, assets_dir).with_context(|| {
        format!(
            "Failed to stage UI dist from {} into {}",
            ui_dist.display(),
            assets_dir.display()
        )
    })?;

    Ok(())
}

fn clean_wavecraft_nih_plug(engine_dir: &Path) -> Result<()> {
    println!(
        "{} Cleaning `wavecraft-nih_plug` to refresh embedded assets...",
        style("→").cyan()
    );

    let status = Command::new("cargo")
        .args(["clean", "-p", "wavecraft-nih_plug"])
        .current_dir(engine_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run `cargo clean -p wavecraft-nih_plug`")?;

    if !status.success() {
        let code = status.code().map_or_else(
            || "terminated by signal".to_string(),
            |value| value.to_string(),
        );
        bail!(
            "Failed to clean `wavecraft-nih_plug` before rebuild (exit: {}).",
            code
        );
    }

    Ok(())
}

fn run_nih_plug_bundle(engine_dir: &Path, package_name: &str) -> Result<()> {
    let helper_manifest = ensure_nih_plug_bundle_helper_manifest()?;

    let status = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            helper_manifest.to_string_lossy().as_ref(),
            "--",
            package_name,
            engine_dir.to_string_lossy().as_ref(),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| {
            format!(
                "Failed to run CLI bundle helper at {}",
                helper_manifest.display()
            )
        })?;

    if status.success() {
        Ok(())
    } else {
        let code = status.code().map_or_else(
            || "terminated by signal".to_string(),
            |value| value.to_string(),
        );
        bail!("Bundle command failed (exit: {}).", code);
    }
}

fn ensure_nih_plug_bundle_helper_manifest() -> Result<PathBuf> {
    let helper_root = std::env::temp_dir().join("wavecraft-nih-plug-bundle-helper");
    let helper_src_dir = helper_root.join("src");
    fs::create_dir_all(&helper_src_dir).with_context(|| {
        format!(
            "Failed to create CLI bundle helper directory at {}",
            helper_src_dir.display()
        )
    })?;

    let helper_manifest = helper_root.join("Cargo.toml");
    let helper_main = helper_src_dir.join("main.rs");

    fs::write(
        &helper_manifest,
        "[package]\nname = \"wavecraft_nih_plug_bundle_helper\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nanyhow = \"1.0\"\nnih_plug_xtask = { git = \"https://github.com/robbert-vdh/nih-plug.git\", rev = \"28b149ec4d62757d0b448809148a0c3ca6e09a95\" }\n",
    )
    .with_context(|| {
        format!(
            "Failed to write CLI bundle helper manifest at {}",
            helper_manifest.display()
        )
    })?;

    fs::write(
        &helper_main,
        "use anyhow::{Context, Result};\nuse std::env;\nuse std::path::PathBuf;\n\nfn main() -> Result<()> {\n    let mut args = env::args().skip(1);\n    let package_name = args.next().context(\"Missing package name argument\")?;\n    let engine_dir = PathBuf::from(args.next().context(\"Missing engine directory argument\")?);\n\n    let original_cwd = env::current_dir().context(\"Failed to capture current directory\")?;\n    let original_manifest_dir = env::var(\"CARGO_MANIFEST_DIR\").ok();\n\n    env::set_current_dir(&engine_dir)\n        .with_context(|| format!(\"Failed to enter engine directory: {}\", engine_dir.display()))?;\n\n    env::remove_var(\"CARGO_MANIFEST_DIR\");\n\n    let bundle_result = nih_plug_xtask::main_with_args(\n        \"wavecraft\",\n        vec![\"bundle\".to_string(), package_name, \"--release\".to_string()],\n    )\n    .map_err(|error| anyhow::anyhow!(\"Bundle command failed: {}\", error));\n\n    if let Some(value) = original_manifest_dir {\n        env::set_var(\"CARGO_MANIFEST_DIR\", value);\n    } else {\n        env::remove_var(\"CARGO_MANIFEST_DIR\");\n    }\n\n    env::set_current_dir(&original_cwd).with_context(|| {\n        format!(\"Failed to restore current directory to {}\", original_cwd.display())\n    })?;\n\n    bundle_result\n}\n",
    )
    .with_context(|| {
        format!(
            "Failed to write CLI bundle helper source at {}",
            helper_main.display()
        )
    })?;

    Ok(helper_manifest)
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

    #[test]
    fn resolve_wavecraft_nih_plug_path_accepts_local_path_dependency() {
        let temp = TempDir::new().expect("temp dir should be created");
        let engine_dir = temp.path().join("engine");
        let dep_dir = temp.path().join("wavecraft-nih-plug");

        fs::create_dir_all(&engine_dir).expect("engine dir");
        fs::create_dir_all(&dep_dir).expect("dep dir");

        fs::write(
            engine_dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n\n[dependencies]\nwavecraft = {{ package = \"wavecraft-nih_plug\", path = \"{}\" }}\n",
                dep_dir.display()
            ),
        )
        .expect("engine cargo");

        let resolved = resolve_wavecraft_nih_plug_path(&engine_dir.join("Cargo.toml"))
            .expect("path should resolve");
        assert_eq!(resolved, dep_dir);
    }

    #[test]
    fn resolve_wavecraft_nih_plug_path_rejects_non_path_dependency() {
        let temp = TempDir::new().expect("temp dir should be created");
        let engine_dir = temp.path().join("engine");
        fs::create_dir_all(&engine_dir).expect("engine dir");

        fs::write(
            engine_dir.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n\n[dependencies]\nwavecraft = { package = \"wavecraft-nih_plug\", git = \"https://example.com/repo.git\" }\n",
        )
        .expect("engine cargo");

        let error = resolve_wavecraft_nih_plug_path(&engine_dir.join("Cargo.toml"))
            .expect_err("non-path dependency should fail");
        let message = error.to_string();
        assert!(message.contains("requires a local SDK path dependency"));
        assert!(message.contains("wavecraft create --local-sdk"));
    }

    #[test]
    fn stage_ui_dist_replaces_previous_assets() {
        let temp = TempDir::new().expect("temp dir should be created");
        let ui_dist = temp.path().join("ui/dist");
        let assets_dir = temp.path().join("wavecraft-nih-plug/assets/ui-dist");

        fs::create_dir_all(ui_dist.join("assets")).expect("ui dist assets");
        fs::write(ui_dist.join("index.html"), "<html>generated</html>").expect("index");
        fs::write(ui_dist.join("assets/app.js"), "console.log('generated')").expect("asset");

        fs::create_dir_all(&assets_dir).expect("stale assets dir");
        fs::write(assets_dir.join("old.txt"), "stale").expect("stale file");

        stage_ui_dist(&ui_dist, &assets_dir).expect("staging should succeed");

        assert!(assets_dir.join("index.html").is_file());
        assert!(assets_dir.join("assets/app.js").is_file());
        assert!(!assets_dir.join("old.txt").exists());
        let index = fs::read_to_string(assets_dir.join("index.html")).expect("read index");
        assert!(index.contains("generated"));
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
