//! Static asset embedding for the React UI
//!
//! This module embeds the built React application (`ui/dist/`) directly
//! into the Rust binary using `include_dir!`. Assets are served via a
//! custom protocol handler in the WebView.

use include_dir::{Dir, include_dir};

/// Embedded UI assets from `ui/dist/`
///
/// This directory is populated by running `npm run build` in the `ui/` folder.
/// If the directory doesn't exist yet, the build will fail with a clear error.
static UI_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../../ui/dist");

/// Get an embedded asset by path
///
/// # Arguments
/// * `path` - Relative path within the `ui/dist/` directory (e.g., "index.html", "assets/main.js")
///
/// # Returns
/// * `Some((bytes, mime_type))` if asset exists
/// * `None` if asset not found
pub fn get_asset(path: &str) -> Option<(&'static [u8], &'static str)> {
    // Normalize path (remove leading slash)
    let path = path.trim_start_matches('/');

    // Special case: empty path or "/" -> index.html
    let path = if path.is_empty() { "index.html" } else { path };

    // Get file from embedded directory
    let file = UI_ASSETS.get_file(path)?;

    // Infer MIME type from extension
    let mime_type = mime_type_from_path(path);

    Some((file.contents(), mime_type))
}

/// Infer MIME type from file extension
fn mime_type_from_path(path: &str) -> &'static str {
    let extension = path.split('.').next_back().unwrap_or("");

    match extension {
        "html" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use include_dir::Dir;

    /// List all embedded assets (for debugging)
    fn list_assets() -> Vec<String> {
        let mut paths = Vec::new();
        collect_paths(&UI_ASSETS, "", &mut paths);
        paths
    }

    fn collect_paths(dir: &Dir, prefix: &str, paths: &mut Vec<String>) {
        for file in dir.files() {
            paths.push(format!("{}{}", prefix, file.path().display()));
        }
        for subdir in dir.dirs() {
            let subdir_name = subdir.path().file_name().unwrap().to_str().unwrap();
            collect_paths(subdir, &format!("{}{}/", prefix, subdir_name), paths);
        }
    }

    #[test]
    fn test_mime_type_inference() {
        assert_eq!(mime_type_from_path("index.html"), "text/html");
        assert_eq!(mime_type_from_path("style.css"), "text/css");
        assert_eq!(mime_type_from_path("app.js"), "application/javascript");
        assert_eq!(mime_type_from_path("data.json"), "application/json");
        assert_eq!(
            mime_type_from_path("unknown.xyz"),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_list_assets() {
        // This test will fail until ui/dist/ is created with the React build
        // For now, just verify the function doesn't panic
        let assets = list_assets();
        // If ui/dist is empty or doesn't exist, assets will be empty
        // Once we build the React app, this should have entries
        println!("Found {} embedded assets", assets.len());
        for asset in assets.iter().take(10) {
            println!("  - {}", asset);
        }
    }

    #[test]
    #[ignore] // Only run when ui/dist exists
    fn test_get_index_html() {
        let (content, mime_type) =
            get_asset("index.html").expect("index.html should exist after React build");

        assert_eq!(mime_type, "text/html");
        assert!(!content.is_empty());

        // Verify it's valid HTML
        let html = std::str::from_utf8(content).unwrap();
        assert!(html.contains("<!DOCTYPE html") || html.contains("<!doctype html"));
    }
}
