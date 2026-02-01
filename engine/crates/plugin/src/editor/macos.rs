//! macOS WebView implementation using WKWebView.
//!
//! This module provides the platform-specific WebView integration for macOS,
//! using WKWebView embedded in the host-provided NSView.

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use nih_plug::prelude::*;
use objc2::rc::{Retained, Weak};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{ClassType, DeclaredClass, declare_class, msg_send, msg_send_id, mutability};
use objc2_app_kit::NSView;
use objc2_foundation::{
    MainThreadMarker, NSData, NSObject, NSObjectProtocol, NSString, NSURL, NSURLResponse,
};
use objc2_web_kit::{
    WKScriptMessage, WKScriptMessageHandler, WKURLSchemeHandler, WKURLSchemeTask,
    WKUserContentController, WKUserScript, WKUserScriptInjectionTime, WKWebView,
    WKWebViewConfiguration,
};

use bridge::IpcHandler;

use super::assets;
use super::bridge::PluginEditorBridge;
use super::webview::{WebViewConfig, WebViewHandle};

/// macOS WebView handle.
///
/// Holds the WKWebView and associated resources.
pub struct MacOSWebView {
    webview: Rc<Mutex<Option<Retained<WKWebView>>>>,
    _handler: Arc<Mutex<IpcHandler<PluginEditorBridge>>>,
}

// SAFETY: The webview will only be accessed from the main thread
// The host ensures this by calling spawn() and other methods on the main thread
unsafe impl Send for MacOSWebView {}

impl WebViewHandle for MacOSWebView {
    fn evaluate_script(&self, script: &str) -> Result<(), String> {
        let webview_lock = self.webview.lock().unwrap();
        if let Some(webview) = webview_lock.as_ref() {
            let js = NSString::from_str(script);
            unsafe {
                // Pass nil as completion handler
                let _: () = msg_send![webview, evaluateJavaScript:&*js completionHandler:std::ptr::null_mut::<AnyObject>()];
            }
            Ok(())
        } else {
            Err("WebView not initialized".to_string())
        }
    }

    fn resize(&self, width: u32, height: u32) {
        let webview_lock = self.webview.lock().unwrap();
        if let Some(webview) = webview_lock.as_ref() {
            let frame = objc2_foundation::CGRect {
                origin: objc2_foundation::CGPoint { x: 0.0, y: 0.0 },
                size: objc2_foundation::CGSize {
                    width: width as f64,
                    height: height as f64,
                },
            };
            unsafe {
                let _: () = msg_send![webview, setFrame: frame];
            }
        }
    }

    fn close(&mut self) {
        let mut webview_lock = self.webview.lock().unwrap();
        if let Some(webview) = webview_lock.take() {
            unsafe {
                let _: () = msg_send![&webview, removeFromSuperview];
            }
        }
    }
}

/// Create a macOS WebView editor.
pub fn create_macos_webview(config: WebViewConfig) -> Result<Box<dyn WebViewHandle>, String> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| "WebView creation must happen on main thread".to_string())?;

    // Get the parent NSView from the raw handle
    let parent_view = unsafe { get_parent_view(config.parent)? };

    // Create IPC handler
    let handler = Arc::new(Mutex::new(super::webview::create_ipc_handler(
        config.params,
        config.context,
        config.meter_consumer,
        config.editor_size,
    )));

    // Create WKWebView first (without configuration yet)
    let frame = objc2_foundation::CGRect {
        origin: objc2_foundation::CGPoint { x: 0.0, y: 0.0 },
        size: objc2_foundation::CGSize {
            width: config.width as f64,
            height: config.height as f64,
        },
    };

    // Create webview configuration with URL scheme handler
    let webview_config = create_webview_config(mtm)?;

    // Create the webview
    let webview =
        unsafe { WKWebView::initWithFrame_configuration(mtm.alloc(), frame, &webview_config) };

    // Enable autoresizing so the webview resizes with the parent
    #[allow(non_upper_case_globals)]
    const NSViewWidthSizable: u64 = 1 << 1;
    #[allow(non_upper_case_globals)]
    const NSViewHeightSizable: u64 = 1 << 4;

    unsafe {
        let autoresizing_mask = NSViewWidthSizable | NSViewHeightSizable;
        let _: () = msg_send![&webview, setAutoresizingMask: autoresizing_mask];
    }

    // Now configure the webview with IPC handler and scripts
    configure_webview(&webview, handler.clone(), mtm)?;

    // Add webview as subview of parent
    unsafe {
        parent_view.addSubview(&webview);
    }

    // Load the UI
    load_ui(&webview)?;

    Ok(Box::new(MacOSWebView {
        webview: Rc::new(Mutex::new(Some(webview))),
        _handler: handler,
    }))
}

