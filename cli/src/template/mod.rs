pub mod variables;

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use regex::Regex;
use std::fs;
use std::path::Path;

use crate::template::variables::TemplateVariables;

// The canonical template lives in sdk-template/ at the repository root and is
// staged via build.rs so build artifacts (target/, node_modules/, dist/) are
// excluded before being embedded into the CLI binary.
static TEMPLATE_DIR: Dir = include_dir!("$OUT_DIR/sdk-template-clean");

/// Extracts the embedded template to the target directory and applies variable replacement.
pub fn extract_template(target_dir: &Path, vars: &TemplateVariables) -> Result<()> {
    extract_dir(&TEMPLATE_DIR, target_dir, vars)
}

fn extract_dir(dir: &Dir, target_dir: &Path, vars: &TemplateVariables) -> Result<()> {
    fs::create_dir_all(target_dir)
        .with_context(|| format!("Failed to create directory: {}", target_dir.display()))?;

    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(subdir) => {
                // Skip build artifacts and dependencies
                let dir_name = subdir.path().file_name().with_context(|| {
                    format!("Invalid directory path: {}", subdir.path().display())
                })?;
                let dir_name_str = dir_name.to_string_lossy();

                if dir_name_str == "target"
                    || dir_name_str == "node_modules"
                    || dir_name_str == "dist"
                {
                    continue; // Skip these directories
                }

                let subdir_path = target_dir.join(dir_name);
                extract_dir(subdir, &subdir_path, vars)?;
            }
            include_dir::DirEntry::File(file) => {
                // Skip binary files and lock files
                let file_name = file
                    .path()
                    .file_name()
                    .with_context(|| format!("Invalid file path: {}", file.path().display()))?;
                let file_name_str = file_name.to_string_lossy();

                if file_name_str == "Cargo.lock"
                    || file_name_str.ends_with(".dylib")
                    || file_name_str.ends_with(".so")
                    || file_name_str.ends_with(".dll")
                    || file_name_str == ".DS_Store"
                {
                    continue; // Skip these files
                }

                // Handle .template files: rename back to original (e.g., Cargo.toml.template -> Cargo.toml)
                // These are renamed to avoid cargo treating the template as a crate during packaging.
                let output_name = if file_name_str.ends_with(".template") {
                    file_name_str
                        .strip_suffix(".template")
                        .expect("filename should have .template suffix after filter")
                        .to_string()
                } else {
                    file_name_str.to_string()
                };
                let file_path = target_dir.join(&output_name);

                // Only process text files
                if let Some(content) = file.contents_utf8() {
                    let mut processed = vars.apply(content).with_context(|| {
                        format!("Failed to process template: {}", file.path().display())
                    })?;

                    // Post-process for local dev mode (replace git deps with path deps)
                    if vars.local_dev.is_some() {
                        processed = apply_local_dev_overrides(&processed, vars)?;
                    }

                    fs::write(&file_path, processed).with_context(|| {
                        format!("Failed to write file: {}", file_path.display())
                    })?;
                } else {
                    // Skip non-UTF8 files (likely binaries)
                    continue;
                }
            }
        }
    }

    Ok(())
}

/// SDK crates under `engine/crates/` that need path replacement in local dev mode.
const SDK_CRATES: [&str; 5] = [
    "wavecraft-core",
    "wavecraft-protocol",
    "wavecraft-dsp",
    "wavecraft-bridge",
    "wavecraft-metering",
];

