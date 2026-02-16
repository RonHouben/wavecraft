use serde_json::json;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

const CORE_REL_PATH: &str = "ui/packages/core/package.json";
const COMPONENTS_REL_PATH: &str = "ui/packages/components/package.json";
const TEMPLATE_REL_PATH: &str = "sdk-template/ui/package.json";

fn write_fixture_workspace(
    dir: &TempDir,
    core_version: &str,
    components_version: &str,
    components_peer_core: &str,
    template_core_dep: &str,
    template_components_dep: &str,
) {
    let root = dir.path();
    let core_path = root.join(CORE_REL_PATH);
    let components_path = root.join(COMPONENTS_REL_PATH);
    let template_path = root.join(TEMPLATE_REL_PATH);

    fs::create_dir_all(
        core_path
            .parent()
            .expect("core package.json path should have parent"),
    )
    .expect("failed to create core fixture directory");
    fs::create_dir_all(
        components_path
            .parent()
            .expect("components package.json path should have parent"),
    )
    .expect("failed to create components fixture directory");
    fs::create_dir_all(
        template_path
            .parent()
            .expect("template package.json path should have parent"),
    )
    .expect("failed to create template fixture directory");

    let core_manifest = json!({
        "name": "@wavecraft/core",
        "version": core_version,
        "description": "fixture"
    });

    let components_manifest = json!({
        "name": "@wavecraft/components",
        "version": components_version,
        "description": "fixture",
        "peerDependencies": {
            "@wavecraft/core": components_peer_core,
            "react": "^18.0.0"
        }
    });

    let template_manifest = json!({
        "name": "fixture-template-ui",
        "private": true,
        "version": "0.1.0",
        "dependencies": {
            "@wavecraft/core": template_core_dep,
            "@wavecraft/components": template_components_dep,
            "react": "^18.3.1"
        }
    });

    fs::write(
        &core_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&core_manifest)
                .expect("failed to serialize core fixture manifest")
        ),
    )
    .expect("failed to write core fixture manifest");
    fs::write(
        &components_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&components_manifest)
                .expect("failed to serialize components fixture manifest")
        ),
    )
    .expect("failed to write components fixture manifest");
    fs::write(
        &template_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&template_manifest)
                .expect("failed to serialize template fixture manifest")
        ),
    )
    .expect("failed to write template fixture manifest");
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
