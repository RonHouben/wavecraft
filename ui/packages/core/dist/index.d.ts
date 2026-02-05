export declare interface ConnectionStatus {
    /** Whether transport is connected and ready */
    connected: boolean;
    /** Type of transport being used */
    transport: TransportType;
}

/**
 * Convert decibels to linear amplitude
 * @param db Decibels
 */
export declare function dbToLinear(db: number): number;

export declare const ERROR_INTERNAL = -32603;

export declare const ERROR_INVALID_PARAMS = -32602;

export declare const ERROR_INVALID_REQUEST = -32600;

export declare const ERROR_METHOD_NOT_FOUND = -32601;

export declare const ERROR_PARAM_NOT_FOUND = -32000;

export declare const ERROR_PARAM_OUT_OF_RANGE = -32001;

export declare const ERROR_PARSE = -32700;

/**
 * IpcBridge - Low-level IPC communication layer
 *
 * Provides a Promise-based API for sending requests and receiving responses
 * using pluggable transport implementations (NativeTransport, WebSocketTransport).
 */
declare type EventCallback<T> = (data: T) => void;

export declare interface GetAllParametersResult {
    parameters: ParameterInfo[];
}

/**
 * Get the latest meter frame from the audio engine
 */
export declare function getMeterFrame(): Promise<MeterFrame | null>;

/**
 * Result from getMeterFrame method
 */
export declare interface GetMeterFrameResult {
    frame: MeterFrame | null;
}

export declare interface GetParameterParams {
    id: string;
}

export declare interface GetParameterResult {
    id: string;
    value: number;
}

export declare class IpcBridge {
    private static instance;
    private nextId;
    private readonly eventListeners;
    private transport;
    private isInitialized;
    private lastDisconnectWarning;
    private readonly DISCONNECT_WARNING_INTERVAL_MS;
    private constructor();
    /**
     * Initialize the IPC bridge (lazy)
     */
    private initialize;
    /**
     * Get singleton instance
     */
    static getInstance(): IpcBridge;
    /**
     * Check if the bridge is connected
     */
    isConnected(): boolean;
    /**
     * Invoke a method and wait for response
     */
    invoke<TResult>(method: string, params?: unknown): Promise<TResult>;
    /**
     * Subscribe to notification events
     */
    on<T>(event: string, callback: EventCallback<T>): () => void;
    /**
     * Handle notification and dispatch to listeners
     */
    private handleNotification;
}

export declare interface IpcError {
    code: number;
    message: string;
    data?: unknown;
}

export declare interface IpcNotification {
    jsonrpc: '2.0';
    method: string;
    params?: unknown;
}

export declare interface IpcRequest {
    jsonrpc: '2.0';
    id: RequestId;
    method: string;
    params?: unknown;
}

export declare interface IpcResponse {
    jsonrpc: '2.0';
    id: RequestId;
    result?: unknown;
    error?: IpcError;
}

/**
 * Check if running in a browser environment (development)
 * @returns true if IPC primitives are NOT available
 */
export declare function isBrowserEnvironment(): boolean;

export declare function isIpcError(obj: unknown): obj is IpcError;

export declare function isIpcNotification(obj: unknown): obj is IpcNotification;

export declare function isIpcResponse(obj: unknown): obj is IpcResponse;

/**
 * Environment Detection
 *
 * Determines if the code is running in WKWebView (production)
 * or a browser (development).
 */
/**
 * Check if running in a WKWebView environment (production)
 * @returns true if globalThis.wavecraft IPC primitives are available
 */
export declare function isWebViewEnvironment(): boolean;

/**
 * Audio Math Utilities
 *
 * Pure functions for audio calculations with no side effects.
 */
/**
 * Convert linear amplitude to decibels
 * @param linear Linear amplitude (0.0 to 1.0+)
 * @param floor Minimum dB value to return (default: -60)
 */