const TSCONFIG_PATHS_MARKER: &str = r#""@wavecraft/core": ["../../ui/packages/core/src/index.ts"]"#;
const TSCONFIG_PATHS_SNIPPET: &str = r#"    /* SDK development — resolve @wavecraft packages from monorepo source */
        "baseUrl": ".",
        "paths": {
            "@wavecraft/core": ["../../ui/packages/core/src/index.ts"],
            "@wavecraft/core/*": ["../../ui/packages/core/src/*"],
            "@wavecraft/components": ["../../ui/packages/components/src/index.ts"],
            "@wavecraft/components/*": ["../../ui/packages/components/src/*"]
        }"#;

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

    let mut result = content.to_string();

    // Replace the main wavecraft dependency (with Cargo rename)
    // Match: wavecraft = { package = "wavecraft-nih_plug", git = "...", tag = "..." }
    let wavecraft_git_pattern = r#"wavecraft\s*=\s*\{\s*package\s*=\s*"wavecraft-nih_plug"\s*,\s*git\s*=\s*"https://github\.com/RonHouben/wavecraft"\s*,\s*tag\s*=\s*"[^"]*"\s*\}"#;
    let wavecraft_path_replacement = format!(
        r#"wavecraft = {{ package = "wavecraft-nih_plug", path = "{}/wavecraft-nih_plug" }}"#,
        sdk_path.display()
    );
    let wavecraft_re = Regex::new(wavecraft_git_pattern)
        .context("Invalid regex pattern for wavecraft-nih_plug")?;
    result = wavecraft_re
        .replace_all(&result, wavecraft_path_replacement.as_str())
        .to_string();

    // Replace individual SDK crate dependencies
    for crate_name in &SDK_CRATES {
        // Match flexible git dependency patterns:
        // - Simple: crate = { git = "...", tag = "..." }
        // - With package: crate = { package = "crate", git = "...", tag = "..." }
        // - With optional: crate = { git = "...", tag = "...", optional = true }
        // - With features: crate = { git = "...", tag = "...", features = ["..."] }
        // - With both: crate = { package = "crate", git = "...", tag = "...", optional = true, features = [...] }
        let git_pattern = format!(
            r#"(?s)({}\s*=\s*\{{\s*)(?:package\s*=\s*"[^"]*"\s*,\s*)?git\s*=\s*"https://github\.com/RonHouben/wavecraft"\s*,\s*tag\s*=\s*"[^"]*"\s*((?:,\s*[^}}]*)?)\}}"#,
            regex::escape(crate_name)
        );

        let re = Regex::new(&git_pattern)
            .with_context(|| format!("Invalid regex pattern for crate: {}", crate_name))?;

        // Perform replacement preserving package and any extra attributes
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let prefix = &caps[1]; // "crate = { "
                let extra_attrs = &caps[2]; // ", optional = true, features = [...]" or empty

                // Check if package attribute exists in the original
                let package_attr = if caps[0].contains("package") {
                    format!(r#"package = "{}", "#, crate_name)
                } else {
                    String::new()
                };

                format!(
                    r#"{}{}path = "{}/{}"{} }}"#,
                    prefix,
                    package_attr,
                    sdk_path.display(),
                    crate_name,
                    extra_attrs
                )
            })
            .to_string();
    }

    // Handle wavecraft-dev-server separately — it lives at the repo root (dev-server/),
    // not under engine/crates/ like the other SDK crates.
    let sdk_root = sdk_path
        .parent()
        .and_then(|engine| engine.parent())
        .unwrap_or(&sdk_path);
    let dev_server_git_pattern = r#"(?s)(wavecraft-dev-server\s*=\s*\{\s*)(?:package\s*=\s*"[^"]*"\s*,\s*)?git\s*=\s*"https://github\.com/RonHouben/wavecraft"\s*,\s*tag\s*=\s*"[^"]*"\s*((?:,\s*[^}]*)?)}"#;
    let dev_server_re = Regex::new(dev_server_git_pattern)
        .context("Invalid regex pattern for wavecraft-dev-server")?;
    result = dev_server_re
        .replace_all(&result, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let extra_attrs = &caps[2];
            let package_attr = if caps[0].contains("package") {
                r#"package = "wavecraft-dev-server", "#.to_string()
            } else {
                String::new()
            };
            format!(
                r#"{}{}path = "{}/dev-server"{} }}"#,
                prefix,
                package_attr,
                sdk_root.display(),
                extra_attrs
            )
        })
        .to_string();

    // Inject TypeScript path mappings for SDK mode so TS language services
    // resolve @wavecraft/* to local monorepo sources (matching Vite aliases).
    result = inject_tsconfig_paths_if_needed(&result)?;

    Ok(result)
}

