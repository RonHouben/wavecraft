/**
 * IPC Types - TypeScript definitions matching Rust protocol
 *
 * These types must stay in sync with engine/crates/protocol/src/ipc.rs
 */

// ============================================================================
// JSON-RPC 2.0 Message Types
// ============================================================================

export type RequestId = string | number;

export interface IpcRequest {
  jsonrpc: '2.0';
  id: RequestId;
  method: string;
  params?: unknown;
}

export interface IpcResponse {
  jsonrpc: '2.0';
  id: RequestId;
  result?: unknown;
  error?: IpcError;
}

export interface IpcNotification {
  jsonrpc: '2.0';
  method: string;
  params?: unknown;
}

export interface IpcError {
  code: number;
  message: string;
  data?: unknown;
}

// ============================================================================
// Error Codes (matching Rust constants)
// ============================================================================

export const ERROR_PARSE = -32700;
export const ERROR_INVALID_REQUEST = -32600;
export const ERROR_METHOD_NOT_FOUND = -32601;
export const ERROR_INVALID_PARAMS = -32602;
export const ERROR_INTERNAL = -32603;
export const ERROR_PARAM_NOT_FOUND = -32000;
export const ERROR_PARAM_OUT_OF_RANGE = -32001;

// ============================================================================
// Injected IPC Primitives (from Rust)
// ============================================================================

export interface WavecraftIpcPrimitives {
  postMessage: (message: string) => void;
  setReceiveCallback: (callback: (message: string) => void) => void;
  onParamUpdate?: (listener: (notification: unknown) => void) => () => void;
  _receive: (message: string) => void; // Internal, called by Rust
  _onParamUpdate?: (message: unknown) => void; // Internal, called by Rust
}

declare global {
  var __WAVECRAFT_IPC__: WavecraftIpcPrimitives | undefined;
}

// ============================================================================
// Type Guards
// ============================================================================

export function isIpcResponse(obj: unknown): obj is IpcResponse {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    'jsonrpc' in obj &&
    'id' in obj &&
    ('result' in obj || 'error' in obj)
  );
}

export function isIpcNotification(obj: unknown): obj is IpcNotification {
  return (
    typeof obj === 'object' && obj !== null && 'jsonrpc' in obj && 'method' in obj && !('id' in obj)
  );
}

export function isIpcError(obj: unknown): obj is IpcError {
  return typeof obj === 'object' && obj !== null && 'code' in obj && 'message' in obj;
}
