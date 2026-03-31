/**
 * Signal chain slot types
 *
 * These types mirror the Rust SignalChainSlot contract and represent the
 * unified runtime ordering for processors and taps.
 */

export type SlotType = 'processor' | 'tap';

export type ProcessorId = string;

export type AudioSignalTapId = string;

/**
 * A single slot in the runtime signal chain order.
 *
 * The full runtime order is represented as `SignalChainOrder[]`.
 */
export type SignalChainOrder = {
  id: ProcessorId | AudioSignalTapId;
  type: SlotType;
};
