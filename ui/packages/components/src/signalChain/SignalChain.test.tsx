import { describe, expect, it } from 'vitest';

import type { SignalChainOrder, SlotType } from '@wavecraft/core';

import { mergeVisibleReorderedSlots } from './mergeVisibleReorderedSlots';

function createIdToTypeMap(order: SignalChainOrder[]): Map<string, SlotType> {
  return new Map(order.map((slot) => [slot.id, slot.type]));
}

describe('mergeVisibleReorderedSlots', () => {
  it('preserves hidden backend-only slots while reordering visible cards', () => {
    const currentOrder: SignalChainOrder[] = [
      { id: 'TestTone', type: 'processor' },
      { id: 'InputTrim', type: 'processor' },
      { id: 'Passthrough', type: 'processor' },
      { id: 'ExampleProcessor', type: 'processor' },
      { id: 'ToneFilter', type: 'processor' },
      { id: 'SoftClip', type: 'processor' },
      { id: 'OutputGain', type: 'processor' },
      { id: 'OscilloscopeTap', type: 'tap' },
    ];

    const reorderedVisibleIds = [
      'InputTrim',
      'TestTone',
      'Passthrough',
      'ToneFilter',
      'SoftClip',
      'OutputGain',
      'OscilloscopeTap',
    ];

    const result = mergeVisibleReorderedSlots(
      currentOrder,
      reorderedVisibleIds,
      createIdToTypeMap(currentOrder)
    );

    expect(result).toEqual([
      { id: 'InputTrim', type: 'processor' },
      { id: 'TestTone', type: 'processor' },
      { id: 'Passthrough', type: 'processor' },
      { id: 'ExampleProcessor', type: 'processor' },
      { id: 'ToneFilter', type: 'processor' },
      { id: 'SoftClip', type: 'processor' },
      { id: 'OutputGain', type: 'processor' },
      { id: 'OscilloscopeTap', type: 'tap' },
    ]);
  });

  it('falls back to visible slots when the backend order has not loaded yet', () => {
    const result = mergeVisibleReorderedSlots(
      [],
      ['TestTone', 'InputTrim'],
      new Map([
        ['TestTone', 'processor' satisfies SlotType],
        ['InputTrim', 'processor' satisfies SlotType],
      ])
    );

    expect(result).toEqual([
      { id: 'TestTone', type: 'processor' },
      { id: 'InputTrim', type: 'processor' },
    ]);
  });
});