export declare function linearToDb(linear: number, floor?: number): number;

export declare interface LogContext {
    [key: string]: unknown;
}

/**
 * Logger class providing structured logging with severity levels.
 *
 * Example usage:
 * ```typescript
 * const logger = new Logger({ minLevel: LogLevel.INFO });
 * logger.info('Parameter updated', { id: 'gain', value: 0.5 });
 * logger.error('IPC failed', { method: 'getParameter', error });
 * ```
 */
export declare class Logger {
    private minLevel;
    constructor(options?: {
        minLevel?: LogLevel;
    });
    /**
     * Set the minimum log level at runtime.
     */
    setMinLevel(level: LogLevel): void;
    /**
     * Log debug message (verbose tracing).
     */
    debug(message: string, context?: LogContext): void;
    /**
     * Log informational message.
     */
    info(message: string, context?: LogContext): void;
    /**
     * Log warning message.
     */
    warn(message: string, context?: LogContext): void;
    /**
     * Log error message.
     */
    error(message: string, context?: LogContext): void;
}

/**
 * Global logger instance for the UI.
 * Configure once at app startup, use throughout the codebase.
 */
export declare const logger: Logger;

/**
 * Logger - Structured logging abstraction for the UI.
 *
 * Wraps browser console API with severity levels and structured context.
 * In production builds, logs can be filtered by level at runtime.
 */
export declare enum LogLevel {
    DEBUG = 0,
    INFO = 1,
    WARN = 2,
    ERROR = 3
}

/**
 * Metering Types
 *
 * Types related to audio metering.
 */
/**
 * Meter frame data (all values in linear scale, not dB)
 */
export declare interface MeterFrame {
    peak_l: number;
    peak_r: number;
    rms_l: number;
    rms_r: number;
    timestamp: number;
}

export declare const METHOD_GET_ALL_PARAMETERS = "getAllParameters";

export declare const METHOD_GET_PARAMETER = "getParameter";

export declare const METHOD_SET_PARAMETER = "setParameter";

/**
 * Native WKWebView transport implementation
 *
 * Uses the __WAVECRAFT_IPC__ primitives injected by the Rust engine.
 */
export declare class NativeTransport implements Transport {
    private readonly pendingRequests;
    private readonly notificationCallbacks;
    private readonly primitives;
    constructor();
    /**
     * Send a JSON-RPC request and wait for response
     */
    send(request: string): Promise<string>;
    /**
     * Register a callback for incoming notifications
     */
    onNotification(callback: NotificationCallback): () => void;
    /**
     * Check if transport is connected (native is always connected)
     */
    isConnected(): boolean;
    /**
     * Clean up resources
     */
    dispose(): void;
    /**
     * Handle incoming message (response or notification)
     */
    private handleIncomingMessage;
    /**
     * Handle JSON-RPC response
     */
    private handleResponse;
    /**
     * Handle notification and dispatch to listeners
     */
    private handleNotification;
}

export declare const NOTIFICATION_PARAMETER_CHANGED = "parameterChanged";

/**
 * Transport interface for IPC communication
 *
 * Provides an abstraction layer for different transport mechanisms
 * (native WKWebView, WebSocket) with consistent request/notification handling.
 */
/**
 * Callback for handling incoming notifications from the engine
 */
export declare type NotificationCallback = (notification: string) => void;

declare type ParameterChangeCallback = (id: string, value: number) => void;

export declare interface ParameterChangedNotification {
    id: string;
    value: number;
}

