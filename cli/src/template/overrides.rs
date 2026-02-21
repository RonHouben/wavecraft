use anyhow::{Context, Result};
use std::fs;

use crate::template::variables::TemplateVariables;

use super::{dependency_rewrites, tsconfig_paths};

/// SDK crates under `engine/crates/` that need path replacement in local dev mode.
const SDK_CRATES: [&str; 5] = [
    "wavecraft-core",
    "wavecraft-protocol",
    "wavecraft-dsp",
    "wavecraft-bridge",
    "wavecraft-metering",
];

pub(super) fn apply_post_processing(content: &str, vars: &TemplateVariables) -> Result<String> {
    if vars.local_dev.is_some() {
        return apply_local_dev_overrides(content, vars);
    }

    Ok(content.to_string())
}

/// Replaces git dependencies with local path dependencies for SDK crates.
/// This is used when --local-dev is specified to allow testing against
/// a local checkout of the Wavecraft SDK.
fn apply_local_dev_overrides(content: &str, vars: &TemplateVariables) -> Result<String> {
    let Some(sdk_path) = &vars.local_dev else {
        return Ok(content.to_string());
    };

    // Canonicalize the SDK path to get absolute path
    let sdk_path = fs::canonicalize(sdk_path)
        .with_context(|| format!("Invalid local-dev path: {}", sdk_path.display()))?;

    // Keep sequencing stable for behavior preservation:
    // dependency rewrites -> npm rewrites -> tsconfig injection.
    let mut result =
        dependency_rewrites::apply_dependency_rewrites(content, &sdk_path, &SDK_CRATES)?;
    result = dependency_rewrites::apply_npm_dependency_rewrites(&result, &sdk_path)?;

    // Inject TypeScript path mappings for SDK mode so TS language services
    // resolve @wavecraft/* to local monorepo sources (matching Vite aliases).
    result = tsconfig_paths::inject_tsconfig_paths_if_needed(&result)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    fn make_vars(local_dev: Option<PathBuf>) -> TemplateVariables {
        TemplateVariables::new(
            "test-plugin".to_string(),
            "Test Vendor".to_string(),
            "test@example.com".to_string(),
            "https://test.com".to_string(),
            "v0.9.0".to_string(),
            local_dev,
        )
    }

    fn setup_sdk_fixture(include_ui_packages: bool) -> (TempDir, PathBuf, PathBuf) {
        // Create a temp directory simulating the SDK layout:
        //   {root}/engine/crates/  ← sdk_path
        //   {root}/dev-server/     ← wavecraft-dev-server location
        let temp = tempdir().expect("failed to create temp dir");
        let sdk_root = temp.path().to_path_buf();
        let sdk_path = sdk_root.join("engine").join("crates");
        fs::create_dir_all(&sdk_path).expect("failed to create sdk crates dir");

        for crate_name in &SDK_CRATES {
            fs::create_dir_all(sdk_path.join(crate_name)).expect("failed to create sdk crate dir");
        }
        fs::create_dir_all(sdk_path.join("wavecraft-nih_plug"))
            .expect("failed to create wavecraft-nih_plug dir");
        fs::create_dir_all(sdk_root.join("dev-server")).expect("failed to create dev-server dir");

        if include_ui_packages {
            fs::create_dir_all(sdk_root.join("ui/packages/core"))
                .expect("failed to create ui/packages/core");
            fs::create_dir_all(sdk_root.join("ui/packages/components"))
                .expect("failed to create ui/packages/components");
        }

        (temp, sdk_root, sdk_path)
    }

    #[test]
    fn test_apply_local_dev_overrides() {
        let content = r#"
[dependencies]
# Main SDK dependency with Cargo rename
wavecraft = { package = "wavecraft-nih_plug", git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }

# Individual SDK crates
wavecraft-core = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
wavecraft-protocol = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
wavecraft-dsp = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
wavecraft-bridge = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
wavecraft-metering = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
wavecraft-dev-server = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0", features = ["audio"], optional = true }
"#;

        let (_temp, _sdk_root, sdk_path) = setup_sdk_fixture(false);
        let vars = make_vars(Some(sdk_path.clone()));

        let result = apply_local_dev_overrides(content, &vars).unwrap();

        // Verify main wavecraft dependency was replaced
        assert!(
            result.contains(r#"wavecraft = { package = "wavecraft-nih_plug", path ="#),
            "Expected main wavecraft dependency to have path, got: {}",
            result
        );
        assert!(
            !result.contains(r#"wavecraft = { package = "wavecraft-nih_plug", git ="#),
            "Expected main wavecraft git dep to be removed, got: {}",
            result
        );

        // Verify all individual SDK crate git deps were replaced with path deps
        for crate_name in &SDK_CRATES {
            assert!(
                result.contains(&format!("{} = {{ path =", crate_name)),
                "Expected {} to have path dependency, got: {}",
                crate_name,
                result
            );
            assert!(
                !result.contains(&format!("{} = {{ git =", crate_name)),
                "Expected {} git dep to be removed, got: {}",
                crate_name,
                result
            );
        }

        // Verify extra attributes (features, optional) are preserved for dev-server
        // dev-server path should point to {sdk_root}/dev-server, not {sdk_path}/wavecraft-dev-server
        assert!(
            result.contains("wavecraft-dev-server = { path = \""),
            "Expected wavecraft-dev-server to use path dependency, got: {}",
            result
        );
        assert!(
            result.contains("/dev-server\""),
            "Expected wavecraft-dev-server path to end with /dev-server, got: {}",
            result
        );
        assert!(
            !result.contains("/wavecraft-dev-server\""),
            "Expected wavecraft-dev-server path to NOT be under engine/crates/, got: {}",
            result
        );
        assert!(
            result.contains("features = [\"audio\"]"),
            "Expected wavecraft-dev-server features to be preserved"
        );
        assert!(
            result.contains("optional = true"),
            "Expected wavecraft-dev-server optional flag to be preserved"
        );
    }

    #[test]
    fn test_apply_local_dev_overrides_no_local_dev() {
        let content = r#"wavecraft-core = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }"#;

        let vars = make_vars(None); // No local_dev

        let result = apply_local_dev_overrides(content, &vars).unwrap();

        // Content should be unchanged when local_dev is None
        assert_eq!(result, content);
    }

    #[test]
    fn test_apply_local_dev_overrides_invalid_path() {
        let content =
            "wavecraft-core = { git = \"https://github.com/RonHouben/wavecraft\", tag = \"v0.7.0\" }";

        let vars = make_vars(Some(PathBuf::from("/nonexistent/path/that/does/not/exist")));

        // Should fail because the path doesn't exist
        let result = apply_local_dev_overrides(content, &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_local_dev_overrides_injects_tsconfig_paths() {
        let content = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,

    /* Bundler mode */
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",

    /* Linting */
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}"#;

        let (_temp, _sdk_root, sdk_path) = setup_sdk_fixture(false);
        let vars = make_vars(Some(sdk_path));

        let result = apply_local_dev_overrides(content, &vars).unwrap();

        assert!(
            result.contains(r#""baseUrl": ".""#),
            "Expected baseUrl in result:\n{}",
            result
        );
        assert!(
            result.contains(r#""@wavecraft/core": ["../../ui/packages/core/src/index.ts"]"#),
            "Expected @wavecraft/core path mapping:\n{}",
            result
        );
        assert!(
            result.contains(
                r#""@wavecraft/components": ["../../ui/packages/components/src/index.ts"]"#
            ),
            "Expected @wavecraft/components path mapping:\n{}",
            result
        );
        assert!(
            result.contains(r#""@wavecraft/core/*": ["../../ui/packages/core/src/*"]"#),
            "Expected @wavecraft/core/* wildcard path:\n{}",
            result
        );
        assert!(
            result.contains(r#""@wavecraft/components/*": ["../../ui/packages/components/src/*"]"#),
            "Expected @wavecraft/components/* wildcard path:\n{}",
            result
        );

        assert!(
            result.contains("/* Bundler mode */"),
            "Expected JSONC comments to be preserved:\n{}",
            result
        );
        assert!(
            result.contains("/* Linting */"),
            "Expected JSONC comments to be preserved:\n{}",
            result
        );
        assert!(
            result.contains("/* SDK development"),
            "Expected SDK development comment:\n{}",
            result
        );
    }

    #[test]
    fn test_apply_local_dev_overrides_injects_paths_without_primary_anchor() {
        let content = r#"{
  "compilerOptions": {
    "strict": true
  },
  "include": ["src"]
}"#;

        let (_temp, _sdk_root, sdk_path) = setup_sdk_fixture(false);
        let vars = make_vars(Some(sdk_path));

        let result = apply_local_dev_overrides(content, &vars).unwrap();

        assert!(
            result.contains(r#""baseUrl": ".""#),
            "Expected baseUrl injection with fallback anchor:\n{}",
            result
        );
        assert!(
            result.contains(r#""@wavecraft/core": ["../../ui/packages/core/src/index.ts"]"#),
            "Expected @wavecraft/core mapping with fallback anchor:\n{}",
            result
        );
    }

    #[test]
    fn test_apply_local_dev_overrides_injects_paths_with_trailing_comma_before_next_property() {
        let content = r#"{
  "compilerOptions": {
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "types": ["node"]
  }
}"#;

        let (_temp, _sdk_root, sdk_path) = setup_sdk_fixture(false);
        let vars = make_vars(Some(sdk_path));

        let result = apply_local_dev_overrides(content, &vars).unwrap();

        assert!(
            result.contains("\"@wavecraft/components/*\": [\"../../ui/packages/components/src/*\"]\n        },\n    \"allowSyntheticDefaultImports\""),
            "Expected trailing comma after injected paths block before following property:\n{}",
            result
        );
        assert!(
            !result.contains("\"@wavecraft/components/*\": [\"../../ui/packages/components/src/*\"]\n        }\n    \"allowSyntheticDefaultImports\""),
            "Injected paths block must not be adjacent to next property without comma:\n{}",
            result
        );
    }

    #[test]
    fn test_apply_local_dev_overrides_no_tsconfig_paths_without_local_dev() {
        let content = r#"{
  "compilerOptions": {
    "noFallthroughCasesInSwitch": true
  }
}"#;

        let vars = make_vars(None);

        let result = apply_local_dev_overrides(content, &vars).unwrap();
        assert_eq!(
            result, content,
            "Content should be unchanged without local_dev"
        );
    }

    #[test]
    fn test_apply_local_dev_overrides_rewrites_npm_sdk_dependencies() {
        let content = r#"{
  "dependencies": {
    "@wavecraft/core": "^0.7.1",
    "@wavecraft/components": "^0.7.1",
    "react": "^18.3.1"
  }
}"#;

        let (_temp, _sdk_root, sdk_path) = setup_sdk_fixture(true);
        let vars = make_vars(Some(sdk_path));

        let result = apply_local_dev_overrides(content, &vars).unwrap();

        assert!(
            result.contains("\"@wavecraft/core\": \"file:"),
            "Expected @wavecraft/core to be rewritten as file dependency:\n{}",
            result
        );
        assert!(
            result.contains("\"@wavecraft/components\": \"file:"),
            "Expected @wavecraft/components to be rewritten as file dependency:\n{}",
            result
        );
        assert!(
            !result.contains("\"@wavecraft/core\": \"^"),
            "Expected @wavecraft/core version dependency to be removed:\n{}",
            result
        );
        assert!(
            !result.contains("\"@wavecraft/components\": \"^"),
            "Expected @wavecraft/components version dependency to be removed:\n{}",
            result
        );
    }

    #[test]
    fn test_apply_local_dev_overrides_adds_base_url_when_paths_exist() {
        let content = r#"{
  "compilerOptions": {
    "strict": true,
    "paths": {
      "@wavecraft/core": ["../../ui/packages/core/src/index.ts"],
      "@wavecraft/components": ["../../ui/packages/components/src/index.ts"]
    }
  }
}"#;

        let (_temp, _sdk_root, sdk_path) = setup_sdk_fixture(false);
        let vars = make_vars(Some(sdk_path));

        let result = apply_local_dev_overrides(content, &vars).unwrap();

        assert!(
            result.contains(r#""baseUrl": ".""#),
            "Expected baseUrl to be injected when paths already exist:\n{}",
            result
        );

        assert_eq!(
            result.matches("\"baseUrl\"").count(),
            1,
            "Expected exactly one baseUrl entry:\n{}",
            result
        );
    }

    #[test]
    fn test_apply_local_dev_overrides_ignores_non_tsconfig_files() {
        let content = r#"[package]
name = "test-plugin"
version = "0.1.0"
"#;

        let (_temp, _sdk_root, sdk_path) = setup_sdk_fixture(false);
        let vars = make_vars(Some(sdk_path));

        let result = apply_local_dev_overrides(content, &vars).unwrap();

        assert!(
            !result.contains("baseUrl"),
            "Non-tsconfig content should not have baseUrl injected:\n{}",
            result
        );
        assert!(
            !result.contains("\"paths\""),
            "Non-tsconfig content should not have paths injected:\n{}",
            result
        );
    }
}
