use anyhow::{bail, Result};
use regex::Regex;

/// Validates that the name is a valid Rust crate name.
/// Rules: lowercase, alphanumeric + underscore/hyphen, starts with letter, not reserved.
pub fn validate_crate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Plugin name cannot be empty");
    }
    
    if name.len() > 64 {
        bail!("Plugin name cannot exceed 64 characters");
    }
    
    let pattern = Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap();
    if !pattern.is_match(name) {
        bail!(
            "Invalid plugin name '{}'. Must start with a lowercase letter \
             and contain only lowercase letters, numbers, hyphens, or underscores.",
            name
        );
    }
    
    const RESERVED: &[&str] = &["std", "core", "alloc", "test", "proc_macro", "self", "super", "crate"];
    if RESERVED.contains(&name) {
        bail!("'{}' is a reserved Rust keyword and cannot be used as a plugin name", name);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn valid_names() {
        assert!(validate_crate_name("my-plugin").is_ok());
        assert!(validate_crate_name("my_plugin").is_ok());
        assert!(validate_crate_name("plugin123").is_ok());
        assert!(validate_crate_name("a").is_ok());
    }
    
    #[test]
    fn invalid_names() {
        assert!(validate_crate_name("").is_err());
        assert!(validate_crate_name("MyPlugin").is_err());
        assert!(validate_crate_name("123plugin").is_err());
        assert!(validate_crate_name("-plugin").is_err());
        assert!(validate_crate_name("std").is_err());
    }
}
