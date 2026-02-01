//! Platform abstraction for WebView integration.
//!
//! Provides a platform-agnostic interface for creating and managing WebViews
//! across different platforms (macOS, Windows, Linux).

use std::any::Any;
use std::sync::{Arc, Mutex};

use bridge::IpcHandler;
use metering::MeterConsumer;
use nih_plug::prelude::*;

use crate::params::VstKitParams;

use super::bridge::PluginEditorBridge;

/// Platform-agnostic handle to a WebView instance.
///
/// This trait provides common operations that all platform implementations
/// must support: script evaluation, resizing, and lifecycle management.
pub trait WebViewHandle: Any + Send {
    /// Evaluate a JavaScript string in the WebView context.
    fn evaluate_script(&self, script: &str) -> Result<(), String>;

    /// Resize the WebView to the given dimensions.
    #[allow(dead_code)] // Platform trait completeness
    fn resize(&self, width: u32, height: u32);

    /// Clean up resources (called on drop).
    #[allow(dead_code)] // Platform trait completeness
    fn close(&mut self);
}

/// Configuration for creating a WebView editor.
#[allow(dead_code)] // Configuration struct for platform implementations
pub struct WebViewConfig {
    pub params: Arc<VstKitParams>,
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
pub fn create_webview(_config: WebViewConfig) -> Result<Box<dyn WebViewHandle>, String> {
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
#[allow(dead_code)] // Will be used when WebView editor is re-enabled
pub fn create_ipc_handler(
    params: Arc<VstKitParams>,
    context: Arc<dyn GuiContext>,
    meter_consumer: Arc<Mutex<MeterConsumer>>,
    editor_size: Arc<Mutex<(u32, u32)>>,
) -> IpcHandler<PluginEditorBridge> {
    let bridge = PluginEditorBridge::new(params, context, meter_consumer, editor_size);
    IpcHandler::new(bridge)
}

/// IPC primitives JavaScript (injected before React loads).
///
/// This is the plugin-specific version for WKWebView, which uses
/// webkit.messageHandlers instead of wry's globalThis.ipc.
#[allow(dead_code)] // Used conditionally per platform
pub const IPC_PRIMITIVES_JS: &str = include_str!("js/ipc-primitives-plugin.js");