export declare class ParameterClient {
    private static instance;
    private readonly bridge;
    private constructor();
    /**
     * Get singleton instance
     */
    static getInstance(): ParameterClient;
    /**
     * Get a single parameter's current value and metadata
     */
    getParameter(id: string): Promise<GetParameterResult>;
    /**
     * Set a parameter's value
     * @param id Parameter ID
     * @param value Normalized value [0.0, 1.0]
     */
    setParameter(id: string, value: number): Promise<void>;
    /**
     * Get all parameters with their current values and metadata
     */
    getAllParameters(): Promise<ParameterInfo[]>;
    /**
     * Test connectivity with Rust backend
     * @returns Roundtrip time in milliseconds
     */
    ping(): Promise<number>;
    /**
     * Subscribe to parameter change notifications
     * @returns Unsubscribe function
     */
    onParameterChanged(callback: ParameterChangeCallback): () => void;
}

export declare interface ParameterGroup {
    name: string;
    parameters: ParameterInfo[];
}

export declare interface ParameterInfo {
    id: string;
    name: string;
    type: ParameterType;
    value: number;
    default: number;
    unit?: string;
    group?: string;
}

/**
 * Parameter Types
 *
 * Types related to plugin parameters.
 */
export declare type ParameterType = 'float' | 'bool' | 'enum';

/**
 * IPC Types - TypeScript definitions matching Rust protocol
 *
 * These types must stay in sync with engine/crates/protocol/src/ipc.rs
 */
export declare type RequestId = string | number;

/**
 * Request resize of the editor window
 *
 * @param width - Desired width in logical pixels
 * @param height - Desired height in logical pixels
 * @returns Promise that resolves to true if accepted, false if rejected
 *
 * @example
 * ```ts
 * const accepted = await requestResize(1024, 768);
 * if (accepted) {
 *   console.log('Resize accepted by host');
 * } else {
 *   console.warn('Resize rejected by host');
 * }
 * ```
 */
export declare function requestResize(width: number, height: number): Promise<boolean>;

/**
 * useWindowResizeSync - Automatic window resize sync to host
 */
export declare interface RequestResizeParams {
    width: number;
    height: number;
}

export declare interface RequestResizeResult {
    accepted: boolean;
}

export declare interface SetParameterParams {
    id: string;
    value: number;
}

export declare type SetParameterResult = Record<string, never>;

/**
 * Transport abstraction for IPC communication
 *
 * Implementations handle the low-level details of sending requests,
 * receiving responses, and dispatching notifications.
 */
export declare interface Transport {
    /**
     * Send a JSON-RPC request and wait for the response
     *
     * @param request - JSON-RPC request string
     * @returns Promise resolving to JSON-RPC response string
     * @throws Error if transport is not connected or request fails
     */
    send(request: string): Promise<string>;
    /**
     * Register a callback for incoming notifications from the engine
     *
     * @param callback - Function called when a notification arrives
     * @returns Cleanup function to remove the callback
     */
    onNotification(callback: NotificationCallback): () => void;
    /**
     * Check if the transport is currently connected
     *
     * @returns true if transport can send/receive messages
     */
    isConnected(): boolean;
    /**
     * Clean up resources (close connections, remove listeners)
     *
     * Should be called when the transport is no longer needed.
     */
    dispose(): void;
}

/**
 * useConnectionStatus - Monitor transport connection status
 *
 * Provides real-time connection status updates for the IPC transport.
 * Useful for showing connection indicators in the UI.
 */
export declare type TransportType = 'native' | 'websocket' | 'none';

export declare function useAllParameters(): UseAllParametersResult;

export declare interface UseAllParametersResult {
    params: ParameterInfo[];
    isLoading: boolean;
    error: Error | null;
    reload: () => Promise<void>;
}

/**
 * Hook to monitor IPC connection status
 *
 * Polls the transport every second to detect connection changes.
 * Native transport is always connected, WebSocket may reconnect.
 *
 * @returns Connection status object
 */
export declare function useConnectionStatus(): ConnectionStatus;

export declare function useLatencyMonitor(intervalMs?: number): UseLatencyMonitorResult;

/**
 * useLatencyMonitor - Hook for monitoring IPC latency
 */
export declare interface UseLatencyMonitorResult {
    latency: number | null;
    avg: number;
    max: number;
    count: number;
}

