//! Build script for the Wavecraft CLI.
//!
//! This script copies the plugin-template directory into the CLI crate so that
//! `include_dir!` can embed it at compile time.
//!
//! The template lives at the repository root (`../plugin-template`) but `cargo publish`
//! only packages files within the crate directory. This script ensures the template
//! is available during both local development and crate verification.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell Cargo to re-run build.rs if the template changes
    println!("cargo::rerun-if-changed=../plugin-template");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = Path::new(&manifest_dir);

    let source = manifest_path.join("../plugin-template");
    let dest = manifest_path.join("plugin-template");

    // If source exists and dest doesn't (or is outdated), copy
    if source.exists() && source.is_dir() {
        // Remove existing copy to ensure fresh state
        if dest.exists() {
            let _ = fs::remove_dir_all(&dest);
        }

        copy_dir_recursive(&source, &dest)
            .expect("Failed to copy plugin-template to cli/plugin-template");

        println!(
            "cargo::warning=Copied plugin-template from {} to {}",
            source.display(),
            dest.display()
        );
    } else if !dest.exists() {
        // Neither source nor dest exists - this happens during `cargo publish` verification
        // when the tarball already contains the template. Check if we're in a package dir.
        let in_package_dir = manifest_path
            .to_string_lossy()
            .contains("target/package/wavecraft-");

        if !in_package_dir {
            panic!(
                "plugin-template not found at {} and cli/plugin-template doesn't exist. \
                 Are you building from the repository root?",
                source.display()
            );
        }
        // During package verification, the template should already be in the tarball
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        // Skip build artifacts and dependencies
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "target"
            || name_str == "node_modules"
            || name_str == "dist"
            || name_str == ".git"
            || name_str == "Cargo.lock"
        {
            continue;
        }

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
