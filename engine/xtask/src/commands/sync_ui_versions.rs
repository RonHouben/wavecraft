//! Sync UI versions command.
//!
//! Enforces deterministic alignment across scoped keys in exactly three files:
//! - ui/packages/core/package.json
//! - ui/packages/components/package.json
//! - sdk-template/ui/package.json
//!
//! Exit code contract:
//! - 0: aligned / apply success
//! - 1: drift detected in check mode
//! - 2: operational/configuration/policy errors (mapped by caller)

use anyhow::{Context, Result, bail};
use semver::Version;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use xtask::output::*;
use xtask::paths;

const CORE_REL_PATH: &str = "ui/packages/core/package.json";
const COMPONENTS_REL_PATH: &str = "ui/packages/components/package.json";
const TEMPLATE_REL_PATH: &str = "sdk-template/ui/package.json";

const CORE_VERSION_PTR: &str = "/version";
const COMPONENTS_VERSION_PTR: &str = "/version";
const COMPONENTS_PEER_CORE_PTR: &str = "/peerDependencies/@wavecraft~1core";
const TEMPLATE_DEP_CORE_PTR: &str = "/dependencies/@wavecraft~1core";
const TEMPLATE_DEP_COMPONENTS_PTR: &str = "/dependencies/@wavecraft~1components";

/// Command mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// Non-mutating drift detection mode.
    Check,
    /// Mutating alignment mode.
    Apply,
}

/// Configuration for sync-ui-versions.
#[derive(Debug, Clone, Copy)]
pub struct SyncUiVersionsConfig {
    /// Command mode.
    pub mode: SyncMode,
    /// Allow minor updates in addition to patch updates.
    pub allow_minor: bool,
    /// Allow major updates (and therefore minor + patch).
    pub allow_major: bool,
    /// Show additional diagnostics.
    pub verbose: bool,
}

#[derive(Debug, Clone)]
struct ScopeFiles {
    core: PathBuf,
    components: PathBuf,
    template: PathBuf,
}

#[derive(Debug, Clone)]
struct LoadedState {
    core_json: Value,
    components_json: Value,
    template_json: Value,
    core_version: Version,
    components_version: Version,
    components_peer_core: Version,
    template_dep_core: Version,
    template_dep_components: Version,
}

#[derive(Debug, Clone)]
struct ExpectedState {
    target_version: Version,
    target_caret: String,
}

