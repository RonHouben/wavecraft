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

export interface ProcessorIdMap extends WavecraftProcessorIdMap {}

/**
 * Internal marker key added by generated module augmentation.
 *
 * Used to distinguish:
 * - unaugmented projects (fallback `string` for backward compatibility)
 * - augmented projects with zero processors (resolves to `never`)
 */
export type ProcessorIdMapAugmentedMarker = '__wavecraft_internal_processors_augmented__';

/**
 * Type-safe processor identifier.
 *
 * When generated augmentation is present, this resolves to a literal string
 * union of discovered processor IDs. If augmentation is present but no
 * processors exist, this resolves to `never`. Without augmentation, it falls
 * back to `string` for backward compatibility.
 */
export type ProcessorId = ProcessorIdMapAugmentedMarker extends keyof ProcessorIdMap
  ? Exclude<Extract<keyof ProcessorIdMap, string>, ProcessorIdMapAugmentedMarker>
  : string;
