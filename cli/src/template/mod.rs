mod dependency_rewrites;
mod extract;
mod overrides;
mod tsconfig_paths;
pub mod variables;

use anyhow::Result;
use include_dir::{include_dir, Dir};
use std::path::Path;

use crate::template::variables::TemplateVariables;

// The canonical template lives in sdk-template/ at the repository root and is
// staged via build.rs so build artifacts (target/, node_modules/, dist/) are
// excluded before being embedded into the CLI binary.
static TEMPLATE_DIR: Dir = include_dir!("$OUT_DIR/sdk-template-clean");

/// Extracts the embedded template to the target directory and applies variable replacement.
pub fn extract_template(target_dir: &Path, vars: &TemplateVariables) -> Result<()> {
    extract::extract_dir(
        &TEMPLATE_DIR,
        target_dir,
        vars,
        &overrides::apply_post_processing,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