#[derive(Debug, Clone)]
struct Drift {
    file: &'static str,
    key: &'static str,
    current: String,
    expected: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VersionMovement {
    None,
    Patch,
    Minor,
    Major,
}

/// Run the sync-ui-versions command.
pub fn run(config: SyncUiVersionsConfig) -> Result<i32> {
    let root = paths::project_root()?;
    run_at_root(&root, config)
}

fn run_at_root(root: &Path, config: SyncUiVersionsConfig) -> Result<i32> {
    let scope = scoped_files(root);

    print_header("Sync UI Versions");
    if config.verbose {
        print_info(&format!("Workspace root: {}", root.display()));
        print_info(&format!("Mode: {}", mode_label(config.mode)));
        print_info(&format!(
            "Policy: patch + {} + {}",
            if config.allow_minor || config.allow_major {
                "minor"
            } else {
                "(minor blocked)"
            },
            if config.allow_major {
                "major"
            } else {
                "(major blocked)"
            }
        ));
        println!();
    }

    let loaded = load_state(&scope)?;
    let expected = compute_expected(&loaded, config)?;
    let drifts = compute_drifts(&loaded, &expected);

    match config.mode {
        SyncMode::Check => run_check_mode(drifts),
        SyncMode::Apply => run_apply_mode(&scope, loaded, expected, drifts, config),
    }
}

fn run_check_mode(drifts: Vec<Drift>) -> Result<i32> {
    if drifts.is_empty() {
        print_success("All scoped UI versions are aligned.");
        return Ok(0);
    }

    print_warning("Scoped drift detected:");
    for drift in drifts {
        println!(
            "  - {} :: {}\n      current: {}\n      expected: {}",
            drift.file, drift.key, drift.current, drift.expected
        );
    }

    Ok(1)
}

fn run_apply_mode(
    scope: &ScopeFiles,
    mut loaded: LoadedState,
    expected: ExpectedState,
    drifts: Vec<Drift>,
    config: SyncUiVersionsConfig,
) -> Result<i32> {
    if drifts.is_empty() {
        print_success("No changes required. Scoped UI versions are already aligned.");
        return Ok(0);
    }

    let core_version_drift = drift_exists(&drifts, CORE_REL_PATH, "version");
    let components_version_drift = drift_exists(&drifts, COMPONENTS_REL_PATH, "version");
    let components_peer_core_drift = drift_exists(
        &drifts,
        COMPONENTS_REL_PATH,
        "peerDependencies[@wavecraft/core]",
    );
    let template_dep_core_drift =
        drift_exists(&drifts, TEMPLATE_REL_PATH, "dependencies[@wavecraft/core]");
    let template_dep_components_drift = drift_exists(
        &drifts,
        TEMPLATE_REL_PATH,
        "dependencies[@wavecraft/components]",
    );

    if core_version_drift {
        set_string(
            &mut loaded.core_json,
            CORE_VERSION_PTR,
            expected.target_version.to_string(),
        )?;
    }
    if components_version_drift {
        set_string(
            &mut loaded.components_json,
            COMPONENTS_VERSION_PTR,
            expected.target_version.to_string(),
        )?;
    }
    if components_peer_core_drift {
        set_string(
            &mut loaded.components_json,
            COMPONENTS_PEER_CORE_PTR,
            expected.target_caret.clone(),
        )?;
    }
    if template_dep_core_drift {
        set_string(
            &mut loaded.template_json,
            TEMPLATE_DEP_CORE_PTR,
            expected.target_caret.clone(),
        )?;
    }
    if template_dep_components_drift {
        set_string(
            &mut loaded.template_json,
            TEMPLATE_DEP_COMPONENTS_PTR,
            expected.target_caret,
        )?;
    }

    let mut changed_files = Vec::new();
    if core_version_drift && write_json_if_changed(&scope.core, &loaded.core_json)? {
        changed_files.push(CORE_REL_PATH);
    }
    if (components_version_drift || components_peer_core_drift)
        && write_json_if_changed(&scope.components, &loaded.components_json)?
    {
        changed_files.push(COMPONENTS_REL_PATH);
    }
    if (template_dep_core_drift || template_dep_components_drift)
        && write_json_if_changed(&scope.template, &loaded.template_json)?
    {
        changed_files.push(TEMPLATE_REL_PATH);
    }

    let verified = load_state(scope)?;
    let expected_after_verify = compute_expected(&verified, config)?;
    let remaining_drifts = compute_drifts(&verified, &expected_after_verify);

    if !remaining_drifts.is_empty() {
        bail!("Post-write verification failed: scoped drift remains after apply");
    }

    if verified.core_version != verified.components_version {
        bail!(
            "Post-write verification failed: lockstep invariant violated (core.version != components.version)"
        );
    }

    print_success("Applied scoped UI version synchronization.");
    print_status("Updated files:");
    for rel in changed_files {
        println!("  - {}", rel);
    }

    Ok(0)
}

fn scoped_files(root: &Path) -> ScopeFiles {
    ScopeFiles {
        core: root.join(CORE_REL_PATH),
        components: root.join(COMPONENTS_REL_PATH),
        template: root.join(TEMPLATE_REL_PATH),
    }
}

fn load_state(scope: &ScopeFiles) -> Result<LoadedState> {
    let core_json = read_json(&scope.core)?;
    let components_json = read_json(&scope.components)?;
    let template_json = read_json(&scope.template)?;

    let core_version = parse_semver(
        &required_string(&core_json, CORE_VERSION_PTR, &scope.core)?,
        &scope.core,
        "version",
    )?;
    let components_version = parse_semver(
        &required_string(&components_json, COMPONENTS_VERSION_PTR, &scope.components)?,
        &scope.components,
        "version",
    )?;
    let components_peer_core = parse_caret_baseline(
        &required_string(
            &components_json,
            COMPONENTS_PEER_CORE_PTR,
            &scope.components,
        )?,
        &scope.components,
        "peerDependencies[@wavecraft/core]",
    )?;

    let template_dep_core = parse_caret_baseline(
        &required_string(&template_json, TEMPLATE_DEP_CORE_PTR, &scope.template)?,
        &scope.template,
        "dependencies[@wavecraft/core]",
    )?;
    let template_dep_components = parse_caret_baseline(
        &required_string(&template_json, TEMPLATE_DEP_COMPONENTS_PTR, &scope.template)?,
        &scope.template,
        "dependencies[@wavecraft/components]",
    )?;

    Ok(LoadedState {
        core_json,
        components_json,
        template_json,
        core_version,
        components_version,
        components_peer_core,
        template_dep_core,
        template_dep_components,
    })
}

fn compute_expected(loaded: &LoadedState, config: SyncUiVersionsConfig) -> Result<ExpectedState> {
    let target_version = std::cmp::max(
        loaded.core_version.clone(),
        loaded.components_version.clone(),
    );
    let transitions = [
        (
            format!("{}::version", CORE_REL_PATH),
            loaded.core_version.clone(),
            target_version.clone(),
        ),
        (
            format!("{}::version", COMPONENTS_REL_PATH),
            loaded.components_version.clone(),
            target_version.clone(),
        ),
        (
            format!("{}::peerDependencies[@wavecraft/core]", COMPONENTS_REL_PATH),
            loaded.components_peer_core.clone(),
            target_version.clone(),
        ),
        (
            format!("{}::dependencies[@wavecraft/core]", TEMPLATE_REL_PATH),
            loaded.template_dep_core.clone(),
            target_version.clone(),
        ),
        (
            format!("{}::dependencies[@wavecraft/components]", TEMPLATE_REL_PATH),
            loaded.template_dep_components.clone(),
            target_version.clone(),
        ),
    ];

    let mut violations = Vec::new();
    for (label, from, to) in transitions {
        let movement = classify_movement(&from, &to);
        if !movement_allowed(movement, config) {
            violations.push(format!(
                "{} requires {} movement: {} -> {}",
                label,
                movement_label(movement),
                from,
                to
            ));
        }
    }

    if !violations.is_empty() {
        let mut msg = String::from("Semver policy blocked required version movement:\n");
        for violation in violations {
            msg.push_str("  - ");
            msg.push_str(&violation);
            msg.push('\n');
        }
        msg.push_str(
            "Pass --allow-minor to allow minor updates, or --allow-major to allow major updates.",
        );
        bail!(msg);
    }

    Ok(ExpectedState {
        target_caret: format!("^{}", target_version),
        target_version,
    })
}

fn compute_drifts(loaded: &LoadedState, expected: &ExpectedState) -> Vec<Drift> {
    let expected_version = expected.target_version.to_string();
    let expected_caret = expected.target_caret.clone();

    let checks = [
        (
            CORE_REL_PATH,
            "version",
            loaded.core_version.to_string(),
            expected_version.clone(),
        ),
        (
            COMPONENTS_REL_PATH,
            "version",
            loaded.components_version.to_string(),
            expected_version,
        ),
        (
            COMPONENTS_REL_PATH,
            "peerDependencies[@wavecraft/core]",
            format!("^{}", loaded.components_peer_core),
            expected_caret.clone(),
        ),
        (
            TEMPLATE_REL_PATH,
            "dependencies[@wavecraft/core]",
            format!("^{}", loaded.template_dep_core),
            expected_caret.clone(),
        ),
        (
            TEMPLATE_REL_PATH,
            "dependencies[@wavecraft/components]",
            format!("^{}", loaded.template_dep_components),
            expected_caret,
        ),
    ];

    checks
        .into_iter()
        .filter_map(|(file, key, current, expected)| {
            if current == expected {
                None
            } else {
                Some(Drift {
                    file,
                    key,
                    current,
                    expected,
                })
            }
        })
        .collect()
}

fn drift_exists(drifts: &[Drift], file: &str, key: &str) -> bool {
    drifts
        .iter()
        .any(|drift| drift.file == file && drift.key == key)
}

fn mode_label(mode: SyncMode) -> &'static str {
    match mode {
        SyncMode::Check => "check",
        SyncMode::Apply => "apply",
    }
}

