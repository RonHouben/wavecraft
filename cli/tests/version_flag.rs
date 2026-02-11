use std::process::Command;

#[test]
fn test_version_flag_long_form() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.arg("--version");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("wavecraft "));
}

#[test]
fn test_version_flag_short_form() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.arg("-V");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("wavecraft "));
}

#[test]
fn test_version_flag_format() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.arg("--version");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should match format "wavecraft X.Y.Z"
    assert!(stdout.starts_with("wavecraft "));

    let parts: Vec<&str> = stdout.trim().split(' ').collect();
    assert_eq!(parts.len(), 2, "Version output should be 'wavecraft X.Y.Z'");
    assert_eq!(parts[0], "wavecraft");

    // Verify version number format (X.Y.Z)
    let version = parts[1];
    let version_parts: Vec<&str> = version.split('.').collect();
    assert_eq!(version_parts.len(), 3, "Version should have format X.Y.Z");

    // Each part should be a number
    for part in version_parts {
        assert!(
            part.parse::<u32>().is_ok(),
            "Version parts should be numbers"
        );
    }
}

#[test]
fn test_help_shows_version_flag() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.arg("--help");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-V, --version"));
    assert!(stdout.contains("Print version"));
}
