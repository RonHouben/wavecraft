//! Windows WebView implementation using WebView2.
//!
//! This module provides the platform-specific WebView integration for Windows,
//! using Microsoft Edge WebView2 embedded in the host-provided HWND.

use std::sync::{Arc, Mutex, Once};

use nih_plug::prelude::*;
use wavecraft_bridge::IpcHandler;
use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2, ICoreWebView2Controller};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use windows::core::HSTRING;

use super::bridge::PluginEditorBridge;
use super::webview::{WebViewConfig, WebViewHandle};

mod content;
mod ipc_bridge;
mod runtime_checks;
mod webview2_init;

/// Trait for handling IPC JSON messages (type-erased interface).
pub(super) trait JsonIpcHandler: Send + Sync {
    fn handle_json(&self, json: &str) -> String;
}

// Implement for IpcHandler with any ParameterHost
impl<H: wavecraft_bridge::ParameterHost> JsonIpcHandler for IpcHandler<H> {
    fn handle_json(&self, json: &str) -> String {
        IpcHandler::handle_json(self, json)
    }
}

/// Windows WebView handle.
///
/// Holds the WebView2 controller and web view instances.
pub struct WindowsWebView {
    /// Parent window handle, retained for potential focus/positioning operations.
    ///
    /// Note: Currently unused but required for future Windows platform features.
    #[allow(dead_code)]
    hwnd: HWND,
    _handler: Arc<Mutex<dyn JsonIpcHandler>>,
    controller: Arc<Mutex<Option<ICoreWebView2Controller>>>,
    webview: Arc<Mutex<Option<ICoreWebView2>>>,
}

// SAFETY: The webview will only be accessed from the main thread.
// The host ensures this by calling spawn() and other methods on the main thread.
unsafe impl Send for WindowsWebView {}

impl WebViewHandle for WindowsWebView {
    fn evaluate_script(&self, script: &str) -> Result<(), String> {
        let webview_lock = self.webview.lock().unwrap();
        if let Some(webview) = webview_lock.as_ref() {
            let script_hstring = HSTRING::from(script);
            unsafe {
                webview
                    .ExecuteScript(&script_hstring, None)
                    .map_err(|e| format!("ExecuteScript failed: {:?}", e))?;
            }
            Ok(())
        } else {
            Err("WebView2 not initialized".to_string())
        }
    }

    fn resize(&self, width: u32, height: u32) {
        let controller_lock = self.controller.lock().unwrap();
        if let Some(controller) = controller_lock.as_ref() {
            let bounds = RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            unsafe {
                let _ = controller.SetBounds(bounds);
            }
        }
    }

    fn close(&mut self) {
        let mut controller_lock = self.controller.lock().unwrap();
        if let Some(controller) = controller_lock.take() {
            unsafe {
                let _ = controller.Close();
            }
        }
        let mut webview_lock = self.webview.lock().unwrap();
        *webview_lock = None;
    }
}

/// Create a Windows WebView editor.
pub fn create_windows_webview<P: Params + 'static>(
    config: WebViewConfig<P>,
) -> Result<Box<dyn WebViewHandle>, String> {
    static COM_INIT: Once = Once::new();
    COM_INIT.call_once(|| unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    });

    runtime_checks::check_webview2_runtime()?;

    let parent_hwnd = unsafe { runtime_checks::get_parent_hwnd(config.parent)? };

    let handler: Arc<Mutex<IpcHandler<PluginEditorBridge<P>>>> =
        Arc::new(Mutex::new(super::webview::create_ipc_handler(
            config.params,
            config.context,
            config.meter_consumer,
            config.oscilloscope_consumer,
            config.editor_size,
        )));

    // Convert to trait object for type erasure
    let handler_trait: Arc<Mutex<dyn JsonIpcHandler>> = handler;

    let child_hwnd =
        unsafe { runtime_checks::create_child_window(parent_hwnd, config.width, config.height)? };

    let controller: Arc<Mutex<Option<ICoreWebView2Controller>>> = Arc::new(Mutex::new(None));
    let webview: Arc<Mutex<Option<ICoreWebView2>>> = Arc::new(Mutex::new(None));

    webview2_init::initialize_webview2(
        child_hwnd,
        controller.clone(),
        webview.clone(),
        handler_trait.clone(),
        config.width,
        config.height,
    )?;

    Ok(Box::new(WindowsWebView {
        hwnd: child_hwnd,
        _handler: handler_trait,
        controller,
        webview,
    }))
}
