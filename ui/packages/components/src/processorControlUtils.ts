import type { ParameterInfo } from '@wavecraft/core';

const PROCESSOR_BYPASS_SUFFIX = '_bypass';

function toTitleCaseWords(value: string): string {
  return value
    .split('_')
    .map((word) => (word.length > 0 ? `${word.charAt(0).toUpperCase()}${word.slice(1)}` : word))
    .join(' ');
}

export function processorGroupName(processorId: string, fallbackName?: string): string {
  if (fallbackName) {
    return fallbackName;
  }

  return toTitleCaseWords(processorId);
}

export function parametersForProcessor(
  parameters: readonly ParameterInfo[],
  processorId: string
): ParameterInfo[] {
  const bypassId = `${processorId}${PROCESSOR_BYPASS_SUFFIX}`;

  return parameters.filter(
    (param) => param.id === bypassId || param.id.startsWith(`${processorId}_`)
  );
}
