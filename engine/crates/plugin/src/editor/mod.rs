//! Plugin editor module - WebView and parameter integration.
//!
//! This module provides the nih-plug Editor implementation, bridging
//! the WebView UI with the plugin's parameter system and metering.

mod bridge;
mod egui;

pub use self::egui::create_egui_editor as create_editor;
pub use bridge::PluginEditorBridge;

// Platform-specific WebView implementations will be added here
// mod webview;
// mod macos;
// #[cfg(target_os = "windows")]
// mod windows;
