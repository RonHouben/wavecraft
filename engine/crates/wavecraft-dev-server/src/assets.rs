//! Static asset embedding for the React UI
//!
//! This module embeds the built React application (`ui/dist/`) directly
//! into the Rust binary using `include_dir!`. Assets are served via a
//! custom protocol handler in the WebView.

use include_dir::{Dir, include_dir};
use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};

/// Embedded fallback UI assets bundled with the crate.
///
/// These assets are used when `ui/dist` is not available on disk.
static UI_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/ui-dist");

/// Get an embedded asset by path
///
/// # Arguments
/// * `path` - Relative path within the `ui/dist/` directory (e.g., "index.html", "assets/main.js")
///
/// # Returns
/// * `Some((bytes, mime_type))` if asset exists
/// * `None` if asset not found
pub fn get_asset(path: &str) -> Option<(Cow<'static, [u8]>, &'static str)> {
    // Normalize path (remove leading slash)
    let path = path.trim_start_matches('/');

    // Special case: empty path or "/" -> index.html
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(contents) = try_disk_asset(path) {
        let mime_type = mime_type_from_path(path);
        return Some((Cow::Owned(contents), mime_type));
    }

    // Get file from embedded fallback directory
    let file = UI_ASSETS.get_file(path)?;

    // Infer MIME type from extension
    let mime_type = mime_type_from_path(path);

    Some((Cow::Borrowed(file.contents()), mime_type))
}

fn try_disk_asset(path: &str) -> Option<Vec<u8>> {
    if !is_safe_relative_path(path) {
        return None;
    }

    let base_dir = ui_dist_dir();
    let asset_path = base_dir.join(path);

    if !asset_path.exists() {
        return None;
    }

    std::fs::read(asset_path).ok()
}

fn ui_dist_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ui/dist")
}

fn is_safe_relative_path(path: &str) -> bool {
    let candidate = Path::new(path);
    !candidate.is_absolute()
        && candidate
            .components()
            .all(|component| component != Component::ParentDir)
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
    fn test_get_index_html() {
        let (content, mime_type) =
            get_asset("index.html").expect("index.html should exist after React build");

        assert_eq!(mime_type, "text/html");
        assert!(!content.is_empty());

        // Verify it's valid HTML
        let html = std::str::from_utf8(content.as_ref()).unwrap();
        assert!(html.contains("<!DOCTYPE html") || html.contains("<!doctype html"));
    }
}
