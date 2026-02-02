/**
 * @vstkit/ipc - IPC library for VstKit WebView ↔ Rust communication
 *
 * Public exports for application code.
 */

// Environment detection
export { isWebViewEnvironment, isBrowserEnvironment } from './environment';

// Types
export type {
  IpcRequest,
  IpcResponse,
  IpcNotification,
  IpcError,
  RequestId,
  ParameterInfo,
  ParameterType,
  GetParameterParams,
  GetParameterResult,
  SetParameterParams,
  SetParameterResult,
  GetAllParametersResult,
  ParameterChangedNotification,
} from './types';

// Constants
export {
  ERROR_PARSE,
  ERROR_INVALID_REQUEST,
  ERROR_METHOD_NOT_FOUND,
  ERROR_INVALID_PARAMS,
  ERROR_INTERNAL,
  ERROR_PARAM_NOT_FOUND,
  ERROR_PARAM_OUT_OF_RANGE,
  METHOD_GET_PARAMETER,
  METHOD_SET_PARAMETER,
  METHOD_GET_ALL_PARAMETERS,
  NOTIFICATION_PARAMETER_CHANGED,
} from './types';

// Core classes (typically not needed by application code)
export { IpcBridge } from './IpcBridge';
export { ParameterClient } from './ParameterClient';

// React hooks (primary API)
export { useParameter, useAllParameters, useLatencyMonitor } from './hooks';
export type { UseParameterResult, UseAllParametersResult, UseLatencyMonitorResult } from './hooks';

// Connection status hook
export { useConnectionStatus } from './useConnectionStatus';
export type { ConnectionStatus, TransportType } from './useConnectionStatus';

// Resize API
export { requestResize, useRequestResize } from './resize';
export type { RequestResizeParams, RequestResizeResult } from './resize';

// Metering API
export { getMeterFrame, linearToDb, dbToLinear } from './meters';
export type { MeterFrame, GetMeterFrameResult } from './meters';

// Transports (advanced use)
export type { Transport, NotificationCallback } from './transports';
export { WebSocketTransport, NativeTransport } from './transports';
