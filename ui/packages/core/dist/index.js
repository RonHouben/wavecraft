import { dbToLinear, linearToDb } from "./meters.js";
import { useState, useEffect, useCallback, useMemo } from "react";
function isWebViewEnvironment() {
  return globalThis.__WAVECRAFT_IPC__ !== void 0;
}
function isBrowserEnvironment() {
  return !isWebViewEnvironment();
}
const ERROR_PARSE = -32700;
const ERROR_INVALID_REQUEST = -32600;
const ERROR_METHOD_NOT_FOUND = -32601;
const ERROR_INVALID_PARAMS = -32602;
const ERROR_INTERNAL = -32603;
const ERROR_PARAM_NOT_FOUND = -32e3;
const ERROR_PARAM_OUT_OF_RANGE = -32001;
function isIpcResponse(obj) {
  return typeof obj === "object" && obj !== null && "jsonrpc" in obj && "id" in obj && ("result" in obj || "error" in obj);
}
function isIpcNotification(obj) {
  return typeof obj === "object" && obj !== null && "jsonrpc" in obj && "method" in obj && !("id" in obj);
}
const METHOD_GET_PARAMETER = "getParameter";
const METHOD_SET_PARAMETER = "setParameter";
const METHOD_GET_ALL_PARAMETERS = "getAllParameters";
const NOTIFICATION_PARAMETER_CHANGED = "parameterChanged";
var LogLevel = /* @__PURE__ */ ((LogLevel2) => {
  LogLevel2[LogLevel2["DEBUG"] = 0] = "DEBUG";
  LogLevel2[LogLevel2["INFO"] = 1] = "INFO";
  LogLevel2[LogLevel2["WARN"] = 2] = "WARN";
  LogLevel2[LogLevel2["ERROR"] = 3] = "ERROR";
  return LogLevel2;
})(LogLevel || {});
class Logger {
  constructor(options = {}) {
    this.minLevel = options.minLevel ?? 0;
  }
  /**
   * Set the minimum log level at runtime.
   */
  setMinLevel(level) {
    this.minLevel = level;
  }
  /**
   * Log debug message (verbose tracing).
   */
  debug(message, context) {
    if (this.minLevel <= 0) {
      console.debug(`[DEBUG] ${message}`, context ?? {});
    }
  }
  /**
   * Log informational message.
   */
  info(message, context) {
    if (this.minLevel <= 1) {
      console.info(`[INFO] ${message}`, context ?? {});
    }
  }
  /**
   * Log warning message.
   */
  warn(message, context) {
    if (this.minLevel <= 2) {
      console.warn(`[WARN] ${message}`, context ?? {});
    }
  }
  /**
   * Log error message.
   */
  error(message, context) {
    if (this.minLevel <= 3) {
      console.error(`[ERROR] ${message}`, context ?? {});
    }
  }
}
const logger = new Logger({
  minLevel: 1
  /* INFO */
});
class NativeTransport {
  constructor() {
    this.pendingRequests = /* @__PURE__ */ new Map();
    this.notificationCallbacks = /* @__PURE__ */ new Set();
    this.primitives = globalThis.__WAVECRAFT_IPC__;
    if (!this.primitives) {
      throw new Error(
        "NativeTransport: __WAVECRAFT_IPC__ primitives not found. Ensure this runs in a WKWebView with injected IPC."
      );
    }
    this.primitives.setReceiveCallback((message) => {
      this.handleIncomingMessage(message);
    });
    if (this.primitives.onParamUpdate) {
      this.primitives.onParamUpdate((notification) => {
        if (isIpcNotification(notification)) {
          this.handleNotification(notification);
        }
      });
    }
  }
  /**
   * Send a JSON-RPC request and wait for response
   */
  async send(request) {
    if (!this.primitives) {
      throw new Error("NativeTransport: Primitives not available");
    }
    const parsedRequest = JSON.parse(request);
    const id = parsedRequest.id;
    if (id === void 0 || id === null) {
      throw new Error("NativeTransport: Request must have an id");
    }
    const responsePromise = new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        this.pendingRequests.delete(id);
        reject(new Error(`Request timeout: ${parsedRequest.method}`));
      }, 5e3);
      this.pendingRequests.set(id, { resolve, reject, timeoutId });
    });
    this.primitives.postMessage(request);
    return responsePromise;
  }
  /**
   * Register a callback for incoming notifications
   */
  onNotification(callback) {
    this.notificationCallbacks.add(callback);
    return () => {
      this.notificationCallbacks.delete(callback);
    };
  }
  /**
   * Check if transport is connected (native is always connected)
   */
  isConnected() {
    return true;
  }
  /**
   * Clean up resources
   */
  dispose() {
    for (const [id, { reject, timeoutId }] of this.pendingRequests.entries()) {
      clearTimeout(timeoutId);
      reject(new Error("Transport disposed"));
      this.pendingRequests.delete(id);
    }
    this.notificationCallbacks.clear();
  }
  /**
   * Handle incoming message (response or notification)
   */
  handleIncomingMessage(message) {
    try {
      const parsed = JSON.parse(message);
      if (isIpcResponse(parsed)) {
        this.handleResponse(parsed);
      } else if (isIpcNotification(parsed)) {
        this.handleNotification(parsed);
      }
    } catch (error) {
      logger.error("Failed to parse incoming message", { error });
    }
  }
  /**
   * Handle JSON-RPC response
   */
  handleResponse(response) {
    const pending = this.pendingRequests.get(response.id);
    if (pending) {
      clearTimeout(pending.timeoutId);
      this.pendingRequests.delete(response.id);
      pending.resolve(JSON.stringify(response));
    }
  }
  /**
   * Handle notification and dispatch to listeners
   */
  handleNotification(notification) {
    const notificationJson = JSON.stringify(notification);
    for (const callback of this.notificationCallbacks) {
      try {
        callback(notificationJson);
      } catch (error) {
        logger.error("Error in notification callback", { error });
      }
    }
  }
}
class WebSocketTransport {
  constructor(options) {
    this.ws = null;
    this.isConnecting = false;
    this.reconnectAttempts = 0;
    this.reconnectTimeoutId = null;
    this.isDisposed = false;
    this.maxAttemptsReached = false;
    this.pendingRequests = /* @__PURE__ */ new Map();
    this.notificationCallbacks = /* @__PURE__ */ new Set();
    this.url = options.url;
    this.reconnectDelayMs = options.reconnectDelayMs ?? 1e3;
    this.maxReconnectAttempts = options.maxReconnectAttempts ?? 5;
    this.connect();
  }
  /**
   * Send a JSON-RPC request and wait for response
   */
  async send(request) {
    if (!this.isConnected()) {
      throw new Error("WebSocketTransport: Not connected");
    }
    const parsedRequest = JSON.parse(request);
    const id = parsedRequest.id;
    if (id === void 0 || id === null) {
      throw new Error("WebSocketTransport: Request must have an id");
    }
    const responsePromise = new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        this.pendingRequests.delete(id);
        reject(new Error(`Request timeout: ${parsedRequest.method}`));
      }, 5e3);
      this.pendingRequests.set(id, { resolve, reject, timeoutId });
    });
    if (!this.ws) {
      throw new Error("WebSocketTransport: Connection lost");
    }
    this.ws.send(request);
    return responsePromise;
  }
  /**
   * Register a callback for incoming notifications
   */
  onNotification(callback) {
    this.notificationCallbacks.add(callback);
    return () => {
      this.notificationCallbacks.delete(callback);
    };
  }
  /**
   * Check if transport is connected
   */
  isConnected() {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }
  /**
   * Clean up resources and close connection
   */
  dispose() {
    this.isDisposed = true;
    if (this.reconnectTimeoutId) {
      clearTimeout(this.reconnectTimeoutId);
      this.reconnectTimeoutId = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    for (const [id, { reject, timeoutId }] of this.pendingRequests.entries()) {
      clearTimeout(timeoutId);
      reject(new Error("Transport disposed"));
      this.pendingRequests.delete(id);
    }
    this.notificationCallbacks.clear();
  }
  /**
   * Attempt to connect to WebSocket server
   */
  connect() {
    if (this.isDisposed || this.isConnecting || this.isConnected()) {
      return;
    }
    this.isConnecting = true;
    try {
      this.ws = new WebSocket(this.url);
      this.ws.onopen = () => {
        this.isConnecting = false;
        this.reconnectAttempts = 0;
        logger.info("WebSocketTransport connected", { url: this.url });
      };
      this.ws.onmessage = (event) => {
        this.handleIncomingMessage(event.data);
      };
      this.ws.onerror = (error) => {
        logger.error("WebSocketTransport connection error", { error });
      };
      this.ws.onclose = () => {
        this.isConnecting = false;
        this.ws = null;
        if (!this.isDisposed && !this.maxAttemptsReached) {
          this.scheduleReconnect();
        }
      };
    } catch (error) {
      this.isConnecting = false;
      logger.error("WebSocketTransport failed to create WebSocket", { error, url: this.url });
      this.scheduleReconnect();
    }
  }
  /**
   * Schedule reconnection attempt with exponential backoff
   */
  scheduleReconnect() {
    if (this.isDisposed || this.maxAttemptsReached) {
      return;
    }
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      this.maxAttemptsReached = true;
      logger.error("WebSocketTransport max reconnect attempts reached", {
        maxAttempts: this.maxReconnectAttempts
      });
      if (this.ws) {
        this.ws.close();
        this.ws = null;
      }
      return;
    }
    this.reconnectAttempts++;
    const delay = this.reconnectDelayMs * Math.pow(2, this.reconnectAttempts - 1);
    logger.debug("WebSocketTransport reconnecting", {
      delayMs: delay,
      attempt: this.reconnectAttempts,
      maxAttempts: this.maxReconnectAttempts
    });
    this.reconnectTimeoutId = setTimeout(() => {
      this.reconnectTimeoutId = null;
      this.connect();
    }, delay);
  }
  /**
   * Handle incoming message (response or notification)
   */
  handleIncomingMessage(message) {
    try {
      const parsed = JSON.parse(message);
      if (isIpcResponse(parsed)) {
        this.handleResponse(parsed);
      } else if (isIpcNotification(parsed)) {
        this.handleNotification(parsed);
      }
    } catch (error) {
      logger.error("WebSocketTransport failed to parse incoming message", { error, message });
    }
  }
  /**
   * Handle JSON-RPC response
   */
  handleResponse(response) {
    const pending = this.pendingRequests.get(response.id);
    if (pending) {
      clearTimeout(pending.timeoutId);
      this.pendingRequests.delete(response.id);
      pending.resolve(JSON.stringify(response));
    }
  }
  /**
   * Handle notification and dispatch to listeners
   */
  handleNotification(notification) {
    const notificationJson = JSON.stringify(notification);
    for (const callback of this.notificationCallbacks) {
      try {
        callback(notificationJson);
      } catch (error) {
        logger.error("WebSocketTransport notification callback error", {
          error,
          method: notification.method
        });
      }
    }
  }
}
const IS_WEBVIEW = isWebViewEnvironment();
let transportInstance = null;
function getTransport() {
  if (transportInstance) {
    return transportInstance;
  }
  if (IS_WEBVIEW) {
    transportInstance = new NativeTransport();
  } else {
    const wsUrl = "ws://127.0.0.1:9000";
    transportInstance = new WebSocketTransport({ url: wsUrl });
  }
  return transportInstance;
}
const _IpcBridge = class _IpcBridge {
  // Log warning max once per 5s
  constructor() {
    this.nextId = 1;
    this.eventListeners = /* @__PURE__ */ new Map();
    this.transport = null;
    this.isInitialized = false;
    this.lastDisconnectWarning = 0;
    this.DISCONNECT_WARNING_INTERVAL_MS = 5e3;
  }
  /**
   * Initialize the IPC bridge (lazy)
   */
  initialize() {
    if (this.isInitialized) {
      return;
    }
    this.transport = getTransport();
    this.transport.onNotification((notificationJson) => {
      try {
        const parsed = JSON.parse(notificationJson);
        if (isIpcNotification(parsed)) {
          this.handleNotification(parsed);
        }
      } catch (error) {
        logger.error("Failed to parse notification", { error });
      }
    });
    this.isInitialized = true;
  }
  /**
   * Get singleton instance
   */
  static getInstance() {
    _IpcBridge.instance ?? (_IpcBridge.instance = new _IpcBridge());
    return _IpcBridge.instance;
  }
  /**
   * Check if the bridge is connected
   */
  isConnected() {
    var _a;
    this.initialize();
    return ((_a = this.transport) == null ? void 0 : _a.isConnected()) ?? false;
  }
  /**
   * Invoke a method and wait for response
   */
  async invoke(method, params) {
    var _a;
    this.initialize();
    if (!((_a = this.transport) == null ? void 0 : _a.isConnected())) {
      const now = Date.now();
      if (now - this.lastDisconnectWarning > this.DISCONNECT_WARNING_INTERVAL_MS) {
        logger.warn("Transport not connected, call will fail. Waiting for reconnection...");
        this.lastDisconnectWarning = now;
      }
      throw new Error("IpcBridge: Transport not connected");
    }
    const id = this.nextId++;
    const request = {
      jsonrpc: "2.0",
      id,
      method,
      params
    };
    const requestJson = JSON.stringify(request);
    const responseJson = await this.transport.send(requestJson);
    const response = JSON.parse(responseJson);
    if (response.error) {
      throw new Error(`IPC Error ${response.error.code}: ${response.error.message}`);
    }
    return response.result;
  }
  /**
   * Subscribe to notification events
   */
  on(event, callback) {
    this.initialize();
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, /* @__PURE__ */ new Set());
    }
    const listeners = this.eventListeners.get(event);
    if (!listeners) {
      throw new Error(`Event listener set not found for event: ${event}`);
    }
    listeners.add(callback);
    return () => {
      listeners.delete(callback);
    };
  }
  /**
   * Handle notification and dispatch to listeners
   */
  handleNotification(notification) {
    const listeners = this.eventListeners.get(notification.method);
    if (!listeners || listeners.size === 0) {
      return;
    }
    for (const listener of listeners) {
      try {
        listener(notification.params);
      } catch (error) {
        logger.error("Error in event listener", { event: notification.method, error });
      }
    }
  }
};
_IpcBridge.instance = null;
let IpcBridge = _IpcBridge;
const _ParameterClient = class _ParameterClient {
  constructor() {
    this.bridge = IpcBridge.getInstance();
  }
  /**
   * Get singleton instance
   */
  static getInstance() {
    _ParameterClient.instance ?? (_ParameterClient.instance = new _ParameterClient());
    return _ParameterClient.instance;
  }
  /**
   * Get a single parameter's current value and metadata
   */
  async getParameter(id) {
    return this.bridge.invoke(METHOD_GET_PARAMETER, { id });
  }
  /**
   * Set a parameter's value
   * @param id Parameter ID
   * @param value Normalized value [0.0, 1.0]
   */
  async setParameter(id, value) {
    await this.bridge.invoke(METHOD_SET_PARAMETER, {
      id,
      value
    });
  }
  /**
   * Get all parameters with their current values and metadata
   */
  async getAllParameters() {
    const result = await this.bridge.invoke(METHOD_GET_ALL_PARAMETERS);
    return result.parameters;
  }
  /**
   * Test connectivity with Rust backend
   * @returns Roundtrip time in milliseconds
   */
  async ping() {
    const start = performance.now();
    await this.bridge.invoke("ping");
    const end = performance.now();
    return end - start;
  }
  /**
   * Subscribe to parameter change notifications
   * @returns Unsubscribe function
   */
  onParameterChanged(callback) {
    return this.bridge.on(NOTIFICATION_PARAMETER_CHANGED, (data) => {
      if (data && typeof data === "object" && "id" in data && "value" in data) {
        callback(data.id, data.value);
      }
    });
  }
};
_ParameterClient.instance = null;
let ParameterClient = _ParameterClient;
function useParameter(id) {
  const [param, setParam] = useState(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState(null);
  useEffect(() => {
    let isMounted = true;
    const client = ParameterClient.getInstance();
    async function loadParameter() {
      try {
        setIsLoading(true);
        setError(null);
        const allParams = await client.getAllParameters();
        const foundParam = allParams.find((p) => p.id === id);
        if (isMounted) {
          if (foundParam) {
            setParam(foundParam);
          } else {
            setError(new Error(`Parameter not found: ${id}`));
          }
        }
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err : new Error(String(err)));
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    }
    loadParameter();
    return () => {
      isMounted = false;
    };
  }, [id]);
  useEffect(() => {
    const client = ParameterClient.getInstance();
    const unsubscribe = client.onParameterChanged((changedId, value) => {
      if (changedId === id) {
        setParam((prev) => prev ? { ...prev, value } : null);
      }
    });
    return unsubscribe;
  }, [id]);
  const setValue = useCallback(
    async (value) => {
      const client = ParameterClient.getInstance();
      try {
        await client.setParameter(id, value);
        setParam((prev) => prev ? { ...prev, value } : null);
      } catch (err) {
        setError(err instanceof Error ? err : new Error(String(err)));
        throw err;
      }
    },
    [id]
  );
  return { param, setValue, isLoading, error };
}
function useAllParameters() {
  const [params, setParams] = useState([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState(null);
  const reload = useCallback(async () => {
    const client = ParameterClient.getInstance();
    try {
      setIsLoading(true);
      setError(null);
      const allParams = await client.getAllParameters();
      setParams(allParams);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setIsLoading(false);
    }
  }, []);
  useEffect(() => {
    reload();
  }, [reload]);
  useEffect(() => {
    const client = ParameterClient.getInstance();
    const handleParamChange = (changedId, value) => {
      setParams((prev) => prev.map((p) => p.id === changedId ? { ...p, value } : p));
    };
    const unsubscribe = client.onParameterChanged(handleParamChange);
    return unsubscribe;
  }, []);
  return { params, isLoading, error, reload };
}
function useParameterGroups(parameters) {
  return useMemo(() => {
    const grouped = /* @__PURE__ */ new Map();
    for (const param of parameters) {
      const groupName = param.group ?? "Parameters";
      const existing = grouped.get(groupName) ?? [];
      existing.push(param);
      grouped.set(groupName, existing);
    }
    const groups = Array.from(grouped.entries()).map(([name, params]) => ({ name, parameters: params })).sort((a, b) => {
      if (a.name === "Parameters") return -1;
      if (b.name === "Parameters") return 1;
      return a.name.localeCompare(b.name);
    });
    return groups;
  }, [parameters]);
}
function useConnectionStatus() {
  const [status, setStatus] = useState(() => {
    const bridge = IpcBridge.getInstance();
    const connected = bridge.isConnected();
    let transport;
    if (isWebViewEnvironment()) {
      transport = "native";
    } else if (connected) {
      transport = "websocket";
    } else {
      transport = "none";
    }
    return { connected, transport };
  });
  useEffect(() => {
    const bridge = IpcBridge.getInstance();
    const intervalId = setInterval(() => {
      const connected = bridge.isConnected();
      let transport;
      if (isWebViewEnvironment()) {
        transport = "native";
      } else if (connected) {
        transport = "websocket";
      } else {
        transport = "none";
      }
      setStatus((prevStatus) => {
        if (prevStatus.connected !== connected || prevStatus.transport !== transport) {
          return { connected, transport };
        }
        return prevStatus;
      });
    }, 1e3);
    return () => {
      clearInterval(intervalId);
    };
  }, []);
  return status;
}
function useLatencyMonitor(intervalMs = 1e3) {
  const [latency, setLatency] = useState(null);
  const [measurements, setMeasurements] = useState([]);
  const bridge = IpcBridge.getInstance();
  useEffect(() => {
    let isMounted = true;
    const client = ParameterClient.getInstance();
    async function measure() {
      if (!bridge.isConnected()) {
        return;
      }
      try {
        const ms = await client.ping();
        if (isMounted) {
          setLatency(ms);
          setMeasurements((prev) => [...prev.slice(-99), ms]);
        }
      } catch (err) {
        logger.debug("Ping failed", { error: err });
      }
    }
    measure();
    const intervalId = setInterval(measure, intervalMs);
    return () => {
      isMounted = false;
      clearInterval(intervalId);
    };
  }, [intervalMs, bridge]);
  const avg = measurements.length > 0 ? measurements.reduce((sum, val) => sum + val, 0) / measurements.length : 0;
  const max = measurements.length > 0 ? Math.max(...measurements) : 0;
  return {
    latency,
    avg,
    max,
    count: measurements.length
  };
}
function useMeterFrame(intervalMs = 50) {
  const [frame, setFrame] = useState(null);
  useEffect(() => {
    let isMounted = true;
    const bridge = IpcBridge.getInstance();
    async function fetchFrame() {
      if (!bridge.isConnected()) return;
      try {
        const result = await bridge.invoke("getMeterFrame");
        if (isMounted && result.frame) {
          setFrame(result.frame);
        }
      } catch {
      }
    }
    fetchFrame();
    const intervalId = setInterval(fetchFrame, intervalMs);
    return () => {
      isMounted = false;
      clearInterval(intervalId);
    };
  }, [intervalMs]);
  return frame;
}
async function requestResize(width, height) {
  const bridge = IpcBridge.getInstance();
  const result = await bridge.invoke("requestResize", { width, height });
  return result.accepted;
}
function useWindowResizeSync() {
  useEffect(() => {
    const handleResize = () => {
      const width = window.innerWidth;
      const height = window.innerHeight;
      requestResize(width, height).catch((err) => {
        logger.error("Failed to notify host of resize", { error: err, width, height });
      });
    };
    window.addEventListener("resize", handleResize);
    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, []);
}
function useRequestResize() {
  return requestResize;
}
async function getMeterFrame() {
  const bridge = IpcBridge.getInstance();
  const result = await bridge.invoke("getMeterFrame");
  return result.frame;
}
export {
  ERROR_INTERNAL,
  ERROR_INVALID_PARAMS,
  ERROR_INVALID_REQUEST,
  ERROR_METHOD_NOT_FOUND,
  ERROR_PARAM_NOT_FOUND,
  ERROR_PARAM_OUT_OF_RANGE,
  ERROR_PARSE,
  IpcBridge,
  LogLevel,
  Logger,
  METHOD_GET_ALL_PARAMETERS,
  METHOD_GET_PARAMETER,
  METHOD_SET_PARAMETER,
  NOTIFICATION_PARAMETER_CHANGED,
  NativeTransport,
  ParameterClient,
  WebSocketTransport,
  dbToLinear,
  getMeterFrame,
  isBrowserEnvironment,
  isWebViewEnvironment,
  linearToDb,
  logger,
  requestResize,
  useAllParameters,
  useConnectionStatus,
  useLatencyMonitor,
  useMeterFrame,
  useParameter,
  useParameterGroups,
  useRequestResize,
  useWindowResizeSync
};
//# sourceMappingURL=index.js.map
