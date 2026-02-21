use wavecraft::prelude::wavecraft_plugin;
use wavecraft::prelude::wavecraft_processor;
use wavecraft::prelude::SignalChain;
use wavecraft::Oscillator;
use wavecraft::OscilloscopeTap;

// Custom processors live in the `processors/` folder.
// See `processors/example_processor.rs` for a minimal custom processor.
mod processors;
use processors::ExampleProcessor;

// ---------------------------------------------------------------------------
// Processor wrappers
// ---------------------------------------------------------------------------
// `wavecraft_processor!` creates a named wrapper around a built-in processor.
// Wrapper names are converted to snake_case and prefixed into parameter IDs
// (e.g. `OutputGain` contributes the `output_gain_*` prefix).
wavecraft_processor!(InputTrim => Gain);
wavecraft_processor!(ToneFilter => Filter);
wavecraft_processor!(SoftClip => Saturator);
wavecraft_processor!(OutputGain => Gain);

// ---------------------------------------------------------------------------
// Plugin definition
// ---------------------------------------------------------------------------
// `SignalChain![]` processes audio through each processor in order.
wavecraft_plugin! {
    name: "My First Plugin",
    signal: SignalChain![
        InputTrim,
        // Generator semantics: additive when enabled, passthrough no-op when disabled.
        Oscillator,
        ExampleProcessor,
        ToneFilter,
        SoftClip,
        OscilloscopeTap,
        OutputGain
    ],
}
