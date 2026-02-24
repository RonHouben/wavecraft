/**
 * Processor Types
 *
 * Types related to discovered processor IDs from the plugin signal chain.
 */

/**
 * Augmentable processor ID registry.
 *
 * The generated `ui/src/generated/processors.ts` file augments the global
 * `WavecraftProcessorIdMap` interface with the plugin's concrete processor IDs.
 */
declare global {
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  interface WavecraftProcessorIdMap {}
}

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface ProcessorIdMap extends WavecraftProcessorIdMap {}

/**
 * Internal marker key added by generated module augmentation.
 *
 * Used to distinguish generated entries from concrete processor IDs.
 */
export type ProcessorIdMapAugmentedMarker = '__wavecraft_internal_processors_augmented__';

/**
 * Type-safe processor identifier.
 *
 * Resolves to a literal string union of discovered processor IDs derived from
 * the generated augmentation map. When no IDs are generated, this resolves to
 * `never`.
 */
export type ProcessorId = Exclude<
  Extract<keyof ProcessorIdMap, string>,
  ProcessorIdMapAugmentedMarker
>;
