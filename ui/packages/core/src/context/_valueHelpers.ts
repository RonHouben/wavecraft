import type { ParameterId, ParameterInfo, ParameterValue } from '../types/parameters';

// private — do not export from index.ts
export function toError(err: unknown): Error {
  return err instanceof Error ? err : new Error(String(err));
}

// private — do not export from index.ts
export function normalizeValue(
  paramType: ParameterInfo['type'],
  value: ParameterValue
): ParameterValue {
  if (paramType === 'bool') {
    return typeof value === 'boolean' ? value : value >= 0.5;
  }

  return typeof value === 'boolean' ? (value ? 1 : 0) : value;
}

// private — do not export from index.ts
export function normalizeParameter(param: ParameterInfo): ParameterInfo {
  return {
    ...param,
    value: normalizeValue(param.type, param.value),
    default: normalizeValue(param.type, param.default),
  };
}

// private — do not export from index.ts
export function updateParameterValue(
  params: ParameterInfo[],
  changedId: ParameterId,
  value: ParameterValue
): ParameterInfo[] {
  let changed = false;

  const next = params.map((param) => {
    if (param.id !== changedId) {
      return param;
    }

    const normalizedValue = normalizeValue(param.type, value);
    if (param.value === normalizedValue) {
      return param;
    }

    changed = true;
    return {
      ...param,
      value: normalizedValue,
    };
  });

  return changed ? next : params;
}

// private — do not export from index.ts
export function rollbackParameterValueIfCurrentMatches(
  params: ParameterInfo[],
  changedId: ParameterId,
  expectedCurrentValue: ParameterValue,
  rollbackValue: ParameterValue
): ParameterInfo[] {
  let changed = false;

  const next = params.map((param) => {
    if (param.id !== changedId) {
      return param;
    }

    const normalizedExpected = normalizeValue(param.type, expectedCurrentValue);
    if (param.value !== normalizedExpected) {
      return param;
    }

    const normalizedRollback = normalizeValue(param.type, rollbackValue);
    if (param.value === normalizedRollback) {
      return param;
    }

    changed = true;
    return {
      ...param,
      value: normalizedRollback,
    };
  });

  return changed ? next : params;
}
