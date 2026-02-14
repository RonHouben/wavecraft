// Import everything from Wavecraft SDK
use wavecraft::prelude::*;

// Custom processors live in the `processors/` folder.
// See `processors/oscillator.rs` for a complete example.
mod processors;
#[allow(unused_imports)] // Oscillator is unused in default signal chain
use processors::Oscillator;

// ---------------------------------------------------------------------------
// Processor wrappers
// ---------------------------------------------------------------------------
// `wavecraft_processor!` creates a named wrapper around a built-in processor.
// The wrapper name becomes the parameter-ID prefix (e.g. "inputgain_gain").
//
// Custom processors (like Oscillator) are used directly — they already
// implement the Processor trait with their own parameter types.

wavecraft_processor!(InputGain => Gain);
wavecraft_processor!(OutputGain => Gain);

// ---------------------------------------------------------------------------
// Plugin definition
// ---------------------------------------------------------------------------
// `SignalChain![]` processes audio through each processor in order.
// Vendor/URL metadata comes from Cargo.toml.
wavecraft_plugin! {
    name: "My First Plugin",
    signal: SignalChain![Oscillator, InputGain, OutputGain],
}
