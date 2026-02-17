use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use tempfile::TempDir;

#[test]
fn test_help_shows_bundle_command() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.arg("help");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(
        output.status.success(),
        "bundle command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bundle"));
}

#[test]
fn test_bundle_help_shows_install_flag() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.args(["bundle", "--help"]);

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(
        output.status.success(),
        "bundle --install command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--install"));
    assert!(stdout.contains("CLI-owned build, bundle"));
}

#[test]
fn test_bundle_without_install_invalid_context_has_actionable_message() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.current_dir(temp.path());
    cmd.arg("bundle");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid project context"));
    assert!(stderr.contains("wavecraft bundle"));
    assert!(!stderr.contains("wavecraft bundle --install"));
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

    // It should fail during bundle/install execution (not context detection), proving root resolution worked.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("Invalid project context"));
}

#[test]
fn test_bundle_delegates_build_ui_before_bundle() {
    let temp = TempDir::new().expect("temp dir should be created");
    let root = create_minimal_project(temp.path());
    let fake_bin_dir = root.join("fake-bin");
    fs::create_dir_all(&fake_bin_dir).expect("fake bin dir");

    let npm_invocations_path = root.join("npm-invocations.log");
    create_fake_runner(&fake_bin_dir.join("npm"), &npm_invocations_path);

    let cargo_invocations_path = root.join("cargo-invocations.log");
    create_fake_runner(&fake_bin_dir.join("cargo"), &cargo_invocations_path);

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.current_dir(root);
    cmd.arg("bundle");
    cmd.env("PATH", prepend_path(&fake_bin_dir));

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(
        !output.status.success(),
        "bundle command unexpectedly succeeded in fixture"
    );

    let npm_invocations =
        fs::read_to_string(npm_invocations_path).expect("npm invocations should be captured");
    assert_eq!(npm_invocations, "run build\n");

    let cargo_invocations = fs::read_to_string(cargo_invocations_path)
        .expect("cargo invocations should be captured");
    let lines: Vec<&str> = cargo_invocations.lines().collect();
    assert!(!lines.is_empty(), "expected at least one cargo invocation");
    assert!(
        lines
            .get(1)
            .is_none(),
        "expected only one cargo invocation before bundling, got: {:?}",
        lines
    );
    assert!(
        lines[0].starts_with("build --release -p fake-engine"),
        "expected cargo build invocation, got: {:?}",
        lines[0]
    );
}

#[test]
fn test_bundle_install_delegates_build_ui_before_bundle_install() {
    let temp = TempDir::new().expect("temp dir should be created");
    let root = create_minimal_project(temp.path());
    let fake_bin_dir = root.join("fake-bin");
    fs::create_dir_all(&fake_bin_dir).expect("fake bin dir");

    let npm_invocations_path = root.join("npm-invocations.log");
    create_fake_runner(&fake_bin_dir.join("npm"), &npm_invocations_path);

    let cargo_invocations_path = root.join("cargo-invocations.log");
    create_fake_runner(&fake_bin_dir.join("cargo"), &cargo_invocations_path);

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.current_dir(root);
    cmd.args(["bundle", "--install"]);
    cmd.env("PATH", prepend_path(&fake_bin_dir));

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(
        !output.status.success(),
        "bundle --install command unexpectedly succeeded in fixture"
    );

    let npm_invocations =
        fs::read_to_string(npm_invocations_path).expect("npm invocations should be captured");
    assert_eq!(npm_invocations, "run build\n");

    let cargo_invocations = fs::read_to_string(cargo_invocations_path)
        .expect("cargo invocations should be captured");
    let lines: Vec<&str> = cargo_invocations.lines().collect();
    assert!(!lines.is_empty(), "expected at least one cargo invocation");
    assert!(
        lines
            .get(1)
            .is_none(),
        "expected only one cargo invocation before bundling, got: {:?}",
        lines
    );
    assert!(
        lines[0].starts_with("build --release -p fake-engine"),
        "expected cargo build invocation, got: {:?}",
        lines[0]
    );
}

fn create_minimal_project(base: &std::path::Path) -> &std::path::Path {
    fs::create_dir_all(base.join("ui")).expect("ui dir");
    fs::create_dir_all(base.join("engine")).expect("engine dir");
    fs::create_dir_all(base.join("ui/node_modules")).expect("ui node_modules dir");
    fs::write(base.join("ui/package.json"), "{}\n").expect("ui package");
    fs::write(
        base.join("engine/Cargo.toml"),
        "[package]\nname = \"fake-engine\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("engine cargo");
    base
}

fn create_fake_runner(fake_path: &std::path::Path, invocations_path: &std::path::Path) {
    let script = format!(
        "#!/bin/sh\necho \"$@\" >> \"{}\"\nexit 0\n",
        invocations_path.display()
    );
    fs::write(fake_path, script).expect("fake runner script");

    let mut perms = fs::metadata(fake_path)
        .expect("fake runner metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(fake_path, perms).expect("fake runner permissions");
}

fn prepend_path(fake_bin_dir: &std::path::Path) -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", fake_bin_dir.display(), current)
}
