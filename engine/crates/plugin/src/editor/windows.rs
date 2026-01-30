//! Windows WebView implementation stub.
//!
//! This module will provide the platform-specific WebView integration for Windows
//! using WebView2. Currently a stub for Phase 7.

use super::webview::{WebViewConfig, WebViewHandle};

/// Windows WebView handle (stub).
pub struct WindowsWebView;

impl WebViewHandle for WindowsWebView {
    fn evaluate_script(&self, _script: &str) -> Result<(), String> {
        Err("Windows WebView not yet implemented".to_string())
    }

    fn resize(&self, _width: u32, _height: u32) {
        // Stub
    }

    fn close(&mut self) {
        // Stub
    }
}

/// Create a Windows WebView editor (stub).
pub fn create_windows_webview(_config: WebViewConfig) -> Result<Box<dyn WebViewHandle>, String> {
    Err("Windows WebView not yet implemented (Phase 7)".to_string())
}
