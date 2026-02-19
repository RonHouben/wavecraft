use anyhow::{bail, Context, Result};
use console::style;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use super::install::copy_dir_recursive_impl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum WavecraftNihPlugDependencyMode {
    LocalPath(PathBuf),
    ExternalSource,
}

fn format_exit_code(status: ExitStatus) -> String {
    status.code().map_or_else(
        || "terminated by signal".to_string(),
        |value| value.to_string(),
    )
}

fn ensure_command_success(status: ExitStatus, failure_message: &str) -> Result<()> {
    if status.success() {
        return Ok(());
    }

    bail!("{} (exit: {}).", failure_message, format_exit_code(status));
}

fn resolve_dependency_path(base_dir: &Path, path_value: &str) -> PathBuf {
    if Path::new(path_value).is_absolute() {
        PathBuf::from(path_value)
    } else {
        base_dir.join(path_value)
    }
}

pub(super) fn build_ui_assets(ui_dir: &Path) -> Result<()> {
    println!("{} Building UI assets...", style("→").cyan());

    if !ui_dir.join("node_modules").is_dir() {
        let install_status = Command::new("npm")
            .args(["install"])
            .current_dir(ui_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("Failed to run `npm install`. Is npm installed and in your PATH?")?;

        ensure_command_success(install_status, "UI dependency install failed")?;
    }

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(ui_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run `npm run build`. Is npm installed and in your PATH?")?;

    ensure_command_success(status, "UI build failed")?;

    Ok(())
}

pub(super) fn sync_ui_dist_into_wavecraft_nih_plug(
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

    match detect_wavecraft_nih_plug_dependency_mode(engine_cargo_toml)? {
        WavecraftNihPlugDependencyMode::LocalPath(wavecraft_nih_plug_dir) => {
            let assets_dir = wavecraft_nih_plug_dir.join("assets").join("ui-dist");

            stage_ui_dist(&ui_dist, &assets_dir)?;

            println!(
                "{} Staged UI dist into {}",
                style("→").cyan(),
                assets_dir.display()
            );

            clean_wavecraft_nih_plug(engine_dir)?;
        }
        WavecraftNihPlugDependencyMode::ExternalSource => {
            println!(
                "{} `wavecraft-nih_plug` is not a local path dependency; skipping local UI asset staging.",
                style("→").cyan()
            );
            println!(
                "{} Continuing with bundle using dependency-provided embedded assets.",
                style("→").cyan()
            );
        }
    }

    Ok(())
}

pub(super) fn detect_wavecraft_nih_plug_dependency_mode(
    engine_cargo_toml: &Path,
) -> Result<WavecraftNihPlugDependencyMode> {
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
            return Ok(WavecraftNihPlugDependencyMode::ExternalSource);
        };

        let base_dir = engine_cargo_toml.parent().ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to resolve base directory for engine manifest: {}",
                engine_cargo_toml.display()
            )
        })?;

        let resolved = resolve_dependency_path(base_dir, path_value);

        if !resolved.is_dir() {
            bail!(
                "Resolved `wavecraft-nih_plug` dependency path does not exist or is not a directory: {}\nRecovery: ensure engine/Cargo.toml points to a valid local SDK checkout.",
                resolved.display()
            );
        }

        return Ok(WavecraftNihPlugDependencyMode::LocalPath(resolved));
    }

    bail!(
        "Unable to find a dependency entry for package `wavecraft-nih_plug` in {}.\nRecovery: ensure engine/Cargo.toml includes `wavecraft = {{ package = \"wavecraft-nih_plug\", git = \"https://github.com/RonHouben/wavecraft\", tag = \"<version>\" }}` or a local `path` dependency.",
        engine_cargo_toml.display()
    )
}

pub(super) fn stage_ui_dist(ui_dist: &Path, assets_dir: &Path) -> Result<()> {
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

    ensure_command_success(
        status,
        "Failed to clean `wavecraft-nih_plug` before rebuild",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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
    fn detect_wavecraft_nih_plug_dependency_mode_accepts_local_path_dependency() {
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

        let resolved = detect_wavecraft_nih_plug_dependency_mode(&engine_dir.join("Cargo.toml"))
            .expect("mode should resolve");
        assert_eq!(resolved, WavecraftNihPlugDependencyMode::LocalPath(dep_dir));
    }

    #[test]
    fn detect_wavecraft_nih_plug_dependency_mode_accepts_non_path_dependency() {
        let temp = TempDir::new().expect("temp dir should be created");
        let engine_dir = temp.path().join("engine");
        fs::create_dir_all(&engine_dir).expect("engine dir");

        fs::write(
            engine_dir.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n\n[dependencies]\nwavecraft = { package = \"wavecraft-nih_plug\", git = \"https://example.com/repo.git\" }\n",
        )
        .expect("engine cargo");

        let resolved = detect_wavecraft_nih_plug_dependency_mode(&engine_dir.join("Cargo.toml"))
            .expect("non-path dependency should be supported");
        assert_eq!(resolved, WavecraftNihPlugDependencyMode::ExternalSource);
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
}
