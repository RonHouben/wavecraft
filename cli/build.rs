use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

const EXCLUDED_DIRS: &[&str] = &["target", "node_modules", "dist"];

fn main() {
    println!("cargo:rerun-if-changed=../sdk-template");

    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by Cargo"),
    );
    let src_root = manifest_dir.join("../sdk-template");
    let out_root =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set by Cargo")).join("sdk-template-clean");

    if out_root.exists() {
        fs::remove_dir_all(&out_root).unwrap_or_else(|err| {
            panic!(
                "Failed to remove stale staged template at {}: {err}",
                out_root.display()
            )
        });
    }

    fs::create_dir_all(&out_root).unwrap_or_else(|err| {
        panic!(
            "Failed to create staged template directory at {}: {err}",
            out_root.display()
        )
    });

    copy_filtered(&src_root, &out_root).unwrap_or_else(|err| {
        panic!(
            "Failed to stage template from {} to {}: {err}",
            src_root.display(),
            out_root.display()
        )
    });
}

fn copy_filtered(src: &Path, dst: &Path) -> io::Result<()> {
    for entry_result in fs::read_dir(src)? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let src_path = entry.path();
        let dst_path = dst.join(&name);

        if file_type.is_dir() {
            if EXCLUDED_DIRS.iter().any(|excluded| *excluded == name_str) {
                continue;
            }

            fs::create_dir_all(&dst_path)?;
            copy_filtered(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path)?;
        } else {
            // Skip symlinks and other special file types.
            continue;
        }
    }

    Ok(())
}