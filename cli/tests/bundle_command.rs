use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

#[test]
fn test_help_shows_bundle_command() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.arg("help");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bundle"));
}

#[test]
fn test_bundle_help_shows_install_flag() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.args(["bundle", "--help"]);

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--install"));
    assert!(stdout.contains("Canonical install workflow"));
}

#[test]
fn test_bundle_without_install_invalid_context_has_actionable_message() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.arg("bundle");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("generated plugin project"));
}

#[test]
fn test_bundle_install_invalid_context_has_actionable_message() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.current_dir(temp.path());
    cmd.args(["bundle", "--install"]);

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid project context"));
    assert!(stderr.contains("ui/package.json"));
    assert!(stderr.contains("engine/Cargo.toml"));
    assert!(stderr.contains("wavecraft bundle --install"));
}

#[test]
fn test_bundle_install_detects_project_root_from_subdirectory() {
    let temp = TempDir::new().expect("temp dir should be created");
    let root = temp.path();

    fs::create_dir_all(root.join("ui/src")).expect("ui dir");
    fs::create_dir_all(root.join("engine")).expect("engine dir");
    fs::write(root.join("ui/package.json"), "{}").expect("ui package");
    fs::write(root.join("engine/Cargo.toml"), "[package]\nname='demo'").expect("engine cargo");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.current_dir(root.join("ui/src"));
    cmd.args(["bundle", "--install"]);

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(!output.status.success());

    // It should fail at delegated xtask (not context detection), which proves root resolution worked.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Delegated command failed")
            || stderr.contains("Failed to run delegated command")
    );
}

#[test]
fn test_bundle_install_delegates_successfully_with_expected_command_and_project_root_cwd() {
    let temp = TempDir::new().expect("temp dir should be created");
    let root = temp.path();

    fs::create_dir_all(root.join("ui/src")).expect("ui dir");
    fs::create_dir_all(root.join("engine")).expect("engine dir");
    fs::write(root.join("ui/package.json"), "{}").expect("ui package");
    fs::write(root.join("engine/Cargo.toml"), "[package]\nname='demo'").expect("engine cargo");

    let bin_dir = root.join("fake-bin");
    fs::create_dir_all(&bin_dir).expect("fake bin dir");

    let captured_args = root.join("captured-args.txt");
    let captured_cwd = root.join("captured-cwd.txt");

    let cargo_shim = bin_dir.join("cargo");
    fs::write(
        &cargo_shim,
        "#!/bin/sh\nprintf '%s' \"$*\" > \"$WAVECRAFT_CAPTURE_ARGS\"\nprintf '%s' \"$PWD\" > \"$WAVECRAFT_CAPTURE_CWD\"\nexit 0\n",
    )
    .expect("write cargo shim");
    make_executable(&cargo_shim);

    let current_path = std::env::var("PATH").unwrap_or_default();
    let shimmed_path = format!("{}:{}", bin_dir.display(), current_path);

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.current_dir(root.join("ui/src"));
    cmd.env("PATH", shimmed_path);
    cmd.env("WAVECRAFT_CAPTURE_ARGS", &captured_args);
    cmd.env("WAVECRAFT_CAPTURE_CWD", &captured_cwd);
    cmd.args(["bundle", "--install"]);

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(
        output.status.success(),
        "Expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let delegated_args =
        fs::read_to_string(&captured_args).expect("delegated args should be captured");
    assert_eq!(delegated_args.trim(), "xtask bundle --install");

    let delegated_cwd =
        fs::read_to_string(&captured_cwd).expect("delegated cwd should be captured");
    let delegated_cwd_path =
        fs::canonicalize(delegated_cwd.trim()).expect("canonical delegated cwd");
    let expected_engine =
        fs::canonicalize(root.join("engine")).expect("canonical expected engine cwd");
    assert_eq!(delegated_cwd_path, expected_engine);
}

fn make_executable(path: &Path) {
    let metadata = fs::metadata(path).expect("metadata");
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("set executable permissions");
}
