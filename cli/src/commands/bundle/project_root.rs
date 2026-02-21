use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub(super) fn resolve_project_root(start_dir: &Path, install: bool) -> Result<PathBuf> {
    if let Some(root) = find_wavecraft_project_root(start_dir) {
        return Ok(root);
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
}
