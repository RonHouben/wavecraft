//! Static asset embedding for the plugin UI.
//!
//! This module conditionally embeds the built React UI assets at compile time.
//! The embedded fallback assets live in `assets/ui-dist/` within the crate.
//!
//! Distribution contract:
//! - Non-discovery builds embed fallback UI assets from `assets/ui-dist/`.
//! - Those files must be tracked in git so git-tag/source consumers compile
//!   without requiring a local UI build step.

#[cfg(any(target_os = "macos", target_os = "windows"))]
use include_dir::{Dir, include_dir};

/// Embedded UI assets from the crate's fallback UI dist directory.
///
/// This is built at compile time and includes all files from assets/ui-dist.
#[cfg(any(target_os = "macos", target_os = "windows"))]
static UI_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/ui-dist");

/// Get an embedded asset by path.
///
/// Returns None if the asset doesn't exist or assets aren't compiled in.
///
/// # Arguments
///
/// * `path` - The path relative to the embedded UI dist directory,
///   e.g. "index.html" or "assets/index-xxx.js"
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn get_asset(path: &str) -> Option<(&'static [u8], &'static str)> {
    // Normalize path by removing leading slash if present
    let normalized = path.strip_prefix('/').unwrap_or(path);

    UI_ASSETS.get_file(normalized).map(|file| {
        let content = file.contents();
        let mime = guess_mime_type(normalized);
        (content, mime)
    })
}

/// List all embedded assets (for debugging).
#[cfg(any(target_os = "macos", target_os = "windows"))]
#[allow(dead_code)]
pub fn list_assets() -> Vec<String> {
    fn collect_files(dir: &Dir, prefix: &str, files: &mut Vec<String>) {
        for file in dir.files() {
            let path = if prefix.is_empty() {
                file.path().to_string_lossy().to_string()
            } else {
                format!("{}/{}", prefix, file.path().to_string_lossy())
            };
            files.push(path);
        }
        for subdir in dir.dirs() {
            let new_prefix = if prefix.is_empty() {
                subdir.path().to_string_lossy().to_string()
            } else {
                format!("{}/{}", prefix, subdir.path().to_string_lossy())
            };
            collect_files(subdir, &new_prefix, files);
        }
    }

    let mut files = Vec::new();
    collect_files(&UI_ASSETS, "", &mut files);
    files
}

/// Guess MIME type from file extension.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn guess_mime_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html",
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod tests {
    use super::*;

    fn assert_required_asset(path: &str) {
        let asset = get_asset(path);
        assert!(
            asset.is_some(),
            "required fallback asset should exist in embedded assets: {}",
            path
        );
    }

    #[test]
    fn test_index_html_exists() {
        // The fallback embedded index should always exist.
        assert_required_asset("index.html");
    }

    #[test]
    fn test_required_fallback_assets_exist() {
        // Keep this list in sync with engine/crates/wavecraft-nih_plug/assets/ui-dist.
        assert_required_asset("index.html");
        assert_required_asset("assets/fallback.css");
        assert_required_asset("assets/fallback.js");
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(guess_mime_type("index.html"), "text/html");
        assert_eq!(guess_mime_type("app.js"), "application/javascript");
        assert_eq!(guess_mime_type("styles.css"), "text/css");
        assert_eq!(guess_mime_type("data.json"), "application/json");
    }
}
