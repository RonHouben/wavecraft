//! Bundle command - Build and bundle VST3/CLAP plugins using nih_plug_xtask.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::build_ui;
use xtask::BuildMode;
use xtask::PLUGIN_NAME;
use xtask::cargo_command;
use xtask::output::*;
use xtask::paths;

/// Run the bundle command.
///
/// This builds the plugin library and packages it into VST3 and CLAP bundles.
pub fn run(mode: BuildMode, package: Option<&str>, verbose: bool) -> Result<()> {
    run_with_features(mode, package, &[], verbose)
}

/// Run the bundle command with specific features.
pub fn run_with_features(
    mode: BuildMode,
    package: Option<&str>,
    features: &[&str],
    verbose: bool,
) -> Result<()> {
    let engine_dir = paths::engine_dir()?;
    let package_name = match resolve_bundle_package(package, &engine_dir) {
        Ok(name) => name,
        Err(err)
            if package.is_none()
                && err
                    .to_string()
                    .contains("No bundleable plugin crate found in engine workspace") =>
        {
            print_warning(
                "No bundleable plugin crate was found in this engine workspace; skipping bundle.",
            );
            print_info(
                "If you're building a generated plugin project, use `wavecraft bundle --install` from that project root.",
            );
            return Ok(());
        }
        Err(err) => return Err(err),
    };

    if package.is_none() {
        print_info(&format!(
            "No --package provided; auto-selected bundleable crate: {}",
            package_name
        ));
    }

    // Build the React UI assets
    print_status("Building React UI assets...");
    build_ui::run(verbose, false)?;

    print_success("React UI built successfully");

    print_status(&format!("Building {} plugin...", package_name));

    // Build the command: cargo build --release -p <package>
    let mut build_cmd = cargo_command();
    build_cmd.current_dir(&engine_dir);
    build_cmd.arg("build");
    build_cmd.arg("-p").arg(&package_name);

    if let Some(flag) = mode.cargo_flag() {
        build_cmd.arg(flag);
    }

    if !features.is_empty() {
        build_cmd.arg("--features").arg(features.join(","));
    }

    if verbose {
        println!(
            "Running: cargo build -p {} {:?} --features {:?}",
            package_name,
            mode.cargo_flag(),
            features
        );
    }

    let build_status = build_cmd.status().context("Failed to run cargo build")?;

    if !build_status.success() {
        anyhow::bail!(
            "Build command failed with exit code: {:?}",
            build_status.code()
        );
    }

    print_status("Bundling plugins...");

    // Use nih_plug_xtask's bundler
    // We need to pass features through to ensure the right binary is bundled
    let mut bundle_args = vec!["bundle".to_string(), package_name.to_string()];
    if let Some(flag) = mode.cargo_flag() {
        bundle_args.push(flag.to_string());
    }

    // Pass features to nih_plug_xtask so it rebuilds with the right features
    if !features.is_empty() {
        bundle_args.push("--features".to_string());
        bundle_args.push(features.join(","));
    }

    if verbose {
        println!("Bundle args: {:?}", bundle_args);
    }

    // Call nih_plug_xtask::main_with_args with features
    // Note: nih_plug_xtask expects the binary name (PLUGIN_NAME), not the package name
    if let Err(e) = nih_plug_xtask::main_with_args(PLUGIN_NAME, bundle_args) {
        anyhow::bail!(
            "Bundle command failed for package '{}': {}\n\nActionable help:\n  - Ensure the package is a bundleable plugin crate with [lib] crate-type including \"cdylib\"\n  - Try: cargo xtask bundle --package <plugin-crate>\n  - In generated projects, prefer: wavecraft bundle",
            package_name,
            e
        );
    }

    // Verify bundles were created.
    // Prefer canonical plugin bundle names first, with package-name fallback.
    let bundled_dir = paths::bundled_dir()?;
    let vst3_bundle = bundled_dir.join(format!("{}.vst3", PLUGIN_NAME));
    let clap_bundle = bundled_dir.join(format!("{}.clap", PLUGIN_NAME));
    let vst3_fallback = bundled_dir.join(format!("{}.vst3", package_name));
    let clap_fallback = bundled_dir.join(format!("{}.clap", package_name));

    let mut found_any_bundle = false;

    if vst3_bundle.exists() {
        print_success_item(&format!("VST3: {}", vst3_bundle.display()));
        found_any_bundle = true;
    } else if vst3_fallback.exists() {
        print_success_item(&format!("VST3: {}", vst3_fallback.display()));
        found_any_bundle = true;
    }

    if clap_bundle.exists() {
        print_success_item(&format!("CLAP: {}", clap_bundle.display()));
        found_any_bundle = true;
    } else if clap_fallback.exists() {
        print_success_item(&format!("CLAP: {}", clap_fallback.display()));
        found_any_bundle = true;
    }

    if !found_any_bundle {
        anyhow::bail!(
            "Bundling completed but no plugin bundles were found in {}.\n\nThis usually means the selected package does not export any nih-plug plugin entry points.\nExpected one of: {}.vst3/.clap or {}.vst3/.clap\n\nActionable help:\n  - Bundle a concrete plugin crate: cargo xtask bundle --package <plugin-crate>\n  - In generated plugin projects, use: wavecraft bundle",
            bundled_dir.display(),
            PLUGIN_NAME,
            package_name
        );
    }

    Ok(())
}

