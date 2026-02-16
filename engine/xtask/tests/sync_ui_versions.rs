use std::fs;
use std::process::Command;
use tempfile::TempDir;
use xtask::test_support::{
    SyncUiFixtureVersions, TEMPLATE_REL_PATH, write_sync_ui_fixture_workspace,
};

fn write_fixture_workspace(
    dir: &TempDir,
    core_version: &str,
    components_version: &str,
    components_peer_core: &str,
    template_core_dep: &str,
    template_components_dep: &str,
) {
    write_sync_ui_fixture_workspace(
        dir.path(),
        SyncUiFixtureVersions {
            core_version,
            components_version,
            components_peer_core,
            template_core_dep,
            template_components_dep,
        },
    )
    .expect("failed to write sync-ui fixture workspace");
}

#[test]
fn check_mode_returns_one_when_scoped_drift_exists() {
    let dir = TempDir::new().expect("failed to create temp directory");
    write_fixture_workspace(&dir, "0.7.5", "0.7.4", "^0.7.0", "^0.7.1", "^0.7.1");

    let status = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(dir.path())
        .env_remove("CARGO_MANIFEST_DIR")
        .args(["sync-ui-versions", "--check"])
        .status()
        .expect("sync-ui-versions --check should run");

    assert_eq!(status.code(), Some(1));
}

#[test]
fn apply_then_check_is_zero_and_aligned() {
    let dir = TempDir::new().expect("failed to create temp directory");
    write_fixture_workspace(&dir, "0.7.5", "0.7.4", "^0.7.0", "^0.7.1", "^0.7.1");

    let apply_status = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(dir.path())
        .env_remove("CARGO_MANIFEST_DIR")
        .args(["sync-ui-versions", "--apply"])
        .status()
        .expect("sync-ui-versions --apply should run");
    assert_eq!(apply_status.code(), Some(0));

    let template_after: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join(TEMPLATE_REL_PATH))
            .expect("failed to read template manifest after apply"),
    )
    .expect("failed to parse template manifest after apply");

    let dep_core = template_after
        .pointer("/dependencies/@wavecraft~1core")
        .and_then(serde_json::Value::as_str)
        .expect("template core dependency should exist");
    let dep_components = template_after
        .pointer("/dependencies/@wavecraft~1components")
        .and_then(serde_json::Value::as_str)
        .expect("template components dependency should exist");

    assert_eq!(dep_core, "^0.7.5");
    assert_eq!(dep_components, "^0.7.5");

    let check_status = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(dir.path())
        .env_remove("CARGO_MANIFEST_DIR")
        .args(["sync-ui-versions", "--check"])
        .status()
        .expect("sync-ui-versions --check should run");
    assert_eq!(check_status.code(), Some(0));
}

#[test]
fn policy_violation_returns_two() {
    let dir = TempDir::new().expect("failed to create temp directory");
    write_fixture_workspace(&dir, "0.8.0", "0.7.5", "^0.7.5", "^0.7.5", "^0.7.5");

    let status = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(dir.path())
        .env_remove("CARGO_MANIFEST_DIR")
        .args(["sync-ui-versions", "--check"])
        .status()
        .expect("sync-ui-versions --check should run");

    assert_eq!(status.code(), Some(2));
}
