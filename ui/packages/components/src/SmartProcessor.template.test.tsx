import { describe, expect, it } from 'vitest';

describe('sdk-template SmartProcessor migration', () => {
  it('documents that SmartProcessor module was removed from sdk-template', async () => {
    const removedModulePath =
      '../../../../sdk-template/ui/src/processors/' + String('SmartProcessor');
    await expect(import(removedModulePath)).rejects.toThrow(
      /Failed to resolve import|Cannot find module/
    );
  });
});