fn resolve_bundle_package(requested_package: Option<&str>, engine_dir: &Path) -> Result<String> {
    if let Some(package) = requested_package {
        return Ok(package.to_string());
    }

    let bundleable_packages = discover_bundleable_packages(engine_dir)?;

    if bundleable_packages.is_empty() {
        anyhow::bail!(
            "No bundleable plugin crate found in engine workspace.\n\nExpected at least one crate under engine/crates/ with [lib] crate-type including \"cdylib\".\n\nActionable help:\n  - Add a plugin crate with crate-type = [\"cdylib\", ...]\n  - Or pass an explicit package once available: cargo xtask bundle --package <plugin-crate>\n  - For generated plugin projects, use: wavecraft bundle"
        );
    }

    if bundleable_packages.len() > 1 {
        anyhow::bail!(
            "Multiple bundleable plugin crates detected: {}.\nPlease specify one explicitly: cargo xtask bundle --package <plugin-crate>",
            bundleable_packages.join(", ")
        );
    }

    Ok(bundleable_packages[0].clone())
}

fn discover_bundleable_packages(engine_dir: &Path) -> Result<Vec<String>> {
    let crates_dir = engine_dir.join("crates");
    let mut bundleable_packages = Vec::new();

    if !crates_dir.exists() {
        return Ok(bundleable_packages);
    }

    for entry in fs::read_dir(&crates_dir)
        .with_context(|| format!("Failed to read crates directory: {}", crates_dir.display()))?
    {
        let entry = entry.with_context(|| {
            format!(
                "Failed to inspect crates directory entry in {}",
                crates_dir.display()
            )
        })?;

        let manifest_path = entry.path().join("Cargo.toml");
        if !manifest_path.exists() {
            continue;
        }

        let manifest = fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read {}", manifest_path.display()))?;
        let manifest_toml: toml::Value = manifest
            .parse()
            .with_context(|| format!("Failed to parse {}", manifest_path.display()))?;

        let crate_types = manifest_toml
            .get("lib")
            .and_then(|lib| lib.get("crate-type"))
            .and_then(|value| value.as_array());

        let is_bundleable = crate_types
            .map(|types| {
                types
                    .iter()
                    .filter_map(|value| value.as_str())
                    .any(|value| value == "cdylib")
            })
            .unwrap_or(false);

        let is_publishable = manifest_toml
            .get("package")
            .and_then(|package| package.get("publish"))
            .and_then(|value| value.as_bool())
            .unwrap_or(true);

        if is_bundleable
            && is_publishable
            && let Some(package_name) = manifest_toml
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(|value| value.as_str())
        {
            bundleable_packages.push(package_name.to_string());
        }
    }

    bundleable_packages.sort();
    Ok(bundleable_packages)
}

#[cfg(test)]
mod tests {
    use super::{discover_bundleable_packages, resolve_bundle_package};
    use tempfile::TempDir;

    fn write_manifest(path: &std::path::Path, name: &str, crate_types: Option<&str>) {
        std::fs::create_dir_all(path).expect("failed to create crate directory");

        let content = match crate_types {
            Some(types) => format!(
                "[package]\nname = \"{}\"\nversion = \"0.1.0\"\n\n[lib]\ncrate-type = [{}]\n",
                name, types
            ),
            None => format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\n", name),
        };

        std::fs::write(path.join("Cargo.toml"), content).expect("failed to write Cargo.toml");
    }

