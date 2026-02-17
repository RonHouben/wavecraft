use std::fs;
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
fn test_bundle_requires_install_flag() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.arg("bundle");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("wavecraft bundle --install"));
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
