// IPC Primitives for Plugin WebView - Injected by Rust into WKWebView
//
// This is adapted for WKWebView's webkit.messageHandlers API instead of wry's globalThis.ipc

(function () {
  'use strict';

  // Queue to store incoming messages from Rust before receive callback is set
  const messageQueue = [];
  let receiveCallback = null;
  
  // Listeners for pushed parameter updates from Rust
  const paramUpdateListeners = [];

  // Internal receive handler called by Rust via WKWebView script evaluation
  function _receive(message) {
    if (receiveCallback) {
      receiveCallback(message);
    } else {
      // Buffer messages until receive callback is registered
      messageQueue.push(message);
    }
  }
  
  // Internal handler for parameter updates pushed from Rust
  function _onParamUpdate(message) {
    // Parse the message (it's a JSON-RPC notification)
    let notification;
    try {
      notification = typeof message === 'string' ? JSON.parse(message) : message;
    } catch (e) {
      console.error('[VSTKIT_IPC] Failed to parse param update:', e);
      return;
    }
    
    // Notify all registered listeners
    paramUpdateListeners.forEach(listener => {
      try {
        listener(notification);
      } catch (e) {
        console.error('[VSTKIT_IPC] Listener error:', e);
      }
    });
  }

  // Public API exposed to TypeScript layer
  const api = {
    /**
     * Send a message to Rust
     * @param {string} message - JSON-encoded IPC request
     */
    postMessage: function (message) {
      if (typeof message !== 'string') {
        throw new TypeError('postMessage requires a string argument');
      }
      // WKWebView's IPC bridge: webkit.messageHandlers.ipcHandler.postMessage
      if (globalThis.webkit && globalThis.webkit.messageHandlers && globalThis.webkit.messageHandlers.ipcHandler) {
        globalThis.webkit.messageHandlers.ipcHandler.postMessage(message);
      } else {
        console.error('[VSTKIT_IPC] globalThis.webkit.messageHandlers.ipcHandler not available');
      }
    },

    /**
     * Set callback to receive messages from Rust
     * @param {function(string): void} callback - Called with JSON-encoded IPC response
     */
    setReceiveCallback: function (callback) {
      if (typeof callback !== 'function') {
        throw new TypeError('setReceiveCallback requires a function argument');
      }

      receiveCallback = callback;

      // Flush queued messages
      while (messageQueue.length > 0) {
        const msg = messageQueue.shift();
        callback(msg);
      }
    },

    /**
     * Subscribe to parameter update notifications
     * @param {function(object): void} listener - Called with parsed JSON-RPC notification
     */
    onParamUpdate: function (listener) {
      if (typeof listener !== 'function') {
        throw new TypeError('onParamUpdate requires a function argument');
      }
      paramUpdateListeners.push(listener);
    },

    // Internal methods (exposed for Rust to call)
    _receive: _receive,
    _onParamUpdate: _onParamUpdate,
  };

  // Freeze and expose
  Object.freeze(api);
  window.__VSTKIT_IPC__ = api;

  console.log('[VSTKIT_IPC] Plugin IPC primitives loaded (WKWebView mode)');
})();
