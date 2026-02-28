/**
 * Shared types for the SignalChain component system.
 */

import type { SlotType } from '@wavecraft/core';

/**
 * An entry in the unified signal chain (processor or tap).
 *
 * `id` is the slot's registered ID string (e.g. "soft_clip" or "oscilloscope_tap").
 * `type` distinguishes between processor and tap slots.
 * `component` is the React node to render for that slot.
 */
export interface SignalChainEntry {
  id: string;
  type: SlotType;
  component: React.ReactNode;
}
