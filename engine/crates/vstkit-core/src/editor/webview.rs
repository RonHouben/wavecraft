//! Platform abstraction for WebView integration.
//!
//! Provides a platform-agnostic interface for creating and managing WebViews
//! across different platforms (macOS, Windows, Linux).

use std::any::Any;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::sync::{Arc, Mutex};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use nih_plug::prelude::*;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use vstkit_bridge::IpcHandler;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use vstkit_metering::MeterConsumer;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use super::bridge::PluginEditorBridge;

/// Platform-agnostic handle to a WebView instance.
///
/// This trait provides common operations that all platform implementations
/// must support: script evaluation, resizing, and lifecycle management.
pub trait WebViewHandle: Any + Send {
    /// Evaluate a JavaScript string in the WebView context.
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn evaluate_script(&self, script: &str) -> Result<(), String>;

    /// Resize the WebView to the given dimensions.
    ///
    /// Note: Called by platform implementations, not from trait consumers.
    /// The allow(dead_code) suppresses false positive from Rust's analysis.
    #[allow(dead_code)]
    fn resize(&self, width: u32, height: u32);

    /// Clean up resources (called on drop).
    ///
    /// Note: Called by platform implementations, not from trait consumers.
    /// The allow(dead_code) suppresses false positive from Rust's analysis.
    #[allow(dead_code)]
    fn close(&mut self);
}

/// Configuration for creating a WebView editor.
///
/// Generic over `P` which must implement nih-plug's `Params` trait.
///
/// Only used on macOS/Windows where WebView is available.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub struct WebViewConfig<P: Params> {
    pub params: Arc<P>,
    pub context: Arc<dyn GuiContext>,
    pub parent: ParentWindowHandle,
    pub width: u32,
    pub height: u32,
    /// Shared meter consumer - cloned from the plugin
    pub meter_consumer: Arc<Mutex<MeterConsumer>>,
    /// Shared editor size - updated on resize requests
    pub editor_size: Arc<Mutex<(u32, u32)>>,
}

/// Create a platform-specific WebView.
///
/// This function dispatches to the appropriate platform implementation
/// based on compile-time target OS.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn create_webview<P: Params>(
    _config: WebViewConfig<P>,
) -> Result<Box<dyn WebViewHandle>, String> {
    #[cfg(target_os = "macos")]
    {
        super::macos::create_macos_webview(_config)
    }

    #[cfg(target_os = "windows")]
    {
        super::windows::create_windows_webview(_config)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("Unsupported platform for WebView editor".to_string())
    }
}

/// Create an IPC handler for the WebView.
///
/// This is shared across all platforms and wires up the bridge between
/// the WebView's IPC messages and the nih-plug parameter system.
///
/// Generic over `P` which must implement nih-plug's `Params` trait.
///
/// Only used on macOS/Windows where WebView is available.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn create_ipc_handler<P: Params>(
    params: Arc<P>,
    context: Arc<dyn GuiContext>,
    meter_consumer: Arc<Mutex<MeterConsumer>>,
    editor_size: Arc<Mutex<(u32, u32)>>,
) -> IpcHandler<PluginEditorBridge<P>> {
    let bridge = PluginEditorBridge::new(params, context, meter_consumer, editor_size);
    IpcHandler::new(bridge)
}

/// IPC primitives JavaScript (injected before React loads).
///
/// This is the plugin-specific version for WKWebView, which uses
/// webkit.messageHandlers instead of wry's globalThis.ipc.
///
/// Only used on macOS/Windows where WebView is available.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub const IPC_PRIMITIVES_JS: &str = include_str!("js/ipc-primitives-plugin.js");
