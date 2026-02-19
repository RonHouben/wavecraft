use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

pub(super) fn build_release_package(engine_dir: &Path, package_name: &str) -> Result<()> {
    let build_status = Command::new("cargo")
        .args(["build", "--release", "-p", package_name])
        .current_dir(engine_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run `cargo build --release`")?;

    if build_status.success() {
        Ok(())
    } else {
        let code = build_status.code().map_or_else(
            || "terminated by signal".to_string(),
            |value| value.to_string(),
        );

        bail!("Build failed (exit: {}).", code);
    }
}
