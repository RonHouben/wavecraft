//! Windows WebView implementation using WebView2.
//!
//! This module provides the platform-specific WebView integration for Windows,
//! using Microsoft Edge WebView2 embedded in the host-provided HWND.

use std::sync::{Arc, Mutex};

use bridge::IpcHandler;
use nih_plug::prelude::*;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, FillRect, HBRUSH, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassExW, CS_HREDRAW, CS_VREDRAW, HMENU,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_PAINT, WNDCLASSEXW, WS_CHILD, WS_VISIBLE,
};

use super::bridge::PluginEditorBridge;
use super::webview::{WebViewConfig, WebViewHandle};

/// Windows WebView handle.
///
/// We wrap the HWND in a Send-safe wrapper since we know it will only be
/// accessed from the main thread (the host ensures this).
pub struct WindowsWebView {
    hwnd: HWND,
    _handler: Arc<Mutex<IpcHandler<PluginEditorBridge>>>,
    webview_controller: Option<WebView2Controller>,
}

// SAFETY: The webview will only be accessed from the main thread.
// The host ensures this by calling spawn() and other methods on the main thread.
unsafe impl Send for WindowsWebView {}

impl WebViewHandle for WindowsWebView {
    fn evaluate_script(&self, script: &str) -> Result<(), String> {
        if let Some(controller) = &self.webview_controller {
            controller.evaluate_script(script)
        } else {
            Err("WebView2 controller not initialized".to_string())
        }
    }

    fn resize(&self, width: u32, height: u32) {
        if let Some(controller) = &self.webview_controller {
            controller.resize(width, height);
        }
    }

    fn close(&mut self) {
        self.webview_controller = None;
        // HWND cleanup is handled by the host
    }
}

/// WebView2 controller wrapper.
struct WebView2Controller {
    // TODO: Add actual WebView2 COM objects
    // For now, this is a placeholder
}

impl WebView2Controller {
    fn evaluate_script(&self, _script: &str) -> Result<(), String> {
        // TODO: Implement script evaluation via WebView2 COM API
        Ok(())
    }

    fn resize(&self, _width: u32, _height: u32) {
        // TODO: Implement resize via WebView2 COM API
    }
}

/// Create a Windows WebView editor.
pub fn create_windows_webview(config: WebViewConfig) -> Result<Box<dyn WebViewHandle>, String> {
    // Check if WebView2 runtime is available
    check_webview2_runtime()?;

    // Get the parent HWND from the raw handle
    let parent_hwnd = unsafe { get_parent_hwnd(config.parent)? };

    // Create IPC handler
    let handler = Arc::new(Mutex::new(super::webview::create_ipc_handler(
        config.params,
        config.context,
    )));

    // Create child window for WebView2
    let child_hwnd = unsafe { create_child_window(parent_hwnd, config.width, config.height)? };

    // For now, create a placeholder window until WebView2 is fully implemented
    // Full WebView2 integration requires async COM initialization
    Ok(Box::new(WindowsWebView {
        hwnd: child_hwnd,
        _handler: handler,
        webview_controller: None,
    }))
}

/// Check if WebView2 runtime is installed.
fn check_webview2_runtime() -> Result<(), String> {
    // Try to detect WebView2 runtime using webview2-com
    match webview2_com::get_available_core_webview2_browser_version_string(None) {
        Ok(version) => {
            if let Some(ver) = version {
                nih_log!("WebView2 runtime found: {}", ver);
                Ok(())
            } else {
                Err(format!(
                    "WebView2 runtime not installed. Please install it from:\n\
                     https://developer.microsoft.com/microsoft-edge/webview2/"
                ))
            }
        }
        Err(e) => Err(format!(
            "Failed to detect WebView2 runtime: {:?}\n\
             Please install it from:\n\
             https://developer.microsoft.com/microsoft-edge/webview2/",
            e
        )),
    }
}

/// Extract HWND from ParentWindowHandle.
unsafe fn get_parent_hwnd(handle: ParentWindowHandle) -> Result<HWND, String> {
    match handle {
        ParentWindowHandle::Win32Hwnd(hwnd_ptr) => {
            if hwnd_ptr.is_null() {
                return Err("Parent HWND is null".to_string());
            }
            Ok(HWND(hwnd_ptr as isize))
        }
        _ => Err("Expected Win32Hwnd parent on Windows".to_string()),
    }
}

/// Create a child window for hosting the WebView2 control.
unsafe fn create_child_window(
    parent: HWND,
    width: u32,
    height: u32,
) -> Result<HWND, String> {
    // Register window class
    let class_name = windows::core::w!("VstKitWebView");
    let h_instance = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandleW failed: {}", e))?;

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance.into(),
        hIcon: Default::default(),
        hCursor: Default::default(),
        hbrBackground: HBRUSH((6) as isize), // COLOR_WINDOW + 1
        lpszMenuName: windows::core::PCWSTR::null(),
        lpszClassName: class_name,
        hIconSm: Default::default(),
    };

    let atom = RegisterClassExW(&wc);
    if atom == 0 {
        // Class might already be registered, which is fine
        nih_log!("Window class registration returned 0 (may already exist)");
    }

    // Create child window
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        class_name,
        windows::core::w!("VstKit Editor"),
        WS_CHILD | WS_VISIBLE,
        0,
        0,
        width as i32,
        height as i32,
        parent,
        HMENU::default(),
        h_instance,
        None,
    )
    .map_err(|e| format!("CreateWindowExW failed: {}", e))?;

    Ok(hwnd)
}

/// Window procedure for the child window.
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if !hdc.is_invalid() {
                // Paint placeholder background
                let mut rect = RECT::default();
                let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rect);
                let brush = windows::Win32::Graphics::Gdi::GetSysColorBrush(
                    windows::Win32::UI::WindowsAndMessaging::COLOR_WINDOW,
                );
                let _ = FillRect(hdc, &rect, brush);

                // TODO: Draw "Loading..." text or WebView2 content
            }
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
