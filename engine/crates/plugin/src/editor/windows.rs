//! Windows WebView implementation using WebView2.
//!
//! This module provides the platform-specific WebView integration for Windows,
//! using Microsoft Edge WebView2 embedded in the host-provided HWND.

use std::sync::{Arc, Mutex, Once};

use bridge::IpcHandler;
use nih_plug::prelude::*;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL, ICoreWebView2, ICoreWebView2Controller,
    ICoreWebView2WebMessageReceivedEventArgs,
};
use webview2_com::{
    CreateCoreWebView2ControllerCompletedHandler, CreateCoreWebView2EnvironmentCompletedHandler,
    WebMessageReceivedEventHandler, WebResourceRequestedEventHandler,
};
use windows::Win32::Foundation::{E_FAIL, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, FillRect, HBRUSH, PAINTSTRUCT};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect,
    HMENU, MSG, PM_REMOVE, PeekMessageW, RegisterClassExW, TranslateMessage, WINDOW_EX_STYLE,
    WM_PAINT, WNDCLASSEXW, WS_CHILD, WS_VISIBLE,
};
use windows::core::{HSTRING, PCWSTR};

use super::assets;
use super::bridge::PluginEditorBridge;
use super::webview::{WebViewConfig, WebViewHandle};

/// Windows WebView handle.
///
/// Holds the WebView2 controller and web view instances.
pub struct WindowsWebView {
    #[allow(dead_code)]
    hwnd: HWND,
    _handler: Arc<Mutex<IpcHandler<PluginEditorBridge>>>,
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
pub fn create_windows_webview(config: WebViewConfig) -> Result<Box<dyn WebViewHandle>, String> {
    static COM_INIT: Once = Once::new();
    COM_INIT.call_once(|| unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    });

    check_webview2_runtime()?;

    let parent_hwnd = unsafe { get_parent_hwnd(config.parent)? };

    let handler = Arc::new(Mutex::new(super::webview::create_ipc_handler(
        config.params,
        config.context,
        config.meter_consumer,
        config.editor_size,
        config.params.clone(),
        config.context.clone(),
        config.meter_consumer.clone(),
    )));

    let child_hwnd = unsafe { create_child_window(parent_hwnd, config.width, config.height)? };

    let controller: Arc<Mutex<Option<ICoreWebView2Controller>>> = Arc::new(Mutex::new(None));
    let webview: Arc<Mutex<Option<ICoreWebView2>>> = Arc::new(Mutex::new(None));

    initialize_webview2(
        child_hwnd,
        controller.clone(),
        webview.clone(),
        handler.clone(),
        config.width,
        config.height,
    )?;

    Ok(Box::new(WindowsWebView {
        hwnd: child_hwnd,
        _handler: handler,
        controller,
        webview,
    }))
}

/// Initialize WebView2 using the COM API.
fn initialize_webview2(
    hwnd: HWND,
    controller: Arc<Mutex<Option<ICoreWebView2Controller>>>,
    webview: Arc<Mutex<Option<ICoreWebView2>>>,
    handler: Arc<Mutex<IpcHandler<PluginEditorBridge>>>,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let user_data_folder = std::env::temp_dir().join("VstKit_WebView2_Data");
    let user_data_path = HSTRING::from(user_data_folder.to_string_lossy().as_ref());

    let controller_clone = controller.clone();
    let webview_clone = webview.clone();
    let handler_clone = handler;

    let env_handler = CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(
        move |result, environment| {
            if result.is_err() {
                nih_error!("Failed to create WebView2 environment: {:?}", result);
                return E_FAIL;
            }

            let environment = match environment {
                Some(env) => env,
                None => {
                    nih_error!("WebView2 environment is null");
                    return E_FAIL;
                }
            };

            let controller_inner = controller_clone.clone();
            let webview_inner = webview_clone.clone();
            let handler_inner = handler_clone.clone();

            let controller_handler = CreateCoreWebView2ControllerCompletedHandler::create(
                Box::new(move |result, ctrl| {
                    if result.is_err() {
                        nih_error!("Failed to create WebView2 controller: {:?}", result);
                        return E_FAIL;
                    }

                    let ctrl = match ctrl {
                        Some(c) => c,
                        None => {
                            nih_error!("WebView2 controller is null");
                            return E_FAIL;
                        }
                    };

                    let bounds = RECT {
                        left: 0,
                        top: 0,
                        right: width as i32,
                        bottom: height as i32,
                    };
                    unsafe {
                        let _ = ctrl.SetBounds(bounds);
                        let _ = ctrl.SetIsVisible(true);
                    }

                    let wv = unsafe {
                        match ctrl.CoreWebView2() {
                            Ok(w) => w,
                            Err(e) => {
                                nih_error!("Failed to get CoreWebView2: {:?}", e);
                                return E_FAIL;
                            }
                        }
                    };

                    configure_webview_settings(&wv);
                    setup_web_message_handler(&wv, handler_inner.clone());
                    setup_url_scheme_handler(&wv);
                    inject_ipc_script(&wv);
                    load_ui_content(&wv);

                    {
                        let mut ctrl_lock = controller_inner.lock().unwrap();
                        *ctrl_lock = Some(ctrl.clone());
                    }
                    {
                        let mut wv_lock = webview_inner.lock().unwrap();
                        *wv_lock = Some(wv);
                    }

                    nih_log!("WebView2 initialized successfully");
                    windows::core::HRESULT(0)
                }),
            );

            unsafe {
                let _ = environment.CreateCoreWebView2Controller(hwnd, &controller_handler);
            }

            windows::core::HRESULT(0)
        },
    ));

    unsafe {
        webview2_com::CreateCoreWebView2EnvironmentWithOptions(
            PCWSTR::null(),
            PCWSTR::from_raw(user_data_path.as_ptr()),
            None,
            &env_handler,
        )
        .map_err(|e| format!("CreateCoreWebView2Environment failed: {:?}", e))?;
    }

    pump_messages_until_ready(&webview, std::time::Duration::from_secs(5));

    Ok(())
}

