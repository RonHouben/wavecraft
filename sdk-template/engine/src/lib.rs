use wavecraft::prelude::wavecraft_plugin;
use wavecraft::prelude::wavecraft_processor;
use wavecraft::prelude::SignalChain;

// Custom processors live in the `processors/` folder.
// See `processors/oscillator.rs` for a complete example.
mod processors;
use processors::Oscillator;

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
    signal: SignalChain![Oscillator, InputGain, OutputGain],
}
