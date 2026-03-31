import type { SignalChainOrder, SlotType } from '@wavecraft/core';

export function mergeVisibleReorderedSlots(
  currentOrder: SignalChainOrder[],
  reorderedVisibleIds: string[],
  idToType: Map<string, SlotType>
): SignalChainOrder[] {
  const reorderedVisibleSlots: SignalChainOrder[] = reorderedVisibleIds
    .map((id) => {
      const type = idToType.get(id);
      return type ? { id, type } : null;
    })
    .filter((slot): slot is SignalChainOrder => slot !== null);

  if (currentOrder.length === 0) {
    return reorderedVisibleSlots;
  }

  const visibleIds = new Set(reorderedVisibleIds);
  let nextVisibleSlotIndex = 0;

  return currentOrder.map((slot) => {
    if (!visibleIds.has(slot.id)) {
      return slot;
    }

    const nextVisibleSlot = reorderedVisibleSlots[nextVisibleSlotIndex];
    nextVisibleSlotIndex += 1;
    return nextVisibleSlot ?? slot;
  });
}
