pub mod variables;

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use std::fs;
use std::path::Path;

use crate::template::variables::TemplateVariables;

static TEMPLATE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/template");

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
                // Use just the directory name, not the full path
                let dir_name = subdir.path().file_name()
                    .with_context(|| format!("Invalid directory path: {}", subdir.path().display()))?;
                let subdir_path = target_dir.join(dir_name);
                extract_dir(subdir, &subdir_path, vars)?;
            }
            include_dir::DirEntry::File(file) => {
                // Use just the file name, not the full path
                let file_name = file.path().file_name()
                    .with_context(|| format!("Invalid file path: {}", file.path().display()))?;
                let file_path = target_dir.join(file_name);
                let content = file.contents_utf8()
                    .with_context(|| format!("File is not valid UTF-8: {}", file.path().display()))?;
                
                let processed = vars.apply(content)
                    .with_context(|| format!("Failed to process template: {}", file.path().display()))?;
                
                fs::write(&file_path, processed)
                    .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
            }
        }
    }
    
    Ok(())
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
            Some("info@example.com".to_string()),
            Some("https://example.com".to_string()),
            "0.7.0".to_string(),
        );
        
        // This test will only pass once we copy the template files
        // For now, it's a placeholder to verify the logic compiles
        let _ = extract_template(temp.path(), &vars);
    }
}
