/**
 * Parameter Types
 *
 * Types related to plugin parameters.
 */

export type ParameterType = 'float' | 'bool' | 'enum';

/**
 * Augmentable parameter ID registry.
 *
 * The generated `ui/src/generated/parameters.ts` file augments this interface
 * with the plugin's concrete parameter IDs.
 */
// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface ParameterIdMap {}

/**
 * Internal marker key added by generated module augmentation.
 *
 * Used to distinguish:
 * - unaugmented projects (fallback `string` for backward compatibility)
 * - augmented projects with zero parameters (resolves to `never`)
 */
export type ParameterIdMapAugmentedMarker = '__wavecraft_internal_augmented__';

/**
 * Type-safe parameter identifier.
 *
 * When generated augmentation is present, this resolves to a literal string
 * union of plugin parameter IDs. If augmentation is present but no parameters
 * exist, this resolves to `never`. Without augmentation, it falls back to
 * `string` for backward compatibility.
 */
export type ParameterId = ParameterIdMapAugmentedMarker extends keyof ParameterIdMap
  ? Exclude<Extract<keyof ParameterIdMap, string>, ParameterIdMapAugmentedMarker>
  : string;

export interface ParameterInfo {
  id: ParameterId;
  name: string;
  type: ParameterType;
  value: number;
  default: number;
  unit?: string;
  group?: string;
}

// getParameter
export interface GetParameterParams {
  id: ParameterId;
}

export interface GetParameterResult {
  id: ParameterId;
  value: number;
}

// setParameter
export interface SetParameterParams {
  id: ParameterId;
  value: number;
}

export type SetParameterResult = Record<string, never>;

// getAllParameters
export interface GetAllParametersResult {
  parameters: ParameterInfo[];
}

// Notification: parameterChanged
export interface ParameterChangedNotification {
  id: ParameterId;
  value: number;
}

// Method Names (matching Rust constants)
export const METHOD_GET_PARAMETER = 'getParameter';
export const METHOD_SET_PARAMETER = 'setParameter';
export const METHOD_GET_ALL_PARAMETERS = 'getAllParameters';
export const NOTIFICATION_PARAMETER_CHANGED = 'parameterChanged';
