//! Validate CLI wavecraft-* dependency versions and publishability.
//!
//! This command replaces the two inline Python heredoc scripts that previously
//! lived in `.github/workflows/continuous-deploy.yml`. It performs two checks
//! for every `wavecraft-*` dependency in `cli/Cargo.toml`:
//!
//! 1. **Version field** — Each dep must have a `version` key (required for `cargo publish`)
//! 2. **Publishability** — The corresponding crate at `engine/crates/{name}/Cargo.toml`
//!    must not have `publish = false`
//!
//! All errors are collected and reported in a single summary (no fail-fast).
//!
//! # Usage
//!
//! ```bash
//! cd engine && cargo xtask validate-cli-deps
//! cd engine && cargo xtask validate-cli-deps --verbose
//! ```

use anyhow::{Context, Result};

use xtask::output::*;

/// Configuration for the validate-cli-deps command.
#[derive(Debug, Clone, Default)]
pub struct ValidateCliDepsConfig {
    /// Show verbose output (per-dependency details).
    pub verbose: bool,
    /// Also verify crate availability on crates.io.
    pub check_registry: bool,
}

/// A discovered `wavecraft-*` dependency from `cli/Cargo.toml`.
#[derive(Debug, Clone)]
struct CliDependency {
    /// Crate name (e.g., `wavecraft-protocol`).
    name: String,
    /// Whether the dependency table includes a `version` key.
    has_version: bool,
    /// The version string, if present.
    version: Option<String>,
    /// The path to the crate (relative to cli/), if specified.
    path: Option<String>,
}

/// Validation error for a single dependency.
#[derive(Debug, Clone)]
struct ValidationError {
    name: String,
    message: String,
}

/// Run the validate-cli-deps command.
pub fn run(config: ValidateCliDepsConfig) -> Result<()> {
    print_header("Validate CLI Dependencies");

    let project_root = xtask::paths::project_root()?;
    let cli_toml_path = project_root.join("cli/Cargo.toml");

    // Step 1: Read and parse cli/Cargo.toml
    let cli_toml_content = std::fs::read_to_string(&cli_toml_path)
        .with_context(|| format!("Failed to read {}", cli_toml_path.display()))?;

    let deps = discover_wavecraft_deps(&cli_toml_content)?;

    if deps.is_empty() {
        anyhow::bail!("No wavecraft-* dependencies found in cli/Cargo.toml");
    }

    println!(
        "Discovered {} wavecraft-* dependenc{} in cli/Cargo.toml\n",
        deps.len(),
        if deps.len() == 1 { "y" } else { "ies" }
    );

    // Step 2: Validate each dependency
    let mut errors: Vec<ValidationError> = Vec::new();

    for dep in &deps {
        let dep_errors = validate_dependency(dep, &project_root);
        if dep_errors.is_empty() {
            if config.verbose {
                let version_str = dep
                    .version
                    .as_deref()
                    .map(|v| format!("version: {v}"))
                    .unwrap_or_else(|| "version: (none)".to_string());
                print_success_item(&format!("{} — {}, publishable: yes", dep.name, version_str));
            }
        } else {
            for err in &dep_errors {
                print_error_item(&format!("{} — {}", err.name, err.message));
            }
            errors.extend(dep_errors);
        }
    }

    // Step 2.5: Check registry availability if requested
    if config.check_registry {
        println!();
        print_header("Registry Availability Check");
        for dep in &deps {
            match check_registry_availability(dep) {
                Ok(reg_errors) => {
                    if reg_errors.is_empty() {
                        if config.verbose {
                            print_success_item(&format!("{} — available on crates.io", dep.name));
                        }
                    } else {
                        for err in &reg_errors {
                            print_error_item(&format!("{} — {}", err.name, err.message));
                        }
                        errors.extend(reg_errors);
                    }
                }
                Err(e) => {
                    print_error_item(&format!("{} — registry check failed: {}", dep.name, e));
                }
            }
        }
    }

    // Step 3: Print summary
    println!();
    if errors.is_empty() {
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        print_success(&format!(
            "All {} dependencies validated: {}",
            deps.len(),
            names.join(", ")
        ));
        Ok(())
    } else {
        print_error(&format!(
            "ERROR: {} validation error(s) found",
            errors.len()
        ));
        anyhow::bail!("{} validation error(s) found", errors.len());
    }
}

