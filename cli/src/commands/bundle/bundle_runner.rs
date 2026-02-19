use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(super) fn run_nih_plug_bundle(engine_dir: &Path, package_name: &str) -> Result<()> {
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

pub(super) fn ensure_nih_plug_bundle_helper_manifest() -> Result<PathBuf> {
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
