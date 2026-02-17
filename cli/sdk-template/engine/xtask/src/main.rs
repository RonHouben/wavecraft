use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const PLUGIN_PACKAGE: &str = "{{plugin_name}}";

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build automation for {{plugin_name}}", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bundle the plugin for distribution
    Bundle {
        /// Build in release mode (default)
        #[arg(long)]
        release: bool,

        /// Validate configuration without building (dry run)
        #[arg(long)]
        check: bool,

        /// Install VST3 plugin after a successful bundle build (macOS only)
        #[arg(short, long)]
        install: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bundle {
            release: _,
            check,
            install,
        } => {
            if check {
                println!("✓ Bundle configuration valid (dry run)");
                println!("  Bundle source: {}", bundled_dir()?.display());

                if install {
                    if cfg!(target_os = "macos") {
                        println!(
                            "✓ Install request validated (dry run): {}",
                            macos_vst3_install_dir()?.display()
                        );
                    } else {
                        println!(
                            "⚠ Install requested, but installation is only supported on macOS in this template"
                        );
                    }
                }

                return Ok(());
            }

            println!("Building {{plugin_name}} plugin...");

            // Build arguments for nih_plug_xtask
            let mut args = vec!["bundle".to_string(), PLUGIN_PACKAGE.to_string()];

            // Always use release mode for bundles
            args.push("--release".to_string());

            // Call nih_plug_xtask with the bundle command
            // This will compile and create VST3/CLAP bundles
            if let Err(e) = nih_plug_xtask::main_with_args("{{plugin_name_snake}}", args) {
                anyhow::bail!("Bundle command failed: {}", e);
            }

            println!("✓ Plugin bundled successfully");
            println!("  Find bundles in: {}", bundled_dir()?.display());

            if install {
                install_vst3_bundle()?;
            }

            Ok(())
        }
    }
}

fn bundled_dir() -> Result<PathBuf> {
    let cwd = env::current_dir().context("Failed to resolve current working directory")?;
    Ok(cwd.join("target").join("bundled"))
}

fn vst3_bundle_path() -> Result<PathBuf> {
    Ok(bundled_dir()?.join(format!("{}.vst3", PLUGIN_PACKAGE)))
}

fn macos_vst3_install_dir() -> Result<PathBuf> {
    if !cfg!(target_os = "macos") {
        bail!(
            "Install failed: `cargo xtask bundle --install` is currently supported on macOS only."
        );
    }

    let home = env::var("HOME")
        .context("Install failed: could not resolve HOME environment variable for install path")?;
    Ok(Path::new(&home)
        .join("Library")
        .join("Audio")
        .join("Plug-Ins")
        .join("VST3"))
}

fn install_vst3_bundle() -> Result<()> {
    let src = vst3_bundle_path()?;
    let dest_dir = macos_vst3_install_dir()?;

    let installed_path = install_vst3_bundle_at_path(&RealFileSystem, &src, &dest_dir)?;

    println!("✓ Plugin installed successfully");
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
            "Install failed: expected bundled VST3 artifact was not found at {}.\nRecovery: run `cargo xtask bundle --install` from the engine directory after the bundle stage succeeds.",
            src.display()
        );
    }

    fs_ops.create_dir_all(dest_dir).map_err(|e| {
        anyhow!(
            "Install failed while creating destination directory {}: {}. Recovery: check filesystem permissions and retry.",
            dest_dir.display(),
            e
        )
    })?;

    let file_name = src
        .file_name()
        .ok_or_else(|| anyhow!("Install failed: invalid VST3 source path {}", src.display()))?;
    let dest = dest_dir.join(file_name);

    if dest.exists() {
        fs_ops.remove_dir_all(&dest).map_err(|e| {
            anyhow!(
                "Install failed while replacing existing plugin at {}: {}. Recovery: close your DAW, verify write permissions, and retry.",
                dest.display(),
                e
            )
        })?;
    }

    fs_ops.copy_dir_recursive(src, &dest).map_err(|e| {
        anyhow!(
            "Install failed while copying VST3 bundle from {} to {}: {}. Recovery: close your DAW, verify write permissions, and retry.",
            src.display(),
            dest.display(),
            e
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

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
    fn install_reports_directory_create_failure_with_diagnostics() {
        let src = create_fake_bundle();
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
        let src = create_fake_bundle();
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
        let src = create_fake_bundle();
        let dest_dir = make_unique_temp_path("vst3-install-replace");
        let dest_bundle = dest_dir.join(format!("{}.vst3", PLUGIN_PACKAGE));
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

    fn create_fake_bundle() -> PathBuf {
        let bundle_dir =
            make_unique_temp_path("vst3-source-bundle").join(format!("{}.vst3", PLUGIN_PACKAGE));
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
        env::temp_dir().join(format!("{}-{}", prefix, nanos))
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
