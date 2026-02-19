use std::fs;
use std::process::Command;

use tempfile::TempDir;

#[test]
fn test_create_initializes_git_with_initial_commit_by_default() {
    let temp = TempDir::new().expect("temp dir should be created");
    let project_dir = temp.path().join("myPlugin");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "myPlugin",
        "--output",
        project_dir.to_str().expect("valid path"),
    ]);
    cmd.env("GIT_AUTHOR_NAME", "Wavecraft Test");
    cmd.env("GIT_AUTHOR_EMAIL", "test@example.com");
    cmd.env("GIT_COMMITTER_NAME", "Wavecraft Test");
    cmd.env("GIT_COMMITTER_EMAIL", "test@example.com");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(
        output.status.success(),
        "create command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(project_dir.join(".git").exists(), "Expected .git directory");

    let mut log_cmd = Command::new("git");
    log_cmd.current_dir(&project_dir);
    log_cmd.args(["log", "--oneline", "-1"]);
    let log_output = log_cmd.output().expect("Failed to read git log");

    assert!(
        log_output.status.success(),
        "git log failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&log_output.stdout),
        String::from_utf8_lossy(&log_output.stderr)
    );

    let latest_commit = String::from_utf8_lossy(&log_output.stdout);
    assert!(latest_commit.contains("Initial commit"));

    let mut status_cmd = Command::new("git");
    status_cmd.current_dir(&project_dir);
    status_cmd.args(["status", "--porcelain"]);
    let status_output = status_cmd.output().expect("Failed to read git status");
    assert!(status_output.status.success());

    let porcelain = String::from_utf8_lossy(&status_output.stdout);
    assert!(
        porcelain.trim().is_empty(),
        "Expected clean git status after scaffold, got: {porcelain}"
    );
}

#[test]
fn test_create_no_git_skips_repository_initialization() {
    let temp = TempDir::new().expect("temp dir should be created");
    let project_dir = temp.path().join("myPluginNoGit");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "myPluginNoGit",
        "--output",
        project_dir.to_str().expect("valid path"),
        "--no-git",
    ]);

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(
        output.status.success(),
        "create --no-git failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(project_dir.exists(), "Project directory should be created");
    assert!(
        !project_dir.join(".git").exists(),
        "No .git directory expected"
    );

    // Sanity check that scaffold files are present
    assert!(fs::metadata(project_dir.join("Cargo.toml")).is_ok());
    assert!(fs::metadata(project_dir.join("engine/Cargo.toml")).is_ok());
    assert!(fs::metadata(project_dir.join("ui/package.json")).is_ok());
}
