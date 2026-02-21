import { describe, expect, it } from 'vitest';

import { getProcessorBypassParamId, isBypassParameterId, PROCESSOR_BYPASS_SUFFIX } from './bypass';

describe('processor bypass helpers', () => {
  it('builds processor bypass parameter id using suffix', () => {
    expect(getProcessorBypassParamId('input_trim')).toBe(`input_trim${PROCESSOR_BYPASS_SUFFIX}`);
  });

  it('detects bypass parameter ids', () => {
    expect(isBypassParameterId('input_trim_bypass')).toBe(true);
    expect(isBypassParameterId('input_trim_gain')).toBe(false);
  });
});