/**
 * Hook to poll meter frames at a specified interval
 *
 * @param intervalMs - Polling interval in milliseconds (default: 50ms = 20fps)
 * @returns Current meter frame or null if not available
 */
export declare function useMeterFrame(intervalMs?: number): MeterFrame | null;

export declare function useParameter(id: string): UseParameterResult;

/**
 * Organize parameters into groups based on their group metadata.
 *
 * @param parameters - Array of all parameters
 * @returns Array of parameter groups, each containing parameters for that group
 *
 * @example
 * ```tsx
 * const { parameters } = useAllParameters();
 * const groups = useParameterGroups(parameters);
 *
 * return (
 *   <>
 *     {groups.map(group => (
 *       <ParameterGroup key={group.name} group={group} />
 *     ))}
 *   </>
 * );
 * ```
 */
export declare function useParameterGroups(parameters: ParameterInfo[]): ParameterGroup[];

export declare interface UseParameterResult {
    param: ParameterInfo | null;
    setValue: (value: number) => Promise<void>;
    isLoading: boolean;
    error: Error | null;
}

/**
 * useRequestResize - Hook for requesting window resize
 */
/**
 * React hook for requesting window resize
 *
 * @returns Function to request resize
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const resize = useRequestResize();
 *
 *   const handleExpand = async () => {
 *     const accepted = await resize(1200, 900);
 *     if (!accepted) {
 *       alert('Host rejected resize request');
 *     }
 *   };
 *
 *   return <button onClick={handleExpand}>Expand</button>;
 * }
 * ```
 */
export declare function useRequestResize(): (width: number, height: number) => Promise<boolean>;

/**
 * Hook that automatically syncs window resize events to the host
 *
 * This hook listens for browser window resize events and notifies the host
 * DAW of size changes. Useful when the user resizes the plugin window via
 * the DAW's window controls or edge dragging.
 *
 * @example
 * ```tsx
 * function App() {
 *   // Automatically sync window size changes to host
 *   useWindowResizeSync();
 *
 *   return <div>Plugin UI</div>;
 * }
 * ```
 */
export declare function useWindowResizeSync(): void;

/**
 * WebSocket transport implementation with automatic reconnection
 *
 * Connects to the standalone dev server for browser-based UI development.
 */
export declare class WebSocketTransport implements Transport {
    private readonly url;
    private readonly reconnectDelayMs;
    private readonly maxReconnectAttempts;
    private ws;
    private isConnecting;
    private reconnectAttempts;
    private reconnectTimeoutId;
    private isDisposed;
    private maxAttemptsReached;
    private readonly pendingRequests;
    private readonly notificationCallbacks;
    constructor(options: WebSocketTransportOptions);
    /**
     * Send a JSON-RPC request and wait for response
     */
    send(request: string): Promise<string>;
    /**
     * Register a callback for incoming notifications
     */
    onNotification(callback: NotificationCallback): () => void;
    /**
     * Check if transport is connected
     */
    isConnected(): boolean;
    /**
     * Clean up resources and close connection
     */
    dispose(): void;
    /**
     * Attempt to connect to WebSocket server
     */
    private connect;
    /**
     * Schedule reconnection attempt with exponential backoff
     */
    private scheduleReconnect;
    /**
     * Handle incoming message (response or notification)
     */
    private handleIncomingMessage;
    /**
     * Handle JSON-RPC response
     */
    private handleResponse;
    /**
     * Handle notification and dispatch to listeners
     */
    private handleNotification;
}

declare interface WebSocketTransportOptions {
    /** WebSocket server URL (e.g., ws://127.0.0.1:9000) */
    url: string;
    /** Reconnection delay in milliseconds (default: 1000) */
    reconnectDelayMs?: number;
    /** Maximum reconnection attempts (default: 5, use Infinity for unlimited) */
    maxReconnectAttempts?: number;
}

export { }