/// Create WKWebView configuration with URL scheme handler.
fn create_webview_config(
    mtm: MainThreadMarker,
) -> Result<Retained<WKWebViewConfiguration>, String> {
    let config = unsafe { WKWebViewConfiguration::new() };

    // Register custom URL scheme handler for vstkit://
    let scheme_handler = AssetSchemeHandler::new(mtm);
    let scheme_name = NSString::from_str("vstkit");

    unsafe {
        config.setURLSchemeHandler_forURLScheme(
            Some(ProtocolObject::from_ref(&*scheme_handler)),
            &scheme_name,
        );
    }

    Ok(config)
}

/// Configure the WKWebView with IPC handler and scripts.
fn configure_webview(
    webview: &Retained<WKWebView>,
    handler: Arc<Mutex<IpcHandler<PluginEditorBridge>>>,
    mtm: MainThreadMarker,
) -> Result<(), String> {
    // Get the configuration and user content controller
    let config = unsafe { webview.configuration() };
    let user_content_controller = unsafe { config.userContentController() };

    // Inject IPC primitives script
    let ipc_primitives = NSString::from_str(super::webview::IPC_PRIMITIVES_JS);
    let user_script = unsafe {
        WKUserScript::initWithSource_injectionTime_forMainFrameOnly(
            mtm.alloc(),
            &ipc_primitives,
            WKUserScriptInjectionTime::AtDocumentStart,
            true,
        )
    };
    unsafe {
        user_content_controller.addUserScript(&user_script);
    }

    // Create and add IPC message handler
    let message_handler = IpcMessageHandler::new(handler, webview, mtm);

    let handler_name = NSString::from_str("ipcHandler");
    unsafe {
        user_content_controller.addScriptMessageHandler_name(
            ProtocolObject::from_ref(&*message_handler),
            &handler_name,
        );
    }

    Ok(())
}

