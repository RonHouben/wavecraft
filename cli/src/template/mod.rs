pub mod variables;

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use regex::Regex;
use std::fs;
use std::path::Path;

use crate::template::variables::TemplateVariables;

static TEMPLATE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../plugin-template");

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
                let dir_name = subdir.path().file_name()
                    .with_context(|| format!("Invalid directory path: {}", subdir.path().display()))?;
                let dir_name_str = dir_name.to_string_lossy();
                
                if dir_name_str == "target" || dir_name_str == "node_modules" || dir_name_str == "dist" {
                    continue; // Skip these directories
                }
                
                let subdir_path = target_dir.join(dir_name);
                extract_dir(subdir, &subdir_path, vars)?;
            }
            include_dir::DirEntry::File(file) => {
                // Skip binary files and lock files
                let file_name = file.path().file_name()
                    .with_context(|| format!("Invalid file path: {}", file.path().display()))?;
                let file_name_str = file_name.to_string_lossy();
                
                if file_name_str == "Cargo.lock" || file_name_str.ends_with(".dylib") 
                    || file_name_str.ends_with(".so") || file_name_str.ends_with(".dll")
                    || file_name_str == ".DS_Store" {
                    continue; // Skip these files
                }
                
                let file_path = target_dir.join(file_name);
                
                // Only process text files
                if let Some(content) = file.contents_utf8() {
                    let mut processed = vars.apply(content)
                        .with_context(|| format!("Failed to process template: {}", file.path().display()))?;
                    
                    // Post-process for local dev mode (replace git deps with path deps)
                    if vars.local_dev.is_some() {
                        processed = apply_local_dev_overrides(&processed, vars)?;
                    }
                    
                    fs::write(&file_path, processed)
                        .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
                } else {
                    // Skip non-UTF8 files (likely binaries)
                    continue;
                }
            }
        }
    }
    
    Ok(())
}

/// SDK crates that need to be replaced when using local dev mode.
const SDK_CRATES: [&str; 5] = [
    "wavecraft-core",
    "wavecraft-protocol",
    "wavecraft-dsp",
    "wavecraft-bridge",
    "wavecraft-metering",
];

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
    result = wavecraft_re.replace_all(&result, wavecraft_path_replacement.as_str()).to_string();
    
    // Replace individual SDK crate dependencies
    for crate_name in &SDK_CRATES {
        // Match: crate_name = { git = "https://github.com/RonHouben/wavecraft", tag = "..." }
        let git_pattern = format!(
            r#"{}\s*=\s*\{{\s*git\s*=\s*"https://github\.com/RonHouben/wavecraft"\s*,\s*tag\s*=\s*"[^"]*"\s*\}}"#,
            regex::escape(crate_name)
        );
        let path_replacement = format!(
            r#"{} = {{ path = "{}/{}" }}"#,
            crate_name,
            sdk_path.display(),
            crate_name
        );
        
        let re = Regex::new(&git_pattern)
            .with_context(|| format!("Invalid regex pattern for crate: {}", crate_name))?;
        result = re.replace_all(&result, path_replacement.as_str()).to_string();
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::path::PathBuf;
    
    #[test]
    fn test_extract_template() {
        let temp = tempdir().unwrap();
        let vars = TemplateVariables::new(
            "my-plugin".to_string(),
            "My Company".to_string(),
            Some("info@example.com".to_string()),
            Some("https://example.com".to_string()),
            "0.7.0".to_string(),
            None, // local_dev
        );
        
        // This test will only pass once we copy the template files
        // For now, it's a placeholder to verify the logic compiles
        let _ = extract_template(temp.path(), &vars);
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
"#;
        
        // Create a temp directory to use as the SDK path
        let temp = tempdir().unwrap();
        let sdk_path = temp.path().to_path_buf();
        
        // Create the crate directories so canonicalize works
        for crate_name in &SDK_CRATES {
            fs::create_dir_all(sdk_path.join(crate_name)).unwrap();
        }
        fs::create_dir_all(sdk_path.join("wavecraft-nih_plug")).unwrap();
        
        let vars = TemplateVariables::new(
            "test-plugin".to_string(),
            "Test Vendor".to_string(),
            None,
            None,
            "v0.7.0".to_string(),
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
    }
    
    #[test]
    fn test_apply_local_dev_overrides_no_local_dev() {
        let content = r#"wavecraft-core = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }"#;
        
        let vars = TemplateVariables::new(
            "test-plugin".to_string(),
            "Test Vendor".to_string(),
            None,
            None,
            "v0.7.0".to_string(),
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
            None,
            None,
            "v0.7.0".to_string(),
            Some(PathBuf::from("/nonexistent/path/that/does/not/exist")),
        );
        
        // Should fail because the path doesn't exist
        let result = apply_local_dev_overrides(content, &vars);
        assert!(result.is_err());
    }
}
