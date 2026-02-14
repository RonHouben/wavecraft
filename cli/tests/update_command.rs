use std::process::Command;

#[test]
fn test_help_shows_update_command() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.arg("help");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("update"));
    assert!(stdout.contains("Update the CLI and project dependencies"));
}

#[test]
fn test_update_help_shows_any_directory_info() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.args(["update", "--help"]);

    let output = cmd.output().expect("Failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("any directory"),
        "Help should mention running from any directory"
    );
    assert!(stdout.contains("CLI"), "Help should mention CLI update");
}

#[test]
fn test_update_skip_self_flag_hidden_from_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.args(["update", "--help"]);

    let output = cmd.output().expect("Failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("skip-self"),
        "--skip-self should be hidden from help output"
    );
}
