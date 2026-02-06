/**
 * Plugin IPC primitives for WKWebView.
 *
 * This script is injected into the WKWebView before React loads.
 * It provides the globalThis.__WAVECRAFT_IPC__ API that mirrors the WebSocket transport.
 *
 * Unlike the browser version (which uses WebSocket), this uses WKWebView's
 * webkit.messageHandlers mechanism for bidirectional communication.
 */

(function () {
  'use strict';

  // Queue for messages received before the callback is registered
  const messageQueue = [];
  let receiveCallback = null;

  // Parameter update notification listeners
  const paramUpdateListeners = [];

  /**
   * Internal function called by native code when a response arrives.
   * @param {string} message - JSON-RPC response string
   */
  function _receive(message) {
    if (receiveCallback) {
      receiveCallback(message);
    } else {
      messageQueue.push(message);
    }
  }

  /**
   * Internal function called by native code for parameter change notifications.
   * @param {Object|string} message - Parameter update notification
   */
  function _onParamUpdate(message) {
    let notification;
    try {
      notification = typeof message === 'string' ? JSON.parse(message) : message;
    } catch (e) {
      console.error('[WAVECRAFT_IPC] Failed to parse param update:', e);
      return;
    }

    paramUpdateListeners.forEach((listener) => {
      try {
        listener(notification);
      } catch (e) {
        console.error('[WAVECRAFT_IPC] Listener error:', e);
      }
    });
  }

  const api = {
    /**
     * Send a message to the native plugin.
     * @param {string} message - JSON-RPC request string
     */
    postMessage: function (message) {
      if (typeof message !== 'string') {
        throw new TypeError('postMessage requires a string argument');
      }

      // Use WKWebView's message handler
      if (
        globalThis.webkit &&
        globalThis.webkit.messageHandlers &&
        globalThis.webkit.messageHandlers.ipcHandler
      ) {
        globalThis.webkit.messageHandlers.ipcHandler.postMessage(message);
      } else {
        console.error('[WAVECRAFT_IPC] webkit.messageHandlers.ipcHandler not available');
      }
    },

    /**
     * Register a callback to receive responses from the native plugin.
     * @param {function} callback - Function to call with response string
     */
    setReceiveCallback: function (callback) {
      if (typeof callback !== 'function') {
        throw new TypeError('setReceiveCallback requires a function argument');
      }

      receiveCallback = callback;

      // Deliver any queued messages
      while (messageQueue.length > 0) {
        const msg = messageQueue.shift();
        callback(msg);
      }
    },

    /**
     * Register a listener for parameter update notifications.
     * @param {function} listener - Function to call with update object
     */
    onParamUpdate: function (listener) {
      if (typeof listener !== 'function') {
        throw new TypeError('onParamUpdate requires a function argument');
      }
      paramUpdateListeners.push(listener);
    },

    // Internal hooks for native code
    _receive: _receive,
    _onParamUpdate: _onParamUpdate,
  };

  // Freeze the API to prevent modification
  Object.freeze(api);
  globalThis.__WAVECRAFT_IPC__ = api;

  console.log('[WAVECRAFT_IPC] Plugin IPC primitives loaded (WKWebView mode)');
})();
