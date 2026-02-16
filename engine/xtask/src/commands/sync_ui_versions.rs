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

#[derive(Debug, Clone)]
struct ScopedStringUpdate {
    pointer: &'static str,
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

    let mut core_updates = Vec::new();
    let mut components_updates = Vec::new();
    let mut template_updates = Vec::new();

    if core_version_drift {
        core_updates.push(ScopedStringUpdate {
            pointer: CORE_VERSION_PTR,
            current: loaded.core_version.to_string(),
            expected: expected.target_version.to_string(),
        });
        set_string(
            &mut loaded.core_json,
            CORE_VERSION_PTR,
            expected.target_version.to_string(),
        )?;
    }
    if components_version_drift {
        components_updates.push(ScopedStringUpdate {
            pointer: COMPONENTS_VERSION_PTR,
            current: loaded.components_version.to_string(),
            expected: expected.target_version.to_string(),
        });
        set_string(
            &mut loaded.components_json,
            COMPONENTS_VERSION_PTR,
            expected.target_version.to_string(),
        )?;
    }
    if components_peer_core_drift {
        components_updates.push(ScopedStringUpdate {
            pointer: COMPONENTS_PEER_CORE_PTR,
            current: format!("^{}", loaded.components_peer_core),
            expected: expected.target_caret.clone(),
        });
        set_string(
            &mut loaded.components_json,
            COMPONENTS_PEER_CORE_PTR,
            expected.target_caret.clone(),
        )?;
    }
    if template_dep_core_drift {
        template_updates.push(ScopedStringUpdate {
            pointer: TEMPLATE_DEP_CORE_PTR,
            current: format!("^{}", loaded.template_dep_core),
            expected: expected.target_caret.clone(),
        });
        set_string(
            &mut loaded.template_json,
            TEMPLATE_DEP_CORE_PTR,
            expected.target_caret.clone(),
        )?;
    }
    if template_dep_components_drift {
        template_updates.push(ScopedStringUpdate {
            pointer: TEMPLATE_DEP_COMPONENTS_PTR,
            current: format!("^{}", loaded.template_dep_components),
            expected: expected.target_caret.clone(),
        });
        set_string(
            &mut loaded.template_json,
            TEMPLATE_DEP_COMPONENTS_PTR,
            expected.target_caret,
        )?;
    }

    let mut changed_files = Vec::new();
    if core_version_drift
        && write_scoped_string_updates_with_fallback(&scope.core, &core_updates, &loaded.core_json)?
    {
        changed_files.push(CORE_REL_PATH);
    }
    if (components_version_drift || components_peer_core_drift)
        && write_scoped_string_updates_with_fallback(
            &scope.components,
            &components_updates,
            &loaded.components_json,
        )?
    {
        changed_files.push(COMPONENTS_REL_PATH);
    }
    if (template_dep_core_drift || template_dep_components_drift)
        && write_scoped_string_updates_with_fallback(
            &scope.template,
            &template_updates,
            &loaded.template_json,
        )?
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

fn write_scoped_string_updates_with_fallback(
    path: &Path,
    updates: &[ScopedStringUpdate],
    fallback_json: &Value,
) -> Result<bool> {
    if updates.is_empty() {
        return Ok(false);
    }

    match write_scoped_string_updates(path, updates) {
        Ok(changed) => Ok(changed),
        Err(_) => write_json_if_changed(path, fallback_json),
    }
}

fn write_scoped_string_updates(path: &Path, updates: &[ScopedStringUpdate]) -> Result<bool> {
    let current = fs::read_to_string(path).with_context(|| {
        format!(
            "Failed to read file for scoped write comparison: {}",
            path.display()
        )
    })?;

    let mut next = current.clone();
    for update in updates {
        apply_scoped_string_update(&mut next, update)?;
    }

    if current == next {
        return Ok(false);
    }

    fs::write(path, next).with_context(|| format!("Failed to write file: {}", path.display()))?;
    Ok(true)
}

fn apply_scoped_string_update(content: &mut String, update: &ScopedStringUpdate) -> Result<()> {
    let segments = decode_json_pointer(update.pointer)?;
    let (leaf_key, parent_path) = segments
        .split_last()
        .ok_or_else(|| anyhow::anyhow!("Invalid empty JSON pointer: {}", update.pointer))?;

    let scope = find_object_scope(content, parent_path)
        .ok_or_else(|| anyhow::anyhow!("Failed to locate parent object for {}", update.pointer))?;

    let (value_start, value_end) =
        find_member_value_span(content, scope, leaf_key).ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to locate key '{}' while applying scoped update {}",
                leaf_key,
                update.pointer
            )
        })?;

    let current_token = &content[value_start..value_end];
    let parsed_current: String = serde_json::from_str(current_token).with_context(|| {
        format!(
            "Expected string value at {} but found '{}'",
            update.pointer, current_token
        )
    })?;

    if parsed_current != update.current {
        bail!(
            "Scoped update precondition failed at {}: expected current '{}', found '{}'",
            update.pointer,
            update.current,
            parsed_current
        );
    }

    let replacement = serde_json::to_string(&update.expected).with_context(|| {
        format!(
            "Failed to serialize replacement value for {}",
            update.pointer
        )
    })?;
    content.replace_range(value_start..value_end, &replacement);
    Ok(())
}

