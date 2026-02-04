/**
 * Parameter Types
 *
 * Types related to plugin parameters.
 */

export type ParameterType = 'float' | 'bool' | 'enum';

export interface ParameterInfo {
  id: string;
  name: string;
  type: ParameterType;
  value: number;
  default: number;
  unit?: string;
  group?: string;
}

// getParameter
export interface GetParameterParams {
  id: string;
}

export interface GetParameterResult {
  id: string;
  value: number;
}

// setParameter
export interface SetParameterParams {
  id: string;
  value: number;
}

export type SetParameterResult = Record<string, never>;

// getAllParameters
export interface GetAllParametersResult {
  parameters: ParameterInfo[];
}

// Notification: parameterChanged
export interface ParameterChangedNotification {
  id: string;
  value: number;
}

// Method Names (matching Rust constants)
export const METHOD_GET_PARAMETER = 'getParameter';
export const METHOD_SET_PARAMETER = 'setParameter';
export const METHOD_GET_ALL_PARAMETERS = 'getAllParameters';
export const NOTIFICATION_PARAMETER_CHANGED = 'parameterChanged';
