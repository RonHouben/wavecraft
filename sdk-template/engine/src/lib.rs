use wavecraft::prelude::wavecraft_plugin;
use wavecraft::prelude::wavecraft_processor;
use wavecraft::prelude::SignalChain;
use wavecraft::Oscillator;

// Custom processors live in the `processors/` folder.
// See `processors/example_processor.rs` for a minimal custom processor.
mod processors;
use processors::ExampleProcessor;

// ---------------------------------------------------------------------------
// Processor wrappers
// ---------------------------------------------------------------------------
// `wavecraft_processor!` creates a named wrapper around a built-in processor.
// The wrapper name becomes the parameter-ID prefix (e.g. "inputgain_gain").
wavecraft_processor!(InputGain => Gain);
wavecraft_processor!(OutputGain => Gain);

// ---------------------------------------------------------------------------
// Plugin definition
// ---------------------------------------------------------------------------
// `SignalChain![]` processes audio through each processor in order.
wavecraft_plugin! {
    name: "My First Plugin",
    signal: SignalChain![Oscillator, ExampleProcessor, InputGain, OutputGain],
}