fn decode_json_pointer(pointer: &str) -> Result<Vec<String>> {
    if pointer.is_empty() {
        return Ok(Vec::new());
    }

    if !pointer.starts_with('/') {
        bail!("Invalid JSON pointer '{}': must start with '/'", pointer);
    }

    Ok(pointer[1..]
        .split('/')
        .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
        .collect())
}

fn find_object_scope(content: &str, path: &[String]) -> Option<(usize, usize)> {
    let mut scope = find_root_object_scope(content)?;
    for segment in path {
        let (value_start, value_end) = find_member_value_span(content, scope, segment)?;
        if content.as_bytes().get(value_start) != Some(&b'{') {
            return None;
        }
        scope = (value_start, value_end);
    }
    Some(scope)
}

fn find_root_object_scope(content: &str) -> Option<(usize, usize)> {
    let mut idx = skip_whitespace(content, 0);
    if content.as_bytes().get(idx) != Some(&b'{') {
        return None;
    }
    let end = parse_json_value_end(content, idx)?;
    idx = skip_whitespace(content, end);
    if idx != content.len() {
        return None;
    }
    Some((content.find('{')?, end))
}

fn find_member_value_span(
    content: &str,
    scope: (usize, usize),
    target_key: &str,
) -> Option<(usize, usize)> {
    let bytes = content.as_bytes();
    let (start, end) = scope;
    if bytes.get(start) != Some(&b'{') {
        return None;
    }

    let mut idx = skip_whitespace(content, start + 1);
    if idx >= end {
        return None;
    }

    if bytes.get(idx) == Some(&b'}') {
        return None;
    }

    loop {
        if bytes.get(idx) != Some(&b'"') {
            return None;
        }
        let key_end = parse_json_string_end(content, idx)?;
        let key: String = serde_json::from_str(&content[idx..key_end]).ok()?;

        idx = skip_whitespace(content, key_end);
        if bytes.get(idx) != Some(&b':') {
            return None;
        }
        idx += 1;
        idx = skip_whitespace(content, idx);

        let value_start = idx;
        let value_end = parse_json_value_end(content, value_start)?;

        if key == target_key {
            return Some((value_start, value_end));
        }

        idx = skip_whitespace(content, value_end);
        match bytes.get(idx) {
            Some(b',') => {
                idx += 1;
                idx = skip_whitespace(content, idx);
            }
            Some(b'}') => return None,
            _ => return None,
        }
    }
}

fn parse_json_value_end(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let ch = *bytes.get(start)?;

    match ch {
        b'"' => parse_json_string_end(content, start),
        b'{' => parse_balanced_json_end(content, start, b'{', b'}'),
        b'[' => parse_balanced_json_end(content, start, b'[', b']'),
        b't' => content[start..].starts_with("true").then_some(start + 4),
        b'f' => content[start..].starts_with("false").then_some(start + 5),
        b'n' => content[start..].starts_with("null").then_some(start + 4),
        b'-' | b'0'..=b'9' => parse_number_end(content, start),
        _ => None,
    }
}

fn parse_json_string_end(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    if bytes.get(start) != Some(&b'"') {
        return None;
    }

    let mut idx = start + 1;
    while idx < bytes.len() {
        match bytes[idx] {
            b'"' => return Some(idx + 1),
            b'\\' => {
                idx += 2;
            }
            _ => {
                idx += 1;
            }
        }
    }

    None
}

