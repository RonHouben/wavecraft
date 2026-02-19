//! Windows WebView2 IPC bridge wiring.
//!
//! Owns web-message handler registration and request/response forwarding glue.

use std::sync::{Arc, Mutex};

use nih_plug::prelude::*;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2, ICoreWebView2WebMessageReceivedEventArgs,
};
use webview2_com::WebMessageReceivedEventHandler;
use windows::core::HSTRING;

use super::JsonIpcHandler;

/// Set up the web message handler for IPC communication.
pub(super) fn setup_web_message_handler(
    webview: &ICoreWebView2,
    handler: Arc<Mutex<dyn JsonIpcHandler>>,
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
                        "globalThis.__WAVECRAFT_IPC__._receive('{}');",
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
