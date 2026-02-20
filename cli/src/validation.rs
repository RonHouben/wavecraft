use anyhow::{bail, Result};
use regex::Regex;
use std::sync::LazyLock;

const MAX_PLUGIN_NAME_LEN: usize = 64;
const STD_CRATES: &[&str] = &["std", "core", "alloc", "test", "proc_macro"];

static CRATE_NAME_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$").expect("valid crate name regex"));

/// Validates that the name is a valid Rust crate name.
/// Rules: alphanumeric + underscore/hyphen, starts with letter, not reserved.
/// Note: Crate names are case-insensitive on crates.io, but we allow mixed case.
pub fn validate_crate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Plugin name cannot be empty");
    }

    if name.len() > MAX_PLUGIN_NAME_LEN {
        return Err(plugin_name_too_long_error());
    }

    ensure_valid_crate_name_format(name)?;

    // Use syn to check if the name (with hyphens converted to underscores) is a valid identifier.
    // This automatically checks against Rust's keyword list and stays up-to-date with the language.
    ensure_not_reserved_rust_keyword(name)?;

    // Additional check for standard library crates to avoid conflicts
    ensure_not_std_crate_name(name)?;

    Ok(())
}

fn ensure_valid_crate_name_format(name: &str) -> Result<()> {
    // Allow mixed case: must start with a letter, then letters/numbers/hyphens/underscores
    if CRATE_NAME_PATTERN.is_match(name) {
        return Ok(());
    }

    Err(invalid_plugin_name_error(name))
}

fn ensure_not_reserved_rust_keyword(name: &str) -> Result<()> {
    let ident_name = name.replace('-', "_");
    if syn::parse_str::<syn::Ident>(&ident_name).is_ok() {
        return Ok(());
    }

    Err(reserved_keyword_error(name))
}

fn ensure_not_std_crate_name(name: &str) -> Result<()> {
    if !STD_CRATES.contains(&name) {
        return Ok(());
    }

    Err(std_crate_name_error(name))
}

fn plugin_name_too_long_error() -> anyhow::Error {
    anyhow::anyhow!("Plugin name cannot exceed 64 characters")
}

fn invalid_plugin_name_error(name: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "Invalid plugin name '{}'. Must start with a letter \
         and contain only letters, numbers, hyphens, or underscores.",
        name
    )
}

fn reserved_keyword_error(name: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "'{}' is a reserved Rust keyword and cannot be used as a plugin name",
        name
    )
}

fn std_crate_name_error(name: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "'{}' is a standard library crate name and cannot be used as a plugin name",
        name
    )
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
        // Mixed case is allowed
        assert!(validate_crate_name("MyPlugin").is_ok());
        assert!(validate_crate_name("myCoolPlugin").is_ok());
        assert!(validate_crate_name("My-Cool-Plugin").is_ok());
    }

    #[test]
    fn invalid_names() {
        assert!(validate_crate_name("").is_err());
        assert!(validate_crate_name("123plugin").is_err());
        assert!(validate_crate_name("-plugin").is_err());
        assert!(validate_crate_name("std").is_err());
        assert!(validate_crate_name("match").is_err());
        assert!(validate_crate_name("async").is_err());
    }
}
