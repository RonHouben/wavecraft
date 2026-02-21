use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::sdk_detect::detect_sdk_repo;

pub(super) fn resolve_project_root(start_dir: &Path, install: bool) -> Result<PathBuf> {
    resolve_project_root_with_sdk_hint(start_dir, install, detect_sdk_repo())
}

fn resolve_project_root_with_sdk_hint(
    start_dir: &Path,
    install: bool,
    sdk_crates_dir_hint: Option<PathBuf>,
) -> Result<PathBuf> {
    if let Some(root) = find_wavecraft_project_root(start_dir) {
        return Ok(root);
    }

    if install {
        if let Some(sdk_root) = resolve_sdk_dev_repo_root(start_dir, sdk_crates_dir_hint.as_deref())
        {
            return Ok(sdk_root);
        }
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

fn resolve_sdk_dev_repo_root(start_dir: &Path, sdk_crates_dir: Option<&Path>) -> Option<PathBuf> {
    let sdk_crates_dir = sdk_crates_dir?;
    let sdk_root = sdk_crates_dir.parent()?.parent()?.to_path_buf();

    if same_canonical_path(start_dir, &sdk_root) {
        Some(sdk_root)
    } else {
        None
    }
}

fn same_canonical_path(left: &Path, right: &Path) -> bool {
    let left = match left.canonicalize() {
        Ok(path) => path,
        Err(_) => return false,
    };

    let right = match right.canonicalize() {
        Ok(path) => path,
        Err(_) => return false,
    };

    left == right
}

pub(super) fn find_wavecraft_project_root(start_dir: &Path) -> Option<PathBuf> {
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
    use std::fs;
    use tempfile::TempDir;

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
    fn resolve_project_root_prefers_generated_project_over_sdk_dev_hint() {
        let temp = TempDir::new().expect("temp dir should be created");
        let plugin_root = temp.path().join("my-plugin");

        fs::create_dir_all(plugin_root.join("ui/src")).expect("ui src dir");
        fs::create_dir_all(plugin_root.join("engine")).expect("engine dir");
        fs::write(plugin_root.join("ui/package.json"), "{}").expect("ui package");
        fs::write(
            plugin_root.join("engine/Cargo.toml"),
            "[package]\nname='demo'",
        )
        .expect("engine cargo");

        let sdk_root = temp.path().join("sdk");
        let sdk_crates = sdk_root.join("engine/crates");
        fs::create_dir_all(&sdk_crates).expect("sdk crates dir");

        let resolved =
            resolve_project_root_with_sdk_hint(&plugin_root.join("ui/src"), true, Some(sdk_crates))
                .expect("should resolve plugin project root");

        assert_eq!(resolved, plugin_root);
    }

    #[test]
    fn resolve_project_root_install_uses_sdk_dev_root_when_cwd_is_repo_root() {
        let temp = TempDir::new().expect("temp dir should be created");
        let sdk_root = temp.path().join("wavecraft");
        let sdk_crates = sdk_root.join("engine/crates");
        fs::create_dir_all(&sdk_crates).expect("sdk crates dir");

        let resolved = resolve_project_root_with_sdk_hint(&sdk_root, true, Some(sdk_crates))
            .expect("should resolve sdk repo root for install");

        assert_eq!(resolved, sdk_root);
    }

    #[test]
    fn resolve_project_root_install_still_errors_for_unrelated_directory() {
        let temp = TempDir::new().expect("temp dir should be created");
        let sdk_root = temp.path().join("wavecraft");
        let sdk_crates = sdk_root.join("engine/crates");
        fs::create_dir_all(&sdk_crates).expect("sdk crates dir");

        let unrelated = temp.path().join("somewhere-else");
        fs::create_dir_all(&unrelated).expect("unrelated dir");

        let result = resolve_project_root_with_sdk_hint(&unrelated, true, Some(sdk_crates));

        assert!(result.is_err());
        let message = result.expect_err("should fail").to_string();
        assert!(message.contains("Invalid project context"));
        assert!(message.contains("wavecraft bundle --install"));
    }
}