    #[test]
    fn discover_bundleable_packages_finds_cdylib_crates() {
        let temp = TempDir::new().expect("failed to create temp dir");
        let engine_dir = temp.path().join("engine");
        let crates_dir = engine_dir.join("crates");

        write_manifest(
            &crates_dir.join("wavecraft-nih_plug"),
            "wavecraft-nih_plug",
            Some("\"cdylib\", \"rlib\""),
        );
        write_manifest(
            &crates_dir.join("wavecraft-core"),
            "wavecraft-core",
            Some("\"rlib\""),
        );

        let packages = discover_bundleable_packages(&engine_dir)
            .expect("discover_bundleable_packages should succeed");

        assert_eq!(packages, vec!["wavecraft-nih_plug"]);
    }

    #[test]
    fn discover_bundleable_packages_ignores_non_publishable_cdylib_crates() {
        let temp = TempDir::new().expect("failed to create temp dir");
        let engine_dir = temp.path().join("engine");
        let crates_dir = engine_dir.join("crates");

        std::fs::create_dir_all(crates_dir.join("internal-plugin"))
            .expect("failed to create crate directory");
        std::fs::write(
            crates_dir.join("internal-plugin").join("Cargo.toml"),
            "[package]\nname = \"internal-plugin\"\nversion = \"0.1.0\"\npublish = false\n\n[lib]\ncrate-type = [\"cdylib\"]\n",
        )
        .expect("failed to write Cargo.toml");

        let packages = discover_bundleable_packages(&engine_dir)
            .expect("discover_bundleable_packages should succeed");

        assert!(packages.is_empty());
    }

    #[test]
    fn resolve_bundle_package_returns_requested_package() {
        let temp = TempDir::new().expect("failed to create temp dir");
        let engine_dir = temp.path().join("engine");

        let package = resolve_bundle_package(Some("custom-plugin"), &engine_dir)
            .expect("resolve_bundle_package should succeed");

        assert_eq!(package, "custom-plugin");
    }

    #[test]
    fn resolve_bundle_package_auto_selects_single_bundleable_package() {
        let temp = TempDir::new().expect("failed to create temp dir");
        let engine_dir = temp.path().join("engine");
        let crates_dir = engine_dir.join("crates");

        write_manifest(
            &crates_dir.join("wavecraft-nih_plug"),
            "wavecraft-nih_plug",
            Some("\"cdylib\", \"rlib\""),
        );

        let package = resolve_bundle_package(None, &engine_dir)
            .expect("resolve_bundle_package should auto-select package");

        assert_eq!(package, "wavecraft-nih_plug");
    }

    #[test]
    fn resolve_bundle_package_fails_when_no_bundleable_package_exists() {
        let temp = TempDir::new().expect("failed to create temp dir");
        let engine_dir = temp.path().join("engine");
        let crates_dir = engine_dir.join("crates");

        write_manifest(
            &crates_dir.join("wavecraft-core"),
            "wavecraft-core",
            Some("\"rlib\""),
        );

        let error = resolve_bundle_package(None, &engine_dir)
            .expect_err("resolve_bundle_package should fail when no bundleable crate exists");

        let error_text = error.to_string();
        assert!(error_text.contains("No bundleable plugin crate found in engine workspace"));
        assert!(error_text.contains("wavecraft bundle"));
    }

    #[test]
    fn resolve_bundle_package_fails_when_multiple_bundleable_packages_exist() {
        let temp = TempDir::new().expect("failed to create temp dir");
        let engine_dir = temp.path().join("engine");
        let crates_dir = engine_dir.join("crates");

        write_manifest(&crates_dir.join("plugin-a"), "plugin-a", Some("\"cdylib\""));
        write_manifest(&crates_dir.join("plugin-b"), "plugin-b", Some("\"cdylib\""));

        let error = resolve_bundle_package(None, &engine_dir)
            .expect_err("resolve_bundle_package should fail when multiple bundleable crates exist");

        let error_text = error.to_string();
        assert!(error_text.contains("Multiple bundleable plugin crates detected"));
        assert!(error_text.contains("--package"));
    }
}