fn movement_label(movement: VersionMovement) -> &'static str {
    match movement {
        VersionMovement::None => "none",
        VersionMovement::Patch => "patch",
        VersionMovement::Minor => "minor",
        VersionMovement::Major => "major",
    }
}

fn classify_movement(from: &Version, to: &Version) -> VersionMovement {
    if from == to {
        return VersionMovement::None;
    }

    if from.major != to.major {
        return VersionMovement::Major;
    }

    if from.minor != to.minor {
        return VersionMovement::Minor;
    }

    VersionMovement::Patch
}

fn movement_allowed(movement: VersionMovement, config: SyncUiVersionsConfig) -> bool {
    match movement {
        VersionMovement::None => true,
        VersionMovement::Patch => true,
        VersionMovement::Minor => config.allow_minor || config.allow_major,
        VersionMovement::Major => config.allow_major,
    }
}

fn read_json(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest: {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON manifest: {}", path.display()))
}

fn required_string(value: &Value, pointer: &str, path: &Path) -> Result<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing required string key '{}' in {}",
                pointer,
                path.display()
            )
        })
}

fn parse_semver(raw: &str, path: &Path, key: &str) -> Result<Version> {
    Version::parse(raw).with_context(|| {
        format!(
            "Invalid semver value for {} in {}: '{}'",
            key,
            path.display(),
            raw
        )
    })
}