fn parse_balanced_json_end(content: &str, start: usize, open: u8, close: u8) -> Option<usize> {
    let bytes = content.as_bytes();
    if bytes.get(start) != Some(&open) {
        return None;
    }

    let mut idx = start;
    let mut depth = 0usize;
    while idx < bytes.len() {
        match bytes[idx] {
            b'"' => {
                idx = parse_json_string_end(content, idx)?;
            }
            ch if ch == open => {
                depth += 1;
                idx += 1;
            }
            ch if ch == close => {
                depth -= 1;
                idx += 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {
                idx += 1;
            }
        }
    }

    None
}

fn parse_number_end(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut idx = start;

    if bytes.get(idx) == Some(&b'-') {
        idx += 1;
    }

    match bytes.get(idx) {
        Some(b'0') => {
            idx += 1;
        }
        Some(b'1'..=b'9') => {
            idx += 1;
            while matches!(bytes.get(idx), Some(b'0'..=b'9')) {
                idx += 1;
            }
        }
        _ => return None,
    }

    if bytes.get(idx) == Some(&b'.') {
        idx += 1;
        if !matches!(bytes.get(idx), Some(b'0'..=b'9')) {
            return None;
        }
        while matches!(bytes.get(idx), Some(b'0'..=b'9')) {
            idx += 1;
        }
    }

    if matches!(bytes.get(idx), Some(b'e' | b'E')) {
        idx += 1;
        if matches!(bytes.get(idx), Some(b'+' | b'-')) {
            idx += 1;
        }
        if !matches!(bytes.get(idx), Some(b'0'..=b'9')) {
            return None;
        }
        while matches!(bytes.get(idx), Some(b'0'..=b'9')) {
            idx += 1;
        }
    }

    Some(idx)
}

fn skip_whitespace(content: &str, mut idx: usize) -> usize {
    let bytes = content.as_bytes();
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use xtask::test_support::{SyncUiFixtureVersions, write_sync_ui_fixture_workspace};

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

    #[test]
    fn apply_mode_preserves_non_scoped_json_layout_with_scoped_replacements() {
        let dir = TempDir::new().expect("failed to create temp directory");

        fs::create_dir_all(
            dir.path()
                .join(CORE_REL_PATH)
                .parent()
                .expect("core package.json path should have parent"),
        )
        .expect("failed to create core fixture directory");
        fs::create_dir_all(
            dir.path()
                .join(COMPONENTS_REL_PATH)
                .parent()
                .expect("components package.json path should have parent"),
        )
        .expect("failed to create components fixture directory");
        fs::create_dir_all(
            dir.path()
                .join(TEMPLATE_REL_PATH)
                .parent()
                .expect("template package.json path should have parent"),
        )
        .expect("failed to create template fixture directory");

        let core_raw =
            "{\"name\":\"@wavecraft/core\",\"version\":\"0.7.5\",\"custom\":{\"a\":1,\"b\":2}}\n";
        let components_raw = "{\"name\":\"@wavecraft/components\",\"version\":\"0.7.4\",\"peerDependencies\":{\"react\":\"^18.0.0\",\"@wavecraft/core\":\"^0.7.0\"},\"custom\":{\"keep\":true,\"arr\":[3,2,1]}}\n";
        let template_raw = "{\"name\":\"fixture-template-ui\",\"private\":true,\"version\":\"0.1.0\",\"dependencies\":{\"react\":\"^18.3.1\",\"@wavecraft/core\":\"^0.7.1\",\"@wavecraft/components\":\"^0.7.1\"},\"z\":0}\n";

        fs::write(dir.path().join(CORE_REL_PATH), core_raw).expect("failed to write core raw");
        fs::write(dir.path().join(COMPONENTS_REL_PATH), components_raw)
            .expect("failed to write components raw");
        fs::write(dir.path().join(TEMPLATE_REL_PATH), template_raw)
            .expect("failed to write template raw");

        let code = run_at_root(dir.path(), default_config(SyncMode::Apply))
            .expect("apply mode should succeed");
        assert_eq!(code, 0);

        let core_after =
            fs::read_to_string(dir.path().join(CORE_REL_PATH)).expect("failed to read core");
        let components_after = fs::read_to_string(dir.path().join(COMPONENTS_REL_PATH))
            .expect("failed to read components");
        let template_after = fs::read_to_string(dir.path().join(TEMPLATE_REL_PATH))
            .expect("failed to read template");

        assert_eq!(
            core_after,
            "{\"name\":\"@wavecraft/core\",\"version\":\"0.7.5\",\"custom\":{\"a\":1,\"b\":2}}\n"
        );
        assert_eq!(
            components_after,
            "{\"name\":\"@wavecraft/components\",\"version\":\"0.7.5\",\"peerDependencies\":{\"react\":\"^18.0.0\",\"@wavecraft/core\":\"^0.7.5\"},\"custom\":{\"keep\":true,\"arr\":[3,2,1]}}\n"
        );
        assert_eq!(
            template_after,
            "{\"name\":\"fixture-template-ui\",\"private\":true,\"version\":\"0.1.0\",\"dependencies\":{\"react\":\"^18.3.1\",\"@wavecraft/core\":\"^0.7.5\",\"@wavecraft/components\":\"^0.7.5\"},\"z\":0}\n"
        );
    }
}
