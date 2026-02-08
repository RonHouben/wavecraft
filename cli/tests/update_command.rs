use assert_cmd::cargo::cargo_bin;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_help_shows_update_command() {
    let mut cmd = Command::new(cargo_bin!("wavecraft"));
    cmd.arg("help");

    let output = cmd.output().expect("Failed to execute wavecraft binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("update"));
    assert!(stdout.contains("Update the CLI and project dependencies"));
}

#[test]
fn test_update_help_shows_any_directory_info() {
    let mut cmd = Command::new(cargo_bin!("wavecraft"));
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
#[ignore] // Triggers `cargo install wavecraft` which requires network access and crates.io
fn test_update_outside_plugin_project() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");

    let mut cmd = Command::new(cargo_bin!("wavecraft"));
    cmd.current_dir(temp_dir.path());
    cmd.arg("update");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // New behavior: running outside a project is no longer an error.
    // Phase 1 (CLI self-update) runs, then Phase 2 detects not in a project.
    assert!(
        combined.contains("Not in a Wavecraft plugin project"),
        "Should show info message about not being in a project"
    );
    assert!(
        combined.contains("skipping dependency updates"),
        "Should mention skipping dependency updates"
    );
}

#[test]
#[ignore] // Triggers `cargo install wavecraft` which requires network access and crates.io
fn test_update_detects_engine_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");

    // Create engine directory structure
    let engine_dir = temp_dir.path().join("engine");
    fs::create_dir(&engine_dir).expect("Failed to create engine directory");
    fs::write(
        engine_dir.join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .expect("Failed to write Cargo.toml");

    let mut cmd = Command::new(cargo_bin!("wavecraft"));
    cmd.current_dir(temp_dir.path());
    cmd.arg("update");

    // Should at least attempt to update Rust dependencies
    // It will fail because there's no valid Cargo project, but it should detect it
    let output = cmd
        .output()
        .expect("Failed to execute wavecraft update command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("Updating Rust dependencies") || combined.contains("Rust"),
        "Should detect and attempt to update Rust dependencies"
    );
}

#[test]
#[ignore] // Triggers `cargo install wavecraft` which requires network access and crates.io
fn test_update_detects_ui_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");

    // Create ui directory structure
    let ui_dir = temp_dir.path().join("ui");
    fs::create_dir(&ui_dir).expect("Failed to create ui directory");
    fs::write(
        ui_dir.join("package.json"),
        "{\"name\": \"test\", \"version\": \"0.1.0\"}",
    )
    .expect("Failed to write package.json");

    let mut cmd = Command::new(cargo_bin!("wavecraft"));
    cmd.current_dir(temp_dir.path());
    cmd.arg("update");

    // Should at least attempt to update npm dependencies
    let output = cmd
        .output()
        .expect("Failed to execute wavecraft update command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("Updating npm dependencies") || combined.contains("npm"),
        "Should detect and attempt to update npm dependencies"
    );
}

#[test]
#[ignore] // Triggers `cargo install wavecraft` which requires network access and crates.io
fn test_update_command_output_format() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");

    // Create minimal plugin structure
    let engine_dir = temp_dir.path().join("engine");
    fs::create_dir(&engine_dir).expect("Failed to create engine directory");
    fs::write(
        engine_dir.join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .expect("Failed to write Cargo.toml");

    let mut cmd = Command::new(cargo_bin!("wavecraft"));
    cmd.current_dir(temp_dir.path());
    cmd.arg("update");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for emoji indicators (even if command fails, should see these)
    assert!(
        stdout.contains("📦") || stdout.contains("✅") || stdout.contains("❌"),
        "Output should contain emoji indicators"
    );
}
