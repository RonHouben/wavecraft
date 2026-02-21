import type { ParameterId } from '../types/parameters';
import type { ProcessorId } from '../types/processors';

export const PROCESSOR_BYPASS_SUFFIX = '_bypass';

export function isBypassParameterId(id: string): boolean {
  return id.endsWith(PROCESSOR_BYPASS_SUFFIX);
}

export function getProcessorBypassParamId(processorId: ProcessorId): ParameterId {
  return `${processorId}${PROCESSOR_BYPASS_SUFFIX}` as ParameterId;
}