/// Configure WebView2 settings.
fn configure_webview_settings(webview: &ICoreWebView2) {
    unsafe {
        if let Ok(settings) = webview.Settings() {
            #[cfg(debug_assertions)]
            let _ = settings.SetAreDevToolsEnabled(true);
            #[cfg(not(debug_assertions))]
            let _ = settings.SetAreDevToolsEnabled(false);
            #[cfg(not(debug_assertions))]
            let _ = settings.SetAreDefaultContextMenusEnabled(false);
            let _ = settings.SetIsScriptEnabled(true);
            let _ = settings.SetIsWebMessageEnabled(true);
        }
    }
}

/// Set up the web message handler for IPC communication.
fn setup_web_message_handler(
    webview: &ICoreWebView2,
    handler: Arc<Mutex<IpcHandler<PluginEditorBridge>>>,
) {
    let webview_clone = webview.clone();

    let message_handler = WebMessageReceivedEventHandler::create(Box::new(
        move |_webview, args: Option<ICoreWebView2WebMessageReceivedEventArgs>| {
            if let Some(args) = args {
                unsafe {
                    let message = match args.TryGetWebMessageAsString() {
                        Ok(msg) => msg.to_string(),
                        Err(_) => return windows::core::HRESULT(0),
                    };

                    nih_trace!("[IPC] Received message: {}", message);

                    let response = {
                        let handler = handler.lock().unwrap();
                        handler.handle_json(&message)
                    };

                    nih_trace!("[IPC] Response: {}", response);

                    let escaped_response = response
                        .replace('\\', "\\\\")
                        .replace('\'', "\\'")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r");

                    let js_code = format!(
                        "globalThis.__VSTKIT_IPC__._receive('{}');",
                        escaped_response
                    );
                    let js_hstring = HSTRING::from(&js_code);
                    let _ = webview_clone.ExecuteScript(&js_hstring, None);
                }
            }
            windows::core::HRESULT(0)
        },
    ));

    unsafe {
        let mut token = windows::Win32::System::WinRT::EventRegistrationToken::default();
        let _ = webview.add_WebMessageReceived(&message_handler, &mut token);
    }
}

/// Set up custom URL scheme handler for vstkit:// protocol.
fn setup_url_scheme_handler(webview: &ICoreWebView2) {
    let filter = HSTRING::from("vstkit://*");

    let request_handler =
        WebResourceRequestedEventHandler::create(Box::new(move |_webview, args| {
            if let Some(args) = args {
                unsafe {
                    let request = match args.Request() {
                        Ok(r) => r,
                        Err(_) => return windows::core::HRESULT(0),
                    };

                    let uri = match request.Uri() {
                        Ok(u) => u.to_string(),
                        Err(_) => return windows::core::HRESULT(0),
                    };

                    nih_trace!("[Asset Handler] Request: {}", uri);

                    let asset_path = uri
                        .strip_prefix("vstkit://localhost/")
                        .or_else(|| uri.strip_prefix("vstkit://"))
                        .unwrap_or(&uri);

                    if let Some((bytes, mime_type)) = assets::get_asset(asset_path) {
                        nih_trace!(
                            "[Asset Handler] Found asset: {} ({} bytes, {})",
                            asset_path,
                            bytes.len(),
                            mime_type
                        );
                        // Note: Full response creation requires IStream implementation
                        // For now, we load HTML via NavigateToString instead
                    } else {
                        nih_error!("[Asset Handler] Asset not found: {}", asset_path);
                    }
                }
            }
            windows::core::HRESULT(0)
        }));

    unsafe {
        let mut token = windows::Win32::System::WinRT::EventRegistrationToken::default();
        let _ =
            webview.AddWebResourceRequestedFilter(&filter, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL);
        let _ = webview.add_WebResourceRequested(&request_handler, &mut token);
    }
}

