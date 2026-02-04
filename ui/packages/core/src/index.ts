/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */

// =============================================================================
// Environment Detection
// =============================================================================
export { isWebViewEnvironment, isBrowserEnvironment } from './utils';

// =============================================================================
// Types
// =============================================================================
export type {
  // IPC types
  IpcRequest,
  IpcResponse,
  IpcNotification,
  IpcError,
  RequestId,
  // Parameter types
  ParameterInfo,
  ParameterType,
  GetParameterParams,
  GetParameterResult,
  SetParameterParams,
  SetParameterResult,
  GetAllParametersResult,
  ParameterChangedNotification,
  // Metering types
  MeterFrame,
  GetMeterFrameResult,
} from './types';

// IPC error codes
export {
  ERROR_PARSE,
  ERROR_INVALID_REQUEST,
  ERROR_METHOD_NOT_FOUND,
  ERROR_INVALID_PARAMS,
  ERROR_INTERNAL,
  ERROR_PARAM_NOT_FOUND,
  ERROR_PARAM_OUT_OF_RANGE,
} from './types';

// IPC method names
export {
  METHOD_GET_PARAMETER,
  METHOD_SET_PARAMETER,
  METHOD_GET_ALL_PARAMETERS,
  NOTIFICATION_PARAMETER_CHANGED,
} from './types';

// =============================================================================
// Core Classes (advanced use)
// =============================================================================
export { IpcBridge, ParameterClient } from './ipc';

// =============================================================================
// React Hooks (primary API)
// =============================================================================
export {
  // Parameter hooks
  useParameter,
  useAllParameters,
  useParameterGroups,
  // Connection & monitoring
  useConnectionStatus,
  useLatencyMonitor,
  // Metering
  useMeterFrame,
  // Resize
  useRequestResize,
  useWindowResizeSync,
} from './hooks';

export type {
  // Hook result types
  UseParameterResult,
  UseAllParametersResult,
  UseLatencyMonitorResult,
  // Parameter grouping
  ParameterGroup,
  // Connection status
  ConnectionStatus,
  TransportType,
} from './hooks';

// =============================================================================
// Resize API (standalone function)
// =============================================================================
export { requestResize } from './hooks';
export type { RequestResizeParams, RequestResizeResult } from './hooks';

// =============================================================================
// Metering API
// =============================================================================
export { getMeterFrame } from './meter-ipc';
export { linearToDb, dbToLinear } from './utils';

// =============================================================================
// Logger
// =============================================================================
export { logger, Logger, LogLevel } from './logger/Logger';
export type { LogContext } from './logger/Logger';

// =============================================================================
// Transports (advanced use)
// =============================================================================
export type { Transport, NotificationCallback } from './transports';
export { WebSocketTransport, NativeTransport } from './transports';
