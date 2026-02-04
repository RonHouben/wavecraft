/**
 * Types - All TypeScript type definitions
 *
 * Re-exports all types from domain-specific files.
 */

// IPC types
export type {
  RequestId,
  IpcRequest,
  IpcResponse,
  IpcNotification,
  IpcError,
  WavecraftIpcPrimitives,
} from './ipc';

export {
  ERROR_PARSE,
  ERROR_INVALID_REQUEST,
  ERROR_METHOD_NOT_FOUND,
  ERROR_INVALID_PARAMS,
  ERROR_INTERNAL,
  ERROR_PARAM_NOT_FOUND,
  ERROR_PARAM_OUT_OF_RANGE,
  isIpcResponse,
  isIpcNotification,
  isIpcError,
} from './ipc';

// Parameter types
export type {
  ParameterType,
  ParameterInfo,
  GetParameterParams,
  GetParameterResult,
  SetParameterParams,
  SetParameterResult,
  GetAllParametersResult,
  ParameterChangedNotification,
} from './parameters';

export {
  METHOD_GET_PARAMETER,
  METHOD_SET_PARAMETER,
  METHOD_GET_ALL_PARAMETERS,
  NOTIFICATION_PARAMETER_CHANGED,
} from './parameters';

// Metering types
export type { MeterFrame, GetMeterFrameResult } from './metering';