/// Parse `cli/Cargo.toml` content and extract all `wavecraft-*` dependencies.
///
/// Handles both inline-table (`wavecraft-foo = { path = "...", version = "1.0" }`)
/// and section-style (`[dependencies.wavecraft-foo]`) TOML formats. The `toml` crate
/// normalizes both into the same `Value::Table` structure.
fn discover_wavecraft_deps(toml_content: &str) -> Result<Vec<CliDependency>> {
    let parsed: toml::Value = toml_content
        .parse()
        .context("Failed to parse cli/Cargo.toml as TOML")?;

    let deps_table = parsed
        .get("dependencies")
        .and_then(|d| d.as_table())
        .context("No [dependencies] table found in cli/Cargo.toml")?;

    let mut result = Vec::new();

    for (name, value) in deps_table {
        if !name.starts_with("wavecraft-") {
            continue;
        }

        let (has_version, version, path) = match value {
            toml::Value::Table(table) => {
                let ver = table.get("version").and_then(|v| v.as_str());
                let p = table.get("path").and_then(|v| v.as_str());
                (ver.is_some(), ver.map(String::from), p.map(String::from))
            }
            toml::Value::String(ver) => {
                // Simple string dep like `wavecraft-foo = "1.0"` — has version
                (true, Some(ver.clone()), None)
            }
            _ => (false, None, None),
        };

        result.push(CliDependency {
            name: name.clone(),
            has_version,
            version,
            path,
        });
    }

    // Sort for deterministic output
    result.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(result)
}

/// Validate a single dependency: check version field and crate publishability.
fn validate_dependency(
    dep: &CliDependency,
    project_root: &std::path::Path,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check 1: version field
    if !dep.has_version {
        errors.push(ValidationError {
            name: dep.name.clone(),
            message: "missing version field".to_string(),
        });
    }

    // Check 2: crate publishability
    // Resolve crate location from `path` key or fallback to convention
    let crate_toml_path = if let Some(ref p) = dep.path {
        // Path is relative to cli/Cargo.toml, resolve from cli/ dir
        project_root.join("cli").join(p).join("Cargo.toml")
    } else {
        // Fallback: convention-based lookup for old deps
        project_root
            .join("engine/crates")
            .join(&dep.name)
            .join("Cargo.toml")
    };

    if !crate_toml_path.exists() {
        errors.push(ValidationError {
            name: dep.name.clone(),
            message: format!(
                "crate Cargo.toml not found at {}",
                crate_toml_path.display()
            ),
        });
        return errors;
    }

    match std::fs::read_to_string(&crate_toml_path) {
        Ok(content) => match content.parse::<toml::Value>() {
            Ok(toml) => {
                let publish = toml
                    .get("package")
                    .and_then(|p| p.get("publish"))
                    .and_then(|p| p.as_bool());

                if publish == Some(false) {
                    errors.push(ValidationError {
                        name: dep.name.clone(),
                        message: "crate has publish = false".to_string(),
                    });
                }
            }
            Err(e) => {
                errors.push(ValidationError {
                    name: dep.name.clone(),
                    message: format!("failed to parse crate Cargo.toml: {e}"),
                });
            }
        },
        Err(e) => {
            errors.push(ValidationError {
                name: dep.name.clone(),
                message: format!("failed to read crate Cargo.toml: {e}"),
            });
        }
    }

    errors
}

/// Generate the crates.io sparse index URL path prefix for a crate name.
///
/// The sparse index uses a specific directory structure based on crate name length:
/// - 1 char: `1/{name}`
/// - 2 chars: `2/{name}`
/// - 3 chars: `3/{first_char}/{name}`
/// - 4+ chars: `{first_two}/{next_two}/{name}`
///
/// Example: `wavecraft-core` → `wa/ve/wavecraft-core`
fn crate_index_prefix(name: &str) -> String {
    match name.len() {
        0 => panic!("crate name cannot be empty"),
        1 => format!("1/{}", name),
        2 => format!("2/{}", name),
        3 => format!("3/{}/{}", &name[..1], name),
        _ => format!("{}/{}/{}", &name[..2], &name[2..4], name),
    }
}