fn parse_caret_baseline(raw: &str, path: &Path, key: &str) -> Result<Version> {
    let Some(without_caret) = raw.strip_prefix('^') else {
        bail!(
            "Invalid range for {} in {}: '{}' (expected caret baseline like ^0.7.29)",
            key,
            path.display(),
            raw
        );
    };

    parse_semver(without_caret, path, key)
}

fn set_string(value: &mut Value, pointer: &str, replacement: String) -> Result<()> {
    let slot = value.pointer_mut(pointer).ok_or_else(|| {
        anyhow::anyhow!(
            "Failed to update key '{}': key is missing during scoped mutation",
            pointer
        )
    })?;

    *slot = Value::String(replacement);
    Ok(())
}

fn write_json_if_changed(path: &Path, json: &Value) -> Result<bool> {
    let current = fs::read_to_string(path).with_context(|| {
        format!(
            "Failed to read file for write comparison: {}",
            path.display()
        )
    })?;

    let mut next = serde_json::to_string_pretty(json)
        .with_context(|| format!("Failed to serialize JSON for {}", path.display()))?;
    next.push('\n');

    if current == next {
        return Ok(false);
    }

    fs::write(path, next).with_context(|| format!("Failed to write file: {}", path.display()))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn default_config(mode: SyncMode) -> SyncUiVersionsConfig {
        SyncUiVersionsConfig {
            mode,
            allow_minor: false,
            allow_major: false,
            verbose: false,
        }
    }

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

        let core_manifest = serde_json::json!({
            "name": "@wavecraft/core",
            "version": core_version,
            "description": "fixture"
        });

        let components_manifest = serde_json::json!({
            "name": "@wavecraft/components",
            "version": components_version,
            "description": "fixture",
            "peerDependencies": {
                "@wavecraft/core": components_peer_core,
                "react": "^18.0.0"
            }
        });

        let template_manifest = serde_json::json!({
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
    fn movement_classification_and_policy_gates() {
        let base = Version::parse("0.7.4").expect("valid semver expected");
        let patch = Version::parse("0.7.5").expect("valid semver expected");
        let minor = Version::parse("0.8.0").expect("valid semver expected");
        let major = Version::parse("1.0.0").expect("valid semver expected");

        assert_eq!(classify_movement(&base, &base), VersionMovement::None);
        assert_eq!(classify_movement(&base, &patch), VersionMovement::Patch);
        assert_eq!(classify_movement(&base, &minor), VersionMovement::Minor);
        assert_eq!(classify_movement(&base, &major), VersionMovement::Major);

        let strict = default_config(SyncMode::Check);
        assert!(movement_allowed(VersionMovement::Patch, strict));
        assert!(!movement_allowed(VersionMovement::Minor, strict));
        assert!(!movement_allowed(VersionMovement::Major, strict));

        let allow_minor = SyncUiVersionsConfig {
            allow_minor: true,
            ..strict
        };
        assert!(movement_allowed(VersionMovement::Minor, allow_minor));
        assert!(!movement_allowed(VersionMovement::Major, allow_minor));

        let allow_major = SyncUiVersionsConfig {
            allow_major: true,
            ..strict
        };
        assert!(movement_allowed(VersionMovement::Minor, allow_major));
        assert!(movement_allowed(VersionMovement::Major, allow_major));
    }

    #[test]
    fn parser_errors_on_invalid_semver_in_scoped_key() {
        let dir = TempDir::new().expect("failed to create temp directory");
        write_fixture_workspace(&dir, "not-a-version", "0.7.4", "^0.7.4", "^0.7.4", "^0.7.4");

        let result = run_at_root(dir.path(), default_config(SyncMode::Check));
        assert!(result.is_err());
        let msg = format!("{:#}", result.expect_err("expected invalid semver error"));
        assert!(msg.contains("Invalid semver value"));
        assert!(msg.contains("version"));
    }

    #[test]
    fn parser_errors_when_required_key_is_missing() {
        let dir = TempDir::new().expect("failed to create temp directory");
        write_fixture_workspace(&dir, "0.7.5", "0.7.5", "^0.7.5", "^0.7.5", "^0.7.5");

        let components_path = dir.path().join(COMPONENTS_REL_PATH);
        let mut components_manifest: Value = serde_json::from_str(
            &fs::read_to_string(&components_path)
                .expect("failed to read components fixture manifest"),
        )
        .expect("failed to parse components fixture manifest");

        let peer = components_manifest
            .pointer_mut("/peerDependencies")
            .and_then(Value::as_object_mut)
            .expect("peerDependencies object should exist");
        peer.remove("@wavecraft/core");

        fs::write(
            &components_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&components_manifest)
                    .expect("failed to serialize modified components manifest")
            ),
        )
        .expect("failed to write modified components manifest");

        let result = run_at_root(dir.path(), default_config(SyncMode::Check));
        assert!(result.is_err());
        let msg = format!("{:#}", result.expect_err("expected missing key error"));
        assert!(msg.contains("Missing required string key"));
    }

    #[test]
    fn check_mode_detects_patch_drift_without_mutation() {
        let dir = TempDir::new().expect("failed to create temp directory");
        write_fixture_workspace(&dir, "0.7.5", "0.7.4", "^0.7.0", "^0.7.1", "^0.7.1");

        let components_path = dir.path().join(COMPONENTS_REL_PATH);
        let template_path = dir.path().join(TEMPLATE_REL_PATH);
        let components_before =
            fs::read_to_string(&components_path).expect("failed to read components fixture");
        let template_before =
            fs::read_to_string(&template_path).expect("failed to read template fixture");

        let code = run_at_root(dir.path(), default_config(SyncMode::Check))
            .expect("check mode should return drift code");
        assert_eq!(code, 1);

        let components_after =
            fs::read_to_string(&components_path).expect("failed to read components fixture after");
        let template_after =
            fs::read_to_string(&template_path).expect("failed to read template fixture after");
        assert_eq!(components_before, components_after);
        assert_eq!(template_before, template_after);
    }

    #[test]
    fn apply_mode_is_idempotent_and_aligns_scoped_keys() {
        let dir = TempDir::new().expect("failed to create temp directory");
        write_fixture_workspace(&dir, "0.7.5", "0.7.4", "^0.7.0", "^0.7.1", "^0.7.1");

        let apply_code = run_at_root(dir.path(), default_config(SyncMode::Apply))
            .expect("apply mode should succeed");
        assert_eq!(apply_code, 0);

        let state = load_state(&scoped_files(dir.path())).expect("failed to reload synced state");
        assert_eq!(state.core_version.to_string(), "0.7.5");
        assert_eq!(state.components_version.to_string(), "0.7.5");
        assert_eq!(state.components_peer_core.to_string(), "0.7.5");
        assert_eq!(state.template_dep_core.to_string(), "0.7.5");
        assert_eq!(state.template_dep_components.to_string(), "0.7.5");

        let core_before_second = fs::read_to_string(dir.path().join(CORE_REL_PATH))
            .expect("read core before second run");
        let components_before_second = fs::read_to_string(dir.path().join(COMPONENTS_REL_PATH))
            .expect("read components before second run");
        let template_before_second = fs::read_to_string(dir.path().join(TEMPLATE_REL_PATH))
            .expect("read template before second run");

        let second_apply_code = run_at_root(dir.path(), default_config(SyncMode::Apply))
            .expect("second apply should be a no-op success");
        assert_eq!(second_apply_code, 0);

        let core_after_second =
            fs::read_to_string(dir.path().join(CORE_REL_PATH)).expect("read core after second run");
        let components_after_second = fs::read_to_string(dir.path().join(COMPONENTS_REL_PATH))
            .expect("read components after second run");
        let template_after_second = fs::read_to_string(dir.path().join(TEMPLATE_REL_PATH))
            .expect("read template after second run");

        assert_eq!(core_before_second, core_after_second);
        assert_eq!(components_before_second, components_after_second);
        assert_eq!(template_before_second, template_after_second);
    }

    #[test]
    fn default_policy_blocks_minor_movement() {
        let dir = TempDir::new().expect("failed to create temp directory");
        write_fixture_workspace(&dir, "0.8.0", "0.7.5", "^0.7.5", "^0.7.5", "^0.7.5");

        let result = run_at_root(dir.path(), default_config(SyncMode::Check));
        assert!(result.is_err());
        let msg = format!("{:#}", result.expect_err("expected policy error"));
        assert!(msg.contains("Semver policy blocked"));
        assert!(msg.contains("allow-minor"));
    }

    #[test]
    fn allow_minor_permits_minor_alignment() {
        let dir = TempDir::new().expect("failed to create temp directory");
        write_fixture_workspace(&dir, "0.8.0", "0.7.5", "^0.7.5", "^0.7.5", "^0.7.5");

        let config = SyncUiVersionsConfig {
            mode: SyncMode::Apply,
            allow_minor: true,
            allow_major: false,
            verbose: false,
        };

        let code = run_at_root(dir.path(), config).expect("apply with --allow-minor should pass");
        assert_eq!(code, 0);

        let state = load_state(&scoped_files(dir.path())).expect("failed to reload after apply");
        assert_eq!(state.core_version.to_string(), "0.8.0");
        assert_eq!(state.components_version.to_string(), "0.8.0");
        assert_eq!(state.components_peer_core.to_string(), "0.8.0");
        assert_eq!(state.template_dep_core.to_string(), "0.8.0");
        assert_eq!(state.template_dep_components.to_string(), "0.8.0");
    }

    #[test]
    fn default_policy_blocks_major_movement() {
        let dir = TempDir::new().expect("failed to create temp directory");
        write_fixture_workspace(&dir, "1.0.0", "0.7.5", "^0.7.5", "^0.7.5", "^0.7.5");

        let result = run_at_root(dir.path(), default_config(SyncMode::Check));
        assert!(result.is_err());
        let msg = format!("{:#}", result.expect_err("expected policy error"));
        assert!(msg.contains("Semver policy blocked"));
        assert!(msg.contains("allow-major"));
    }

    #[test]
    fn allow_major_permits_major_alignment() {
        let dir = TempDir::new().expect("failed to create temp directory");
        write_fixture_workspace(&dir, "1.0.0", "0.7.5", "^0.7.5", "^0.7.5", "^0.7.5");

        let config = SyncUiVersionsConfig {
            mode: SyncMode::Apply,
            allow_minor: false,
            allow_major: true,
            verbose: false,
        };

        let code = run_at_root(dir.path(), config).expect("apply with --allow-major should pass");
        assert_eq!(code, 0);

        let state = load_state(&scoped_files(dir.path())).expect("failed to reload after apply");
        assert_eq!(state.core_version.to_string(), "1.0.0");
        assert_eq!(state.components_version.to_string(), "1.0.0");
        assert_eq!(state.components_peer_core.to_string(), "1.0.0");
        assert_eq!(state.template_dep_core.to_string(), "1.0.0");
        assert_eq!(state.template_dep_components.to_string(), "1.0.0");
    }

    #[test]
    fn strict_scope_preserves_unrelated_keys() {
        let dir = TempDir::new().expect("failed to create temp directory");
        write_fixture_workspace(&dir, "0.7.5", "0.7.4", "^0.7.0", "^0.7.1", "^0.7.1");

        let components_path = dir.path().join(COMPONENTS_REL_PATH);
        let mut components_manifest: Value = serde_json::from_str(
            &fs::read_to_string(&components_path)
                .expect("failed to read components fixture manifest"),
        )
        .expect("failed to parse components fixture manifest");

        let object = components_manifest
            .as_object_mut()
            .expect("components manifest should be object");
        object.insert(
            "customUnrelatedKey".to_string(),
            Value::String("must-stay-unchanged".to_string()),
        );

        fs::write(
            &components_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&components_manifest)
                    .expect("failed to serialize components with custom key")
            ),
        )
        .expect("failed to write modified components manifest");

        let code = run_at_root(dir.path(), default_config(SyncMode::Apply))
            .expect("apply mode should succeed");
        assert_eq!(code, 0);

        let after: Value = serde_json::from_str(
            &fs::read_to_string(&components_path)
                .expect("failed to read components manifest after apply"),
        )
        .expect("failed to parse components manifest after apply");

        let value = after
            .pointer("/customUnrelatedKey")
            .and_then(Value::as_str)
            .expect("custom unrelated key should remain present");
        assert_eq!(value, "must-stay-unchanged");
    }
}
