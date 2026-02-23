import { describe, expect, it } from 'vitest';

import { Processor, SineWaveIcon, getWaveformIcon } from './index';

describe('components barrel', () => {
  it('exports canonical Processor component', () => {
    expect(Processor).toBeTypeOf('function');
  });

  it('exports waveform icon helpers', () => {
    expect(SineWaveIcon).toBeTypeOf('function');
    expect(getWaveformIcon('Sine')).not.toBeNull();
  });
});
