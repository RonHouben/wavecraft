use wavecraft::prelude::wavecraft_plugin;
use wavecraft::prelude::wavecraft_processor;
use wavecraft::prelude::PassthroughDsp;
use wavecraft::OscilloscopeTap;
use wavecraft::TestToneProcessor;

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
wavecraft_processor!(TestTone => TestToneProcessor);
wavecraft_processor!(Passthrough => PassthroughDsp);
wavecraft_processor!(ToneFilter => Filter);
wavecraft_processor!(SoftClip => Saturator);
wavecraft_processor!(OutputGain => Gain);

// ---------------------------------------------------------------------------
// Plugin definition
// ---------------------------------------------------------------------------
// `processors: [...]` defines the signal chain — processors run in order.
// `taps: [...]` declares observer taps that can be placed anywhere in the chain.
wavecraft_plugin! {
    name: "My First Plugin",
    processors: [
        TestTone,
        InputTrim,
        Passthrough,
        ExampleProcessor,
        ToneFilter,
        SoftClip,
        OutputGain,
    ],
    taps: [OscilloscopeTap],
}