/// Inject IPC primitives JavaScript before page load.
fn inject_ipc_script(webview: &ICoreWebView2) {
    let ipc_script = get_windows_ipc_primitives();
    let script_hstring = HSTRING::from(ipc_script);

    unsafe {
        let _ = webview.AddScriptToExecuteOnDocumentCreated(&script_hstring, None);
    }
}

/// Load the UI content into the WebView.
fn load_ui_content(webview: &ICoreWebView2) {
    if let Some((html_bytes, _)) = assets::get_asset("index.html") {
        let html = String::from_utf8_lossy(html_bytes);
        let html_hstring = HSTRING::from(html.as_ref());
        unsafe {
            let _ = webview.NavigateToString(&html_hstring);
        }
    } else {
        let fallback_html = get_fallback_html();
        let html_hstring = HSTRING::from(fallback_html);
        unsafe {
            let _ = webview.NavigateToString(&html_hstring);
        }
    }
}

/// Check if WebView2 runtime is installed.
fn check_webview2_runtime() -> Result<(), String> {
    match webview2_com::get_available_core_webview2_browser_version_string(None) {
        Ok(version) => {
            if let Some(ver) = version {
                nih_log!("WebView2 runtime found: {}", ver);
                Ok(())
            } else {
                Err("WebView2 runtime not installed. Please install it from:\n\
                     https://developer.microsoft.com/microsoft-edge/webview2/"
                    .to_string())
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
unsafe fn create_child_window(parent: HWND, width: u32, height: u32) -> Result<HWND, String> {
    let class_name = windows::core::w!("VstKitWebView");
    let h_instance =
        GetModuleHandleW(None).map_err(|e| format!("GetModuleHandleW failed: {}", e))?;

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance.into(),
        hIcon: Default::default(),
        hCursor: Default::default(),
        hbrBackground: HBRUSH(6_isize),
        lpszMenuName: windows::core::PCWSTR::null(),
        lpszClassName: class_name,
        hIconSm: Default::default(),
    };

    let atom = RegisterClassExW(&wc);
    if atom == 0 {
        nih_log!("Window class registration returned 0 (may already exist)");
    }

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
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                let brush = windows::Win32::Graphics::Gdi::GetSysColorBrush(
                    windows::Win32::UI::WindowsAndMessaging::COLOR_WINDOW,
                );
                let _ = FillRect(hdc, &rect, brush);
            }
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Pump Windows messages until WebView2 is ready or timeout.
fn pump_messages_until_ready(
    webview: &Arc<Mutex<Option<ICoreWebView2>>>,
    timeout: std::time::Duration,
) {
    let start = std::time::Instant::now();

    loop {
        {
            let lock = webview.lock().unwrap();
            if lock.is_some() {
                break;
            }
        }

        if start.elapsed() > timeout {
            nih_error!("WebView2 initialization timed out");
            break;
        }

        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_QUIT {
                    break;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// Windows-specific IPC primitives that use window.chrome.webview.postMessage.
fn get_windows_ipc_primitives() -> &'static str {
    r#"
(function () {
  'use strict';

  const messageQueue = [];
  let receiveCallback = null;
  const paramUpdateListeners = [];

  function _receive(message) {
    if (receiveCallback) {
      receiveCallback(message);
    } else {
      messageQueue.push(message);
    }
  }
  
  function _onParamUpdate(message) {
    let notification;
    try {
      notification = typeof message === 'string' ? JSON.parse(message) : message;
    } catch (e) {
      console.error('[VSTKIT_IPC] Failed to parse param update:', e);
      return;
    }
    
    paramUpdateListeners.forEach(listener => {
      try {
        listener(notification);
      } catch (e) {
        console.error('[VSTKIT_IPC] Listener error:', e);
      }
    });
  }

  const api = {
    postMessage: function (message) {
      if (typeof message !== 'string') {
        throw new TypeError('postMessage requires a string argument');
      }
      if (window.chrome && window.chrome.webview) {
        window.chrome.webview.postMessage(message);
      } else {
        console.error('[VSTKIT_IPC] window.chrome.webview not available');
      }
    },

    setReceiveCallback: function (callback) {
      if (typeof callback !== 'function') {
        throw new TypeError('setReceiveCallback requires a function argument');
      }

      receiveCallback = callback;

      while (messageQueue.length > 0) {
        const msg = messageQueue.shift();
        callback(msg);
      }
    },

    onParamUpdate: function (listener) {
      if (typeof listener !== 'function') {
        throw new TypeError('onParamUpdate requires a function argument');
      }
      paramUpdateListeners.push(listener);
    },

    _receive: _receive,
    _onParamUpdate: _onParamUpdate,
  };

  Object.freeze(api);
  window.__VSTKIT_IPC__ = api;

  console.log('[VSTKIT_IPC] Plugin IPC primitives loaded (WebView2 mode)');
})();
"#
}

/// Fallback HTML when UI assets are not built.
fn get_fallback_html() -> &'static str {
    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>VstKit Plugin</title>
    <style>
        body { 
            margin: 0; 
            padding: 20px; 
            font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
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
</html>"#
}
