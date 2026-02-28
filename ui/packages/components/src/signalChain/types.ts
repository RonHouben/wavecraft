/**
 * Shared types for the SignalChain component system.
 */

/**
 * An entry in the SignalChain processor list.
 *
 * `id` is the processor's registered ID string (e.g. "input_trim").
 * `component` is the React node to render for that processor.
 */
export interface SignalChainProcessorEntry {
  id: string;
  component: React.ReactNode;
}
