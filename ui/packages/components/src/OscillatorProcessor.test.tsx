import { describe, expect, it } from 'vitest';

import { Processor } from './index';

describe('components barrel', () => {
  it('exports canonical Processor component', () => {
    expect(Processor).toBeTypeOf('function');
  });
});
