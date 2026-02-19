//! Windows WebView content loading and IPC script injection helpers.

use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
use windows::core::HSTRING;

use super::super::assets;

/// Inject IPC primitives JavaScript into the WebView.
pub(super) fn inject_ipc_script(webview: &ICoreWebView2) {
    let ipc_js = get_windows_ipc_primitives();
    let js_hstring = HSTRING::from(ipc_js);
    unsafe {
        let _ = webview.AddScriptToExecuteOnDocumentCreated(&js_hstring, None);
    }
}

/// Load UI content into the WebView.
pub(super) fn load_ui_content(webview: &ICoreWebView2) {
    // Try to load index.html from embedded assets
    if let Some((html_bytes, _mime)) = assets::get_asset("index.html") {
        let html_string = String::from_utf8_lossy(html_bytes);
        let ns_html = HSTRING::from(html_string.as_ref());
        unsafe {
            let _ = webview.NavigateToString(&ns_html);
        }
    } else {
        // Fallback HTML
        let fallback_html = HSTRING::from(get_fallback_html());
        unsafe {
            let _ = webview.NavigateToString(&fallback_html);
        }
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
      console.error('[WAVECRAFT_IPC] Failed to parse param update:', e);
      return;
    }
    
    paramUpdateListeners.forEach(listener => {
      try {
        listener(notification);
      } catch (e) {
        console.error('[WAVECRAFT_IPC] Listener error:', e);
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
        console.error('[WAVECRAFT_IPC] window.chrome.webview not available');
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
  globalThis.__WAVECRAFT_IPC__ = api;

  console.log('[WAVECRAFT_IPC] Plugin IPC primitives loaded (WebView2 mode)');
})();
"#
}

/// Fallback HTML when UI assets are not built.
fn get_fallback_html() -> &'static str {
    r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
    <title>Wavecraft Plugin</title>
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
    <h1>Wavecraft WebView Editor</h1>
    <p>UI assets not found. Please build the React UI first:</p>
    <pre>cd ui && npm run build</pre>
</body>
</html>"#
}
