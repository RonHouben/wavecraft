/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */

// =============================================================================
// Environment Detection
// =============================================================================
export { isWebViewEnvironment, isBrowserEnvironment } from './utils/environment';

// =============================================================================
// Types
// =============================================================================
export type {
  // IPC types
  AudioDiagnostic,
  AudioDiagnosticCode,
  AudioRuntimePhase,
  AudioRuntimeStatus,
  GetAudioStatusResult,
  IpcRequest,
  IpcResponse,
  IpcNotification,
  IpcError,
  RequestId,
} from './types/ipc';

export type {
  // Parameter types
  ParameterInfo,
  ParameterValue,
  ParameterId,
  ParameterIdMap,
  ParameterType,
  GetParameterParams,
  GetParameterResult,
  SetParameterParams,
  SetParameterResult,
  GetAllParametersResult,
  ParameterChangedNotification,
} from './types/parameters';

export type {
  // Metering types
  MeterFrame,
  GetMeterFrameResult,
} from './types/metering';

// IPC error codes
export {
  ERROR_PARSE,
  ERROR_INVALID_REQUEST,
  ERROR_METHOD_NOT_FOUND,
  ERROR_INVALID_PARAMS,
  ERROR_INTERNAL,
  ERROR_PARAM_NOT_FOUND,
  ERROR_PARAM_OUT_OF_RANGE,
  METHOD_GET_AUDIO_STATUS,
  NOTIFICATION_AUDIO_STATUS_CHANGED,
  isAudioRuntimeStatus,
  isIpcResponse,
  isIpcNotification,
  isIpcError,
} from './types/ipc';

// IPC method names
export {
  METHOD_GET_PARAMETER,
  METHOD_SET_PARAMETER,
  METHOD_GET_ALL_PARAMETERS,
  NOTIFICATION_PARAMETER_CHANGED,
} from './types/parameters';

// =============================================================================
// Core Classes (advanced use)
// =============================================================================
export { IpcBridge } from './ipc/IpcBridge';
export { ParameterClient } from './ipc/ParameterClient';

// =============================================================================
// React Hooks (primary API)
// =============================================================================
export { useParameter } from './hooks/useParameter';
export type { UseParameterResult } from './hooks/useParameter';

export { useAllParameters } from './hooks/useAllParameters';
export type { UseAllParametersResult } from './hooks/useAllParameters';

export { useParameterGroups } from './hooks/useParameterGroups';
export type { ParameterGroup } from './hooks/useParameterGroups';

export { useConnectionStatus } from './hooks/useConnectionStatus';
export type { ConnectionStatus, TransportType } from './hooks/useConnectionStatus';

export { useLatencyMonitor } from './hooks/useLatencyMonitor';
export type { UseLatencyMonitorResult } from './hooks/useLatencyMonitor';

export { useMeterFrame } from './hooks/useMeterFrame';
export { useAudioStatus } from './hooks/useAudioStatus';
export type { UseAudioStatusResult } from './hooks/useAudioStatus';

export { useRequestResize } from './hooks/useRequestResize';

export { requestResize, useWindowResizeSync } from './hooks/useWindowResizeSync';
export type { RequestResizeParams, RequestResizeResult } from './hooks/useWindowResizeSync';

// =============================================================================
// Metering API
// =============================================================================
export { getMeterFrame } from './meter-ipc';
export { linearToDb, dbToLinear } from './utils/audio-math';

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