fn inject_tsconfig_paths_if_needed(content: &str) -> Result<String> {
    // Only attempt tsconfig injection for tsconfig-like JSON content.
    if !content.contains("\"compilerOptions\"") {
        return Ok(content.to_string());
    }

    // Idempotent: if paths already exist, do nothing.
    if content.contains(TSCONFIG_PATHS_MARKER) {
        return Ok(content.to_string());
    }

    let Some((compiler_options_start, compiler_options_end)) =
        find_object_bounds_after_key(content, "\"compilerOptions\"")
    else {
        return Ok(content.to_string());
    };

    let compiler_options_content = &content[compiler_options_start + 1..compiler_options_end];
    if compiler_options_content.contains("\"paths\"") {
        return Ok(content.to_string());
    }

    let anchor_re = Regex::new(
        r#"\"(noFallthroughCasesInSwitch|noUnusedParameters|noUnusedLocals|strict|jsx|noEmit|moduleResolution|target)\"\s*:\s*[^\n]*"#,
    )
    .context("Invalid regex for tsconfig anchor detection")?;

    if let Some(anchor) = anchor_re.find(compiler_options_content) {
        let anchor_start = compiler_options_start + 1 + anchor.start();
        let anchor_end = compiler_options_start + 1 + anchor.end();
        let anchor_text = &content[anchor_start..anchor_end];
        let needs_comma = !anchor_text.trim_end().ends_with(',');
        let comma = if needs_comma { "," } else { "" };
        let has_following_properties =
            has_jsonc_property_after_anchor(&content[anchor_end..compiler_options_end]);

        let mut injected = String::with_capacity(content.len() + 256);
        injected.push_str(&content[..anchor_end]);
        injected.push_str(comma);
        injected.push_str("\n\n");
        injected.push_str(TSCONFIG_PATHS_SNIPPET);
        if has_following_properties {
            injected.push(',');
        }
        injected.push_str(&content[anchor_end..]);
        return Ok(injected);
    }

    let trimmed = compiler_options_content.trim_end();
    let has_properties = trimmed.contains('"') && trimmed.contains(':');
    let needs_comma = has_properties && !trimmed.ends_with(',');
    let comma = if needs_comma { "," } else { "" };

    let mut injected = String::with_capacity(content.len() + 256);
    injected.push_str(&content[..compiler_options_end]);
    injected.push_str(comma);
    injected.push_str("\n\n");
    injected.push_str(TSCONFIG_PATHS_SNIPPET);
    injected.push_str("\n");
    injected.push_str(&content[compiler_options_end..]);

    Ok(injected)
}

fn has_jsonc_property_after_anchor(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        while index < bytes.len() && (bytes[index].is_ascii_whitespace() || bytes[index] == b',') {
            index += 1;
        }

        if index >= bytes.len() {
            return false;
        }

        if bytes[index] == b'/' && index + 1 < bytes.len() {
            if bytes[index + 1] == b'/' {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                continue;
            }

            if bytes[index + 1] == b'*' {
                index += 2;
                while index + 1 < bytes.len() {
                    if bytes[index] == b'*' && bytes[index + 1] == b'/' {
                        index += 2;
                        break;
                    }
                    index += 1;
                }
                continue;
            }
        }

        return bytes[index] == b'"';
    }

    false
}

