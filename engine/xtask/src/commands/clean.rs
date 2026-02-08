//! Clean command - Remove build artifacts and optionally installed plugins.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

use xtask::PLUGIN_DISPLAY_NAME;
use xtask::PLUGIN_NAME;
use xtask::Platform;
use xtask::output::*;
use xtask::paths;

/// Track cleaned directory with its reclaimed size.
struct CleanedItem {
    path: String,
    size_bytes: u64,
}

/// Calculate directory size recursively.
fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    let mut size = 0u64;
    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            } else if path.is_dir() {
                size += dir_size(&path);
            }
        }
    }

    size
}

/// Format bytes as human-readable size.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Remove a directory and track its size.
fn remove_dir(path: &Path, name: &str, verbose: bool) -> Result<Option<CleanedItem>> {
    if !path.exists() {
        if verbose {
            println!("  Skipping {} (not found)", name);
        }
        return Ok(None);
    }

    // Calculate size before deletion
    let size_bytes = dir_size(path);

    // Remove directory
    fs::remove_dir_all(path).with_context(|| format!("Failed to remove {}", path.display()))?;

    Ok(Some(CleanedItem {
        path: name.to_string(),
        size_bytes,
    }))
}

/// Run the clean command.
///
/// Cleans build artifacts across the entire workspace and optionally removes installed plugins.
pub fn run(include_installed: bool, force: bool, dry_run: bool, verbose: bool) -> Result<()> {
    let mut cleaned_items = Vec::new();

    print_status("Cleaning workspace build artifacts...");

    // 1. Run cargo clean for engine
    let engine_dir = paths::engine_dir()?;
    let engine_target = engine_dir.join("target");
    if engine_target.exists() {
        let size_bytes = dir_size(&engine_target);

        if dry_run {
            println!("  [dry-run] Would run: cargo clean in engine/");
        } else {
            if verbose {
                println!("Running: cargo clean in engine/");
            }
            let status = Command::new("cargo")
                .current_dir(&engine_dir)
                .arg("clean")
                .status()
                .context("Failed to run cargo clean")?;

            if status.success() {
                cleaned_items.push(CleanedItem {
                    path: "engine/target".to_string(),
                    size_bytes,
                });
            } else {
                print_warning("cargo clean returned non-zero exit code");
            }
        }
    } else if verbose {
        println!("  Skipping engine/target (not found)");
    }

    // 2. Clean CLI target directory
    let project_root = paths::project_root()?;
    let cli_target = project_root.join("cli").join("target");
    if let Some(item) = remove_dir(&cli_target, "cli/target", verbose)? {
        cleaned_items.push(item);
    }

    // 3. Clean UI build outputs
    let ui_dist = paths::ui_dir()?.join("dist");
    if let Some(item) = remove_dir(&ui_dist, "ui/dist", verbose)? {
        cleaned_items.push(item);
    }

    // 4. Clean UI test coverage
    let ui_coverage = paths::ui_dir()?.join("coverage");
    if let Some(item) = remove_dir(&ui_coverage, "ui/coverage", verbose)? {
        cleaned_items.push(item);
    }

    // 5. Clean temporary test artifacts
    let tmp_dir = project_root.join("target").join("tmp");
    if let Some(item) = remove_dir(&tmp_dir, "target/tmp", verbose)? {
        cleaned_items.push(item);
    }

    // 6. Remove bundled directory
    let bundled_dir = paths::bundled_dir()?;
    if bundled_dir.exists() {
        let size_bytes = dir_size(&bundled_dir);

        if dry_run {
            println!("  [dry-run] Would remove: {}", bundled_dir.display());
        } else {
            if verbose {
                println!("Removing: {}", bundled_dir.display());
            }
            fs::remove_dir_all(&bundled_dir)
                .with_context(|| format!("Failed to remove {}", bundled_dir.display()))?;

            cleaned_items.push(CleanedItem {
                path: "engine/target/bundled".to_string(),
                size_bytes,
            });
        }
    } else if verbose {
        println!("  Skipping engine/target/bundled (not found)");
    }

    // 7. Remove AU wrapper build directory (macOS)
    if Platform::current().is_macos() {
        let au_build_dir = paths::au_wrapper_dir()?.join("build");
        if let Some(item) = remove_dir(&au_build_dir, "packaging/macos/au-wrapper/build", verbose)?
        {
            cleaned_items.push(item);
        }
    }

    // Print summary of cleaned items
    if !dry_run && !cleaned_items.is_empty() {
        println!();
        for item in &cleaned_items {
            print_success_item(&format!("{} ({})", item.path, format_size(item.size_bytes)));
        }

        let total_bytes: u64 = cleaned_items.iter().map(|i| i.size_bytes).sum();
        println!();
        print_success(&format!(
            "Workspace cleaned successfully ({} reclaimed)",
            format_size(total_bytes)
        ));
    } else if cleaned_items.is_empty() && !dry_run {
        println!();
        print_info("No build artifacts found to clean");
    }

    // Remove installed plugins if requested
    if include_installed {
        if !force && !dry_run {
            println!();
            print_warning("Removing installed plugins requires --force flag for safety.");
            print_warning("Use: cargo xtask clean --installed --force");
            return Ok(());
        }

        println!();
        print_status("Removing installed plugins...");

        let mut plugin_items = Vec::new();

        // VST3
        let vst3_dir = paths::vst3_install_dir()?;
        let vst3_plugin = vst3_dir.join(format!("{}.vst3", PLUGIN_NAME));
        if vst3_plugin.exists() {
            let size_bytes = dir_size(&vst3_plugin);

            if dry_run {
                println!("  [dry-run] Would remove: {}", vst3_plugin.display());
            } else {
                fs::remove_dir_all(&vst3_plugin)
                    .with_context(|| format!("Failed to remove {}", vst3_plugin.display()))?;

                plugin_items.push(CleanedItem {
                    path: format!("{}.vst3", PLUGIN_NAME),
                    size_bytes,
                });
            }
        }

        // CLAP
        let clap_dir = paths::clap_install_dir()?;
        let clap_plugin = clap_dir.join(format!("{}.clap", PLUGIN_NAME));
        if clap_plugin.exists() {
            let size_bytes = dir_size(&clap_plugin);

            if dry_run {
                println!("  [dry-run] Would remove: {}", clap_plugin.display());
            } else {
                fs::remove_dir_all(&clap_plugin)
                    .with_context(|| format!("Failed to remove {}", clap_plugin.display()))?;

                plugin_items.push(CleanedItem {
                    path: format!("{}.clap", PLUGIN_NAME),
                    size_bytes,
                });
            }
        }

        // AU (macOS only)
        if Platform::current().is_macos() {
            let au_dir = paths::au_install_dir()?;
            let au_plugin = au_dir.join(format!("{}.component", PLUGIN_DISPLAY_NAME));
            if au_plugin.exists() {
                let size_bytes = dir_size(&au_plugin);

                if dry_run {
                    println!("  [dry-run] Would remove: {}", au_plugin.display());
                } else {
                    fs::remove_dir_all(&au_plugin)
                        .with_context(|| format!("Failed to remove {}", au_plugin.display()))?;

                    plugin_items.push(CleanedItem {
                        path: format!("{}.component", PLUGIN_DISPLAY_NAME),
                        size_bytes,
                    });
                }
            }
        }

        // Print plugin removal summary
        if !dry_run && !plugin_items.is_empty() {
            println!();
            for item in &plugin_items {
                print_success_item(&format!("{} ({})", item.path, format_size(item.size_bytes)));
            }

            let total_bytes: u64 = plugin_items.iter().map(|i| i.size_bytes).sum();
            println!();
            print_success(&format!(
                "Installed plugins removed ({} reclaimed)",
                format_size(total_bytes)
            ));
        } else if plugin_items.is_empty() && !dry_run {
            println!();
            print_info("No installed plugins found to remove");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 bytes");
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1 KB");
        assert_eq!(format_size(1536), "2 KB"); // Rounds to nearest KB
        assert_eq!(format_size(1024 * 1024), "1 MB");
        assert_eq!(format_size(1536 * 1024), "2 MB"); // Rounds to nearest MB
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1536 * 1024 * 1024), "1.50 GB");
    }

    #[test]
    fn test_dir_size_empty_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let size = dir_size(temp_dir.path());
        assert_eq!(size, 0, "Empty directory should have size 0");
    }

    #[test]
    fn test_dir_size_single_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        let mut file = fs::File::create(&file_path).expect("Failed to create file");
        file.write_all(b"Hello, World!")
            .expect("Failed to write file");
        drop(file);

        let size = dir_size(temp_dir.path());
        assert_eq!(size, 13, "Directory with 13-byte file should have size 13");
    }

    #[test]
    fn test_dir_size_multiple_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create file 1 (100 bytes)
        let file1_path = temp_dir.path().join("file1.txt");
        let mut file1 = fs::File::create(&file1_path).expect("Failed to create file1");
        file1
            .write_all(&vec![b'A'; 100])
            .expect("Failed to write file1");
        drop(file1);

        // Create file 2 (200 bytes)
        let file2_path = temp_dir.path().join("file2.txt");
        let mut file2 = fs::File::create(&file2_path).expect("Failed to create file2");
        file2
            .write_all(&vec![b'B'; 200])
            .expect("Failed to write file2");
        drop(file2);

        let size = dir_size(temp_dir.path());
        assert_eq!(
            size, 300,
            "Directory with 100+200 byte files should have size 300"
        );
    }

    #[test]
    fn test_dir_size_nested_dirs() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create nested directory structure
        let nested_dir = temp_dir.path().join("nested");
        fs::create_dir(&nested_dir).expect("Failed to create nested dir");

        // File in root (50 bytes)
        let root_file = temp_dir.path().join("root.txt");
        let mut file = fs::File::create(&root_file).expect("Failed to create root file");
        file.write_all(&vec![b'R'; 50])
            .expect("Failed to write root file");
        drop(file);

        // File in nested (75 bytes)
        let nested_file = nested_dir.join("nested.txt");
        let mut file = fs::File::create(&nested_file).expect("Failed to create nested file");
        file.write_all(&vec![b'N'; 75])
            .expect("Failed to write nested file");
        drop(file);

        let size = dir_size(temp_dir.path());
        assert_eq!(
            size, 125,
            "Directory with nested structure should sum all file sizes"
        );
    }

    #[test]
    fn test_dir_size_nonexistent() {
        let nonexistent_path = Path::new("/tmp/nonexistent_dir_test_12345");
        let size = dir_size(nonexistent_path);
        assert_eq!(size, 0, "Nonexistent directory should return size 0");
    }

    #[test]
    fn test_remove_dir_success() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_dir = temp_dir.path().join("test_remove");
        fs::create_dir(&test_dir).expect("Failed to create test dir");

        // Add a file
        let file_path = test_dir.join("file.txt");
        fs::write(&file_path, b"test content").expect("Failed to write file");

        let result = remove_dir(&test_dir, "test_remove", false);
        assert!(result.is_ok(), "remove_dir should succeed");

        let item = result.unwrap();
        assert!(item.is_some(), "Should return cleaned item");

        let item = item.unwrap();
        assert_eq!(item.path, "test_remove");
        assert_eq!(item.size_bytes, 12); // "test content" is 12 bytes

        assert!(!test_dir.exists(), "Directory should be removed");
    }

    #[test]
    fn test_remove_dir_nonexistent() {
        let nonexistent_path = Path::new("/tmp/nonexistent_dir_test_67890");
        let result = remove_dir(nonexistent_path, "nonexistent", false);

        assert!(
            result.is_ok(),
            "remove_dir should not error on nonexistent directory"
        );
        assert!(
            result.unwrap().is_none(),
            "Should return None for nonexistent directory"
        );
    }
}