/// Load the UI into the WebView.
fn load_ui(webview: &WKWebView) -> Result<(), String> {
    // Try to load index.html from embedded assets
    if let Some((html_bytes, _mime)) = assets::get_asset("index.html") {
        let html_string = String::from_utf8_lossy(html_bytes);
        let ns_html = NSString::from_str(&html_string);

        // Create base URL for resolving relative paths
        let base_url_string = NSString::from_str("vstkit://localhost/");
        let base_url = unsafe { NSURL::URLWithString(&base_url_string) }
            .ok_or_else(|| "Failed to create base URL".to_string())?;

        unsafe {
            let _: () = msg_send![webview, loadHTMLString:&*ns_html baseURL:&*base_url];
        }

        Ok(())
    } else {
        // Fallback: Load a minimal HTML page
        let fallback_html = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>VstKit Plugin</title>
    <style>
        body { 
            margin: 0; 
            padding: 20px; 
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: #1e1e1e;
            color: #fff;
        }
        h1 { color: #61dafb; }
    </style>
</head>
<body>
    <h1>VstKit WebView Editor</h1>
    <p>UI assets not found. Please build the React UI first:</p>
    <pre>cd ui && npm run build</pre>
</body>
</html>
        "#;

        let ns_html = NSString::from_str(fallback_html);
        let base_url_string = NSString::from_str("about:blank");
        let base_url = unsafe { NSURL::URLWithString(&base_url_string) }
            .ok_or_else(|| "Failed to create base URL".to_string())?;

        unsafe {
            let _: () = msg_send![webview, loadHTMLString:&*ns_html baseURL:&*base_url];
        }

        Ok(())
    }
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

/// IPC message handler for WKWebView script messages.
struct IpcMessageHandlerIvars {
    handler: Arc<Mutex<IpcHandler<PluginEditorBridge>>>,
    webview: Weak<WKWebView>,
}

declare_class!(
    struct IpcMessageHandler;

    unsafe impl ClassType for IpcMessageHandler {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "VstKitIpcMessageHandler";
    }

    impl DeclaredClass for IpcMessageHandler {
        type Ivars = IpcMessageHandlerIvars;
    }

    unsafe impl NSObjectProtocol for IpcMessageHandler {}

    unsafe impl WKScriptMessageHandler for IpcMessageHandler {
        #[method(userContentController:didReceiveScriptMessage:)]
        #[allow(non_snake_case)] // Objective-C selector naming convention
        unsafe fn userContentController_didReceiveScriptMessage(
            &self,
            _controller: &WKUserContentController,
            message: &WKScriptMessage,
        ) {
            let vars = self.ivars();

            // Get message body as string
            // SAFETY: WKScriptMessage::body() is safe to call within this message handler context
            let body = unsafe { message.body() };

            // The body is an NSObject - try to convert it to a string
            let body_str: String = unsafe {
                let desc: Retained<NSString> = msg_send_id![&*body, description];
                desc.to_string()
            };

            nih_trace!("[IPC] Received message: {}", body_str);

            // Handle the IPC message
            let response = {
                let handler = vars.handler.lock().unwrap();
                handler.handle_json(&body_str)
            };

            nih_trace!("[IPC] Response: {}", response);

            // Send response back to WebView
            let escaped_response = response
                .replace('\\', "\\\\")
                .replace('\'', "\\'")
                .replace('\n', "\\n")
                .replace('\r', "\\r");

            let js_code = format!("globalThis.__VSTKIT_IPC__._receive('{}');", escaped_response);
            let js_string = NSString::from_str(&js_code);

            // Get webview from weak reference
            if let Some(wv) = vars.webview.load() {
                unsafe {
                    let _: () = msg_send![&*wv, evaluateJavaScript:&*js_string completionHandler:std::ptr::null_mut::<AnyObject>()];
                }
            }
        }
    }
);

impl IpcMessageHandler {
    fn new(
        handler: Arc<Mutex<IpcHandler<PluginEditorBridge>>>,
        webview: &Retained<WKWebView>,
        mtm: MainThreadMarker,
    ) -> Retained<Self> {
        let ivars = IpcMessageHandlerIvars {
            handler,
            webview: Weak::from_retained(webview),
        };
        unsafe {
            let this = mtm.alloc().set_ivars(ivars);
            msg_send_id![super(this), init]
        }
    }
}

/// URL Scheme handler for serving embedded assets.
struct AssetSchemeHandlerIvars {}

declare_class!(
    struct AssetSchemeHandler;

    unsafe impl ClassType for AssetSchemeHandler {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "VstKitAssetSchemeHandler";
    }

    impl DeclaredClass for AssetSchemeHandler {
        type Ivars = AssetSchemeHandlerIvars;
    }

    unsafe impl NSObjectProtocol for AssetSchemeHandler {}

    unsafe impl WKURLSchemeHandler for AssetSchemeHandler {
        #[method(webView:startURLSchemeTask:)]
        #[allow(non_snake_case)] // Objective-C selector naming convention
        unsafe fn webView_startURLSchemeTask(
            &self,
            _webview: &WKWebView,
            task: &ProtocolObject<dyn WKURLSchemeTask>,
        ) {
            // Get the request URL
            let request = unsafe { task.request() };
            let url = unsafe { request.URL() };

            if let Some(url) = url {
                let url_string = unsafe { url.absoluteString() };
                let path = if let Some(ns_str) = url_string {
                    ns_str.to_string()
                } else {
                    return; // No URL string, nothing to do
                };

                nih_trace!("[Asset Handler] Request: {}", path);

                // Remove the vstkit://localhost/ prefix
                let asset_path = path
                    .strip_prefix("vstkit://localhost/")
                    .unwrap_or(&path);

                // Try to load the asset
                if let Some((bytes, mime_type)) = assets::get_asset(asset_path) {
                    nih_trace!("[Asset Handler] Found asset: {} ({} bytes, {})", asset_path, bytes.len(), mime_type);

                    // Create NSData from bytes
                    // SAFETY: NSData::with_bytes is safe to call
                    let data = NSData::with_bytes(bytes);

                    // Create response
                    let response = unsafe {
                        let mime_string = NSString::from_str(mime_type);
                        let url_response: Retained<NSURLResponse> = msg_send_id![
                            msg_send_id![objc2::class!(NSURLResponse), alloc],
                            initWithURL: &*url
                            MIMEType: &*mime_string
                            expectedContentLength: bytes.len() as i64
                            textEncodingName: std::ptr::null::<NSString>()
                        ];
                        url_response
                    };

                    // Send response and data
                    unsafe {
                        task.didReceiveResponse(&response);
                        task.didReceiveData(&data);
                        task.didFinish();
                    }
                } else {
                    nih_error!("[Asset Handler] Asset not found: {}", asset_path);

                    // Send 404 error
                    let error = unsafe {
                        let error_domain = NSString::from_str("VstKitErrorDomain");
                        let error_code = 404i64;
                        let error: Retained<AnyObject> = msg_send_id![
                            msg_send_id![objc2::class!(NSError), alloc],
                            initWithDomain: &*error_domain
                            code: error_code
                            userInfo: std::ptr::null::<AnyObject>()
                        ];
                        error
                    };

                    unsafe {
                        let _: () = msg_send![task, didFailWithError: &*error];
                    }
                }
            }
        }

        #[method(webView:stopURLSchemeTask:)]
        #[allow(non_snake_case)] // Objective-C selector naming convention
        unsafe fn webView_stopURLSchemeTask(
            &self,
            _webview: &WKWebView,
            _task: &ProtocolObject<dyn WKURLSchemeTask>,
        ) {
            // Task cancelled - nothing to clean up
        }
    }
);

impl AssetSchemeHandler {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let ivars = AssetSchemeHandlerIvars {};
        unsafe {
            let this = mtm.alloc().set_ivars(ivars);
            msg_send_id![super(this), init]
        }
    }
}
