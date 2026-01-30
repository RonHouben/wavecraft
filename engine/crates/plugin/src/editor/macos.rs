//! macOS WebView implementation using WKWebView.
//!
//! This module provides the platform-specific WebView integration for macOS,
//! using WKWebView embedded in the host-provided NSView.

use std::sync::{Arc, Mutex};

use nih_plug::prelude::*;
use objc2::rc::Retained;
use objc2_app_kit::NSView;
use objc2_foundation::MainThreadMarker;

use bridge::IpcHandler;

use super::bridge::PluginEditorBridge;
use super::webview::{WebViewConfig, WebViewHandle};

/// macOS WebView handle.
/// 
/// We wrap the NSView pointer in a Send-safe wrapper since we know
/// it will only be accessed from the main thread.
pub struct MacOSWebView {
    _handler: Arc<Mutex<IpcHandler<PluginEditorBridge>>>,
}

// SAFETY: The webview will only be accessed from the main thread
// The host ensures this by calling spawn() and other methods on the main thread
unsafe impl Send for MacOSWebView {}

impl WebViewHandle for MacOSWebView {
    fn evaluate_script(&self, _script: &str) -> Result<(), String> {
        // TODO: Implement script evaluation
        Ok(())
    }

    fn resize(&self, _width: u32, _height: u32) {
        // TODO: Implement resize
    }

    fn close(&mut self) {
        // TODO: Implement close
    }
}

/// Create a macOS WebView editor.
pub fn create_macos_webview(config: WebViewConfig) -> Result<Box<dyn WebViewHandle>, String> {
    let _mtm = MainThreadMarker::new()
        .ok_or_else(|| "WebView creation must happen on main thread".to_string())?;

    // Get the parent NSView from the raw handle
    let parent_view = unsafe { get_parent_view(config.parent)? };

    // Create IPC handler
    let handler = Arc::new(Mutex::new(super::webview::create_ipc_handler(
        config.params,
        config.context,
    )));

    // For now, just create a simple label to show that the editor is loading
    // Full WebView integration will be implemented after getting basic structure working
    create_placeholder_label(&parent_view)?;

    Ok(Box::new(MacOSWebView {
        _handler: handler,
    }))
}

/// Extract NSView from ParentWindowHandle.
unsafe fn get_parent_view(handle: ParentWindowHandle) -> Result<Retained<NSView>, String> {
    match handle {
        ParentWindowHandle::AppKitNsView(ns_view_ptr) => {
            if ns_view_ptr.is_null() {
                return Err("Parent NSView is null".to_string());
            }

            // SAFETY: We've checked that the pointer is non-null
            // The parent view is owned by the host, so we retain it to ensure it stays alive
            unsafe {
                let view = Retained::retain(ns_view_ptr as *mut NSView)
                    .ok_or_else(|| "Failed to retain parent NSView".to_string())?;
                Ok(view)
            }
        }
        _ => Err("Expected AppKitNsView parent on macOS".to_string()),
    }
}

/// Create a placeholder label to show editor is loading.
fn create_placeholder_label(parent_view: &NSView) -> Result<(), String> {
    let _mtm = MainThreadMarker::new()
        .ok_or_else(|| "Label creation must happen on main thread".to_string())?;

    // For now, just set the window background to confirm the editor is loaded
    // We'll add a proper label or WebView in the next iteration

    // Create a simple label using NSTextField
    use objc2::msg_send_id;
    use objc2::runtime::AnyObject;
    use objc2_foundation::{CGRect, NSString};

    let frame = CGRect {
        origin: objc2_foundation::CGPoint { x: 10.0, y: 10.0 },
        size: objc2_foundation::CGSize {
            width: 300.0,
            height: 30.0,
        },
    };

    unsafe {
        let ns_text_field_class = objc2::class!(NSTextField);
        let label: Retained<AnyObject> = msg_send_id![
            msg_send_id![ns_text_field_class, alloc],
            initWithFrame: frame
        ];

        let text = NSString::from_str("VstKit WebView Editor Loading...");
        let _: () = objc2::msg_send![&label, setStringValue: &*text];
        let _: () = objc2::msg_send![&label, setBezeled: false];
        let _: () = objc2::msg_send![&label, setDrawsBackground: false];
        let _: () = objc2::msg_send![&label, setEditable: false];
        let _: () = objc2::msg_send![&label, setSelectable: false];

        parent_view.addSubview(&*(label.as_ref() as *const AnyObject as *const NSView));
    }

    Ok(())
}