fn find_object_bounds_after_key(content: &str, key: &str) -> Option<(usize, usize)> {
    let key_start = content.find(key)?;
    let bytes = content.as_bytes();
    let mut index = key_start + key.len();

    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }

    if index >= bytes.len() || bytes[index] != b':' {
        return None;
    }
    index += 1;

    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }

    while index < bytes.len() && bytes[index] != b'{' {
        index += 1;
    }

    if index >= bytes.len() || bytes[index] != b'{' {
        return None;
    }

    let open_index = index;
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut is_escaped = false;
    let mut cursor = open_index;

    while cursor < bytes.len() {
        let ch = bytes[cursor];

        if in_string {
            if is_escaped {
                is_escaped = false;
            } else if ch == b'\\' {
                is_escaped = true;
            } else if ch == b'"' {
                in_string = false;
            }
            cursor += 1;
            continue;
        }

        if ch == b'"' {
            in_string = true;
            cursor += 1;
            continue;
        }

        if ch == b'/' && cursor + 1 < bytes.len() {
            let next = bytes[cursor + 1];
            if next == b'/' {
                cursor += 2;
                while cursor < bytes.len() && bytes[cursor] != b'\n' {
                    cursor += 1;
                }
                continue;
            }

            if next == b'*' {
                cursor += 2;
                while cursor + 1 < bytes.len() {
                    if bytes[cursor] == b'*' && bytes[cursor + 1] == b'/' {
                        cursor += 2;
                        break;
                    }
                    cursor += 1;
                }
                continue;
            }
        }

        if ch == b'{' {
            depth += 1;
        } else if ch == b'}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some((open_index, cursor));
            }
        }

        cursor += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_extract_template() {
        let temp = tempdir().unwrap();
        let vars = TemplateVariables::new(
            "my-plugin".to_string(),
            "My Company".to_string(),
            "info@example.com".to_string(),
            "https://example.com".to_string(),
            "0.9.0".to_string(),
            None, // local_dev
        );

        // This test will only pass once we copy the template files
        // For now, it's a placeholder to verify the logic compiles
        let _ = extract_template(temp.path(), &vars);
    }

    #[test]
    fn test_embedded_template_excludes_build_artifact_directories() {
        assert!(
            TEMPLATE_DIR.get_dir("target").is_none(),
            "target should not be embedded"
        );
        assert!(
            TEMPLATE_DIR.get_dir("engine/target").is_none(),
            "engine/target should not be embedded"
        );
        assert!(
            TEMPLATE_DIR.get_dir("ui/node_modules").is_none(),
            "ui/node_modules should not be embedded"
        );
        assert!(
            TEMPLATE_DIR.get_dir("ui/dist").is_none(),
            "ui/dist should not be embedded"
        );
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

        // Create a temp directory simulating the SDK layout:
        //   {root}/engine/crates/  ← sdk_path
        //   {root}/dev-server/     ← wavecraft-dev-server location
        let temp = tempdir().unwrap();
        let sdk_root = temp.path();
        let sdk_path = sdk_root.join("engine").join("crates");
        fs::create_dir_all(&sdk_path).unwrap();

        // Create the engine crate directories so canonicalize works
        for crate_name in &SDK_CRATES {
            fs::create_dir_all(sdk_path.join(crate_name)).unwrap();
        }
        fs::create_dir_all(sdk_path.join("wavecraft-nih_plug")).unwrap();

        // Create the dev-server directory at the SDK root
        fs::create_dir_all(sdk_root.join("dev-server")).unwrap();

        let vars = TemplateVariables::new(
            "test-plugin".to_string(),
            "Test Vendor".to_string(),
            "test@example.com".to_string(),
            "https://test.com".to_string(),
            "v0.9.0".to_string(),
            Some(sdk_path.clone()),
        );

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

        let vars = TemplateVariables::new(
            "test-plugin".to_string(),
            "Test Vendor".to_string(),
            "test@example.com".to_string(),
            "https://test.com".to_string(),
            "v0.9.0".to_string(),
            None, // No local_dev
        );

        let result = apply_local_dev_overrides(content, &vars).unwrap();

        // Content should be unchanged when local_dev is None
        assert_eq!(result, content);
    }

    #[test]
    fn test_apply_local_dev_overrides_invalid_path() {
        let content = "wavecraft-core = { git = \"https://github.com/RonHouben/wavecraft\", tag = \"v0.7.0\" }";

        let vars = TemplateVariables::new(
            "test-plugin".to_string(),
            "Test Vendor".to_string(),
            "test@example.com".to_string(),
            "https://test.com".to_string(),
            "v0.9.0".to_string(),
            Some(PathBuf::from("/nonexistent/path/that/does/not/exist")),
        );

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

        let temp = tempdir().unwrap();
        let sdk_root = temp.path();
        let sdk_path = sdk_root.join("engine").join("crates");
        fs::create_dir_all(&sdk_path).unwrap();

        for crate_name in &SDK_CRATES {
            fs::create_dir_all(sdk_path.join(crate_name)).unwrap();
        }
        fs::create_dir_all(sdk_path.join("wavecraft-nih_plug")).unwrap();
        fs::create_dir_all(sdk_root.join("dev-server")).unwrap();

        let vars = TemplateVariables::new(
            "test-plugin".to_string(),
            "Test Vendor".to_string(),
            "test@example.com".to_string(),
            "https://test.com".to_string(),
            "v0.9.0".to_string(),
            Some(sdk_path),
        );

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

        let temp = tempdir().unwrap();
        let sdk_root = temp.path();
        let sdk_path = sdk_root.join("engine").join("crates");
        fs::create_dir_all(&sdk_path).unwrap();

        for crate_name in &SDK_CRATES {
            fs::create_dir_all(sdk_path.join(crate_name)).unwrap();
        }
        fs::create_dir_all(sdk_path.join("wavecraft-nih_plug")).unwrap();
        fs::create_dir_all(sdk_root.join("dev-server")).unwrap();

        let vars = TemplateVariables::new(
            "test-plugin".to_string(),
            "Test Vendor".to_string(),
            "test@example.com".to_string(),
            "https://test.com".to_string(),
            "v0.9.0".to_string(),
            Some(sdk_path),
        );

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

        let temp = tempdir().unwrap();
        let sdk_root = temp.path();
        let sdk_path = sdk_root.join("engine").join("crates");
        fs::create_dir_all(&sdk_path).unwrap();

        for crate_name in &SDK_CRATES {
            fs::create_dir_all(sdk_path.join(crate_name)).unwrap();
        }
        fs::create_dir_all(sdk_path.join("wavecraft-nih_plug")).unwrap();
        fs::create_dir_all(sdk_root.join("dev-server")).unwrap();

        let vars = TemplateVariables::new(
            "test-plugin".to_string(),
            "Test Vendor".to_string(),
            "test@example.com".to_string(),
            "https://test.com".to_string(),
            "v0.9.0".to_string(),
            Some(sdk_path),
        );

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

        let vars = TemplateVariables::new(
            "test-plugin".to_string(),
            "Test Vendor".to_string(),
            "test@example.com".to_string(),
            "https://test.com".to_string(),
            "v0.9.0".to_string(),
            None,
        );

        let result = apply_local_dev_overrides(content, &vars).unwrap();
        assert_eq!(
            result, content,
            "Content should be unchanged without local_dev"
        );
    }

    #[test]
    fn test_apply_local_dev_overrides_ignores_non_tsconfig_files() {
        let content = r#"[package]
name = "test-plugin"
version = "0.1.0"
"#;

        let temp = tempdir().unwrap();
        let sdk_root = temp.path();
        let sdk_path = sdk_root.join("engine").join("crates");
        fs::create_dir_all(&sdk_path).unwrap();

        for crate_name in &SDK_CRATES {
            fs::create_dir_all(sdk_path.join(crate_name)).unwrap();
        }
        fs::create_dir_all(sdk_path.join("wavecraft-nih_plug")).unwrap();
        fs::create_dir_all(sdk_root.join("dev-server")).unwrap();

        let vars = TemplateVariables::new(
            "test-plugin".to_string(),
            "Test Vendor".to_string(),
            "test@example.com".to_string(),
            "https://test.com".to_string(),
            "v0.9.0".to_string(),
            Some(sdk_path),
        );

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
