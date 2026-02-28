import { describe, expect, it } from 'vitest';
import { getControlStateClass } from './controlStates';

describe('getControlStateClass', () => {
  it('maps hover state to hover visual class', () => {
    const className = getControlStateClass({ state: 'hover' });

    expect(className).toContain('brightness-110');
    expect(className).toContain('hover:brightness-110');
  });

  it('maps focus state to focus ring classes', () => {
    const className = getControlStateClass({ state: 'focus' });

    expect(className).toContain('ring-2');
    expect(className).toContain('ring-accent-light');
    expect(className).toContain('ring-offset-2');
    expect(className).toContain('ring-offset-plugin-dark');
  });

  it('maps active state to active scale/brightness classes', () => {
    const className = getControlStateClass({ state: 'active' });

    expect(className).toContain('scale-[0.98]');
    expect(className).toContain('brightness-95');
    expect(className).toContain('active:scale-[0.98]');
  });

  it('returns disabled classes and suppresses interactive classes when disabled', () => {
    const className = getControlStateClass({ state: 'hover', disabled: true });

    expect(className).toContain('cursor-not-allowed');
    expect(className).toContain('opacity-50');
    expect(className).not.toContain('hover:brightness-110');
  });
});
