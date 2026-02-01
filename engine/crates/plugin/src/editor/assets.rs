//! Static asset embedding for the React UI in the plugin.
//!
//! This module embeds the built React application (`ui/dist/`) directly
//! into the plugin binary using `include_dir!`. Assets are served via a
//! custom protocol handler in the WebView.

#[cfg(any(target_os = "macos", target_os = "windows"))]
use include_dir::{Dir, include_dir};

/// Embedded UI assets from `ui/dist/`
///
/// This directory is populated by running `npm run build` in the `ui/` folder.
/// If the directory doesn't exist yet, we use an empty directory.
///
/// Only used on macOS/Windows where WebView is available.
#[cfg(any(target_os = "macos", target_os = "windows"))]
static UI_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../../ui/dist");

/// Get an embedded asset by path.
///
/// # Arguments
/// * `path` - Relative path within the `ui/dist/` directory (e.g., "index.html", "assets/main.js")
///
/// # Returns
/// * `Some((bytes, mime_type))` if asset exists
/// * `None` if asset not found
///
/// Only used on macOS/Windows where WebView is available.
#[cfg(any(target_os = "macos", target_os = "windows"))]
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

/// Infer MIME type from file extension.
///
/// Only used on macOS/Windows where WebView is available.
#[cfg(any(target_os = "macos", target_os = "windows"))]
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
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    use super::*;

    #[test]
    #[cfg(any(target_os = "macos", target_os = "windows"))]
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
}
