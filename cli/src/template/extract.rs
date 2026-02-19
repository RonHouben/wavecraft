use anyhow::{Context, Result};
use include_dir::Dir;
use std::fs;
use std::path::Path;

use crate::template::variables::TemplateVariables;

pub(super) fn extract_dir<F>(
    dir: &Dir,
    target_dir: &Path,
    vars: &TemplateVariables,
    post_process: &F,
) -> Result<()>
where
    F: Fn(&str, &TemplateVariables) -> Result<String> + ?Sized,
{
    fs::create_dir_all(target_dir)
        .with_context(|| format!("Failed to create directory: {}", target_dir.display()))?;

    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(subdir) => {
                // Skip build artifacts and dependencies
                let dir_name = subdir.path().file_name().with_context(|| {
                    format!("Invalid directory path: {}", subdir.path().display())
                })?;
                let dir_name_str = dir_name.to_string_lossy();

                if dir_name_str == "target"
                    || dir_name_str == "node_modules"
                    || dir_name_str == "dist"
                {
                    continue; // Skip these directories
                }

                let subdir_path = target_dir.join(dir_name);
                extract_dir(subdir, &subdir_path, vars, post_process)?;
            }
            include_dir::DirEntry::File(file) => {
                // Skip binary files and lock files
                let file_name = file
                    .path()
                    .file_name()
                    .with_context(|| format!("Invalid file path: {}", file.path().display()))?;
                let file_name_str = file_name.to_string_lossy();

                if file_name_str == "Cargo.lock"
                    || file_name_str.ends_with(".dylib")
                    || file_name_str.ends_with(".so")
                    || file_name_str.ends_with(".dll")
                    || file_name_str == ".DS_Store"
                {
                    continue; // Skip these files
                }

                // Handle .template files: rename back to original (e.g., Cargo.toml.template -> Cargo.toml)
                // These are renamed to avoid cargo treating the template as a crate during packaging.
                let output_name = if file_name_str.ends_with(".template") {
                    file_name_str
                        .strip_suffix(".template")
                        .expect("filename should have .template suffix after filter")
                        .to_string()
                } else {
                    file_name_str.to_string()
                };
                let file_path = target_dir.join(&output_name);

                // Only process text files
                if let Some(content) = file.contents_utf8() {
                    let processed = vars.apply(content).with_context(|| {
                        format!("Failed to process template: {}", file.path().display())
                    })?;

                    let processed = post_process(&processed, vars)?;

                    fs::write(&file_path, processed).with_context(|| {
                        format!("Failed to write file: {}", file_path.display())
                    })?;
                } else {
                    // Skip non-UTF8 files (likely binaries)
                    continue;
                }
            }
        }
    }

    Ok(())
}