/// Check if a compatible version of the crate exists on crates.io.
///
/// Queries the crates.io sparse index to get the latest published version
/// and checks if it satisfies the CLI's version requirement.
fn check_registry_availability(dep: &CliDependency) -> Result<Vec<ValidationError>> {
    let version_req = match &dep.version {
        Some(v) => v,
        None => {
            return Ok(vec![ValidationError {
                name: dep.name.clone(),
                message: "no version specified for registry check".to_string(),
            }]);
        }
    };

    // Query crates.io sparse index
    let url = format!("https://index.crates.io/{}", crate_index_prefix(&dep.name));

    let response = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .with_context(|| format!("Failed to query crates.io for {}", dep.name))?;

    let body = response
        .into_string()
        .with_context(|| format!("Failed to read response for {}", dep.name))?;

    // Parse the latest version (last line of the index file)
    let latest_version = body
        .lines()
        .last()
        .and_then(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .and_then(|v| v["vers"].as_str().map(String::from));

    match latest_version {
        Some(ver) => {
            // Parse the CLI's version requirement as a semver range
            // Cargo interprets `version = "0.11.0"` as `^0.11.0` (caret requirement)
            let req_str = format!("^{}", version_req);
            let req = semver::VersionReq::parse(&req_str)
                .with_context(|| format!("Failed to parse version requirement: {}", req_str))?;

            let published_ver = semver::Version::parse(&ver)
                .with_context(|| format!("Failed to parse published version: {}", ver))?;

            if req.matches(&published_ver) {
                Ok(vec![]) // Compatible version exists
            } else {
                Ok(vec![ValidationError {
                    name: dep.name.clone(),
                    message: format!(
                        "crates.io has v{} but CLI requires ^{} — engine may need version bump",
                        ver, version_req
                    ),
                }])
            }
        }
        None => Ok(vec![ValidationError {
            name: dep.name.clone(),
            message: "not found on crates.io".to_string(),
        }]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_real_cli_cargo_toml() {
        // Test against the actual cli/Cargo.toml in the repo
        let project_root = xtask::paths::project_root().expect("should find project root");
        let cli_toml_path = project_root.join("cli/Cargo.toml");
        let content = std::fs::read_to_string(&cli_toml_path).expect("should read cli/Cargo.toml");

        let deps = discover_wavecraft_deps(&content).expect("should parse deps");

        // Should find exactly 3 wavecraft-* deps
        assert_eq!(deps.len(), 3, "Expected 3 wavecraft-* deps, got: {deps:?}");

        // Each should have a version
        for dep in &deps {
            assert!(
                dep.has_version,
                "Dependency {} should have a version field",
                dep.name
            );
            assert!(
                dep.version.is_some(),
                "Dependency {} should have a version string",
                dep.name
            );
        }

        // Verify the expected dep names (sorted)
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"wavecraft-bridge"));
        assert!(names.contains(&"wavecraft-dev-server"));
        assert!(names.contains(&"wavecraft-protocol"));
    }

    #[test]
    fn test_detect_missing_version() {
        let toml_content = r#"
[package]
name = "test-cli"
version = "0.1.0"

[dependencies]
wavecraft-foo = { path = "../engine/crates/wavecraft-foo" }
"#;

        let deps = discover_wavecraft_deps(toml_content).expect("should parse");
        assert_eq!(deps.len(), 1);
        assert!(!deps[0].has_version, "Should detect missing version");
        assert_eq!(deps[0].name, "wavecraft-foo");
    }

    #[test]
    fn test_detect_unpublishable_crate() {
        // Create a synthetic crate TOML with publish = false
        let crate_toml = r#"
[package]
name = "wavecraft-fake"
version = "0.1.0"
publish = false
"#;

        let parsed: toml::Value = crate_toml.parse().unwrap();
        let publish = parsed
            .get("package")
            .and_then(|p| p.get("publish"))
            .and_then(|p| p.as_bool());

        assert_eq!(publish, Some(false), "Should detect publish = false");
    }

    #[test]
    fn test_no_wavecraft_deps_found() {
        let toml_content = r#"
[package]
name = "test-cli"
version = "0.1.0"

[dependencies]
clap = "4.0"
anyhow = "1.0"
"#;

        let deps = discover_wavecraft_deps(toml_content).expect("should parse");
        assert!(deps.is_empty(), "Should find no wavecraft-* deps");
    }

    #[test]
    fn test_all_passing_synthetic() {
        let toml_content = r#"
[package]
name = "test-cli"
version = "0.1.0"

[dependencies]
clap = "4.0"

[dependencies.wavecraft-alpha]
path = "../engine/crates/wavecraft-alpha"
version = "0.5.0"

[dependencies.wavecraft-beta]
path = "../engine/crates/wavecraft-beta"
version = "0.5.0"
"#;

        let deps = discover_wavecraft_deps(toml_content).expect("should parse");
        assert_eq!(deps.len(), 2);
        assert!(deps[0].has_version);
        assert!(deps[1].has_version);
        assert_eq!(deps[0].version.as_deref(), Some("0.5.0"));
        assert_eq!(deps[1].version.as_deref(), Some("0.5.0"));
    }

    #[test]
    fn test_inline_table_format() {
        let toml_content = r#"
[package]
name = "test-cli"
version = "0.1.0"

[dependencies]
wavecraft-protocol = { path = "../engine/crates/wavecraft-protocol", version = "0.11.0" }
"#;

        let deps = discover_wavecraft_deps(toml_content).expect("should parse");
        assert_eq!(deps.len(), 1);
        assert!(deps[0].has_version);
        assert_eq!(deps[0].version.as_deref(), Some("0.11.0"));
    }

    #[test]
    fn test_validate_dependency_missing_crate_dir() {
        let dep = CliDependency {
            name: "wavecraft-nonexistent".to_string(),
            has_version: true,
            version: Some("0.1.0".to_string()),
            path: Some("../engine/crates/wavecraft-nonexistent".to_string()),
        };
        // Use a temp dir so the crate path definitely doesn't exist
        let fake_root = std::env::temp_dir().join("wavecraft-test-missing-crate");
        let errors = validate_dependency(&dep, &fake_root);
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0].message.contains("crate Cargo.toml not found"),
            "Expected 'crate Cargo.toml not found', got: {}",
            errors[0].message
        );
    }

    #[test]
    fn test_validate_dependency_malformed_toml() {
        let tmp = tempfile::tempdir().expect("should create temp dir");
        // Create cli dir for path resolution
        std::fs::create_dir_all(tmp.path().join("cli")).unwrap();
        let crate_dir = tmp.path().join("engine/crates/wavecraft-broken");
        std::fs::create_dir_all(&crate_dir).unwrap();
        std::fs::write(crate_dir.join("Cargo.toml"), "this is not valid [[[ toml").unwrap();

        let dep = CliDependency {
            name: "wavecraft-broken".to_string(),
            has_version: true,
            version: Some("0.1.0".to_string()),
            path: Some("../engine/crates/wavecraft-broken".to_string()),
        };
        let errors = validate_dependency(&dep, tmp.path());
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0].message.contains("failed to parse"),
            "Expected 'failed to parse', got: {}",
            errors[0].message
        );
    }

    #[test]
    fn test_publish_key_absent_is_publishable() {
        let tmp = tempfile::tempdir().expect("should create temp dir");
        // Create cli dir for path resolution
        std::fs::create_dir_all(tmp.path().join("cli")).unwrap();
        let crate_dir = tmp.path().join("engine/crates/wavecraft-publishable");
        std::fs::create_dir_all(&crate_dir).unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "wavecraft-publishable"
version = "0.1.0"
"#,
        )
        .unwrap();

        let dep = CliDependency {
            name: "wavecraft-publishable".to_string(),
            has_version: true,
            version: Some("0.1.0".to_string()),
            path: Some("../engine/crates/wavecraft-publishable".to_string()),
        };
        let errors = validate_dependency(&dep, tmp.path());
        assert!(
            errors.is_empty(),
            "Absent publish key should be treated as publishable, got errors: {errors:?}"
        );
    }

    #[test]
    fn test_crate_index_prefix() {
        // Test the crates.io sparse index URL path generation
        assert_eq!(crate_index_prefix("a"), "1/a");
        assert_eq!(crate_index_prefix("ab"), "2/ab");
        assert_eq!(crate_index_prefix("abc"), "3/a/abc");
        assert_eq!(crate_index_prefix("abcd"), "ab/cd/abcd");
        assert_eq!(crate_index_prefix("wavecraft-core"), "wa/ve/wavecraft-core");
        assert_eq!(
            crate_index_prefix("wavecraft-bridge"),
            "wa/ve/wavecraft-bridge"
        );
    }

    #[test]
    fn test_check_registry_config_default() {
        // Verify that check_registry defaults to false
        let config = ValidateCliDepsConfig::default();
        assert!(
            !config.check_registry,
            "check_registry should default to false"
        );
        assert!(!config.verbose, "verbose should default to false");
    }
}
