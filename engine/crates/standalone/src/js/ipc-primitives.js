// IPC Primitives - Injected by Rust into WebView
//
// This script provides the minimal low-level interface between the WebView
// and Rust. It exposes a frozen `window.__VSTKIT_IPC__` object with methods
// to send messages to Rust and receive messages from Rust.
//
// DO NOT MODIFY THIS FILE without understanding the security implications.
// This code runs with full WebView privileges.

(function () {
  'use strict';

  // Queue to store incoming messages from Rust before receive callback is set
  const messageQueue = [];
  let receiveCallback = null;
  
  // Listeners for pushed parameter updates from Rust
  const paramUpdateListeners = [];

  // Internal receive handler called by Rust via wry's IPC
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
      // wry's IPC bridge: globalThis.ipc.postMessage sends to Rust
      if (globalThis.ipc && globalThis.ipc.postMessage) {
        globalThis.ipc.postMessage(message);
      } else {
        console.error('[VSTKIT_IPC] globalThis.ipc.postMessage not available');
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
        const message = messageQueue.shift();
        callback(message);
      }
    },
    
    /**
     * Register a listener for parameter updates pushed from Rust
     * @param {function(object): void} listener - Called with parameter update notification
     * @returns {function(): void} Unsubscribe function
     */
    onParamUpdate: function (listener) {
      if (typeof listener !== 'function') {
        throw new TypeError('onParamUpdate requires a function argument');
      }
      
      paramUpdateListeners.push(listener);
      
      // Return unsubscribe function
      return function () {
        const index = paramUpdateListeners.indexOf(listener);
        if (index !== -1) {
          paramUpdateListeners.splice(index, 1);
        }
      };
    },

    /**
     * Internal receive handler (called by Rust)
     * @private
     */
    _receive: _receive,
    
    /**
     * Internal parameter update handler (called by Rust)
     * @private
     */
    _onParamUpdate: _onParamUpdate,
  };

  // Freeze API to prevent tampering
  Object.freeze(api);

  // Expose on window
  window.__VSTKIT_IPC__ = api;

  console.log('[VSTKIT_IPC] Primitives loaded');
})();
