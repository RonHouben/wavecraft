//! Test support helpers shared by unit and integration tests.

use anyhow::{Context, Result};
use serde_json::json;
use std::fs;
use std::path::Path;

/// Relative path to the core package manifest.
pub const CORE_REL_PATH: &str = "ui/packages/core/package.json";
/// Relative path to the components package manifest.
pub const COMPONENTS_REL_PATH: &str = "ui/packages/components/package.json";
/// Relative path to the sdk-template UI manifest.
pub const TEMPLATE_REL_PATH: &str = "sdk-template/ui/package.json";

/// Fixture values used by sync-ui-versions tests.
#[derive(Debug, Clone, Copy)]
pub struct SyncUiFixtureVersions<'a> {
    pub core_version: &'a str,
    pub components_version: &'a str,
    pub components_peer_core: &'a str,
    pub template_core_dep: &'a str,
    pub template_components_dep: &'a str,
}

/// Write the three scoped package.json fixture files used by sync-ui-versions tests.
pub fn write_sync_ui_fixture_workspace(
    root: &Path,
    versions: SyncUiFixtureVersions<'_>,
) -> Result<()> {
    let core_path = root.join(CORE_REL_PATH);
    let components_path = root.join(COMPONENTS_REL_PATH);
    let template_path = root.join(TEMPLATE_REL_PATH);

    fs::create_dir_all(
        core_path
            .parent()
            .expect("core package.json path should have parent"),
    )
    .with_context(|| format!("failed to create fixture dir for {}", core_path.display()))?;

    fs::create_dir_all(
        components_path
            .parent()
            .expect("components package.json path should have parent"),
    )
    .with_context(|| {
        format!(
            "failed to create fixture dir for {}",
            components_path.display()
        )
    })?;

    fs::create_dir_all(
        template_path
            .parent()
            .expect("template package.json path should have parent"),
    )
    .with_context(|| {
        format!(
            "failed to create fixture dir for {}",
            template_path.display()
        )
    })?;

    let core_manifest = json!({
        "name": "@wavecraft/core",
        "version": versions.core_version,
        "description": "fixture"
    });

    let components_manifest = json!({
        "name": "@wavecraft/components",
        "version": versions.components_version,
        "description": "fixture",
        "peerDependencies": {
            "@wavecraft/core": versions.components_peer_core,
            "react": "^18.0.0"
        }
    });

    let template_manifest = json!({
        "name": "fixture-template-ui",
        "private": true,
        "version": "0.1.0",
        "dependencies": {
            "@wavecraft/core": versions.template_core_dep,
            "@wavecraft/components": versions.template_components_dep,
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
    .with_context(|| format!("failed to write {}", core_path.display()))?;

    fs::write(
        &components_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&components_manifest)
                .expect("failed to serialize components fixture manifest")
        ),
    )
    .with_context(|| format!("failed to write {}", components_path.display()))?;

    fs::write(
        &template_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&template_manifest)
                .expect("failed to serialize template fixture manifest")
        ),
    )
    .with_context(|| format!("failed to write {}", template_path.display()))?;

    Ok(())
}
