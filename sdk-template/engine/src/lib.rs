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
// Plugin metadata (vendor, URL, email) is automatically derived from Cargo.toml.

wavecraft_plugin! {
    name: "{{plugin_name_title}}",

    // Default: gain-only chain (silent — requires external audio input).
    // Uncomment the line below to hear the oscillator example instead.
    //
    // signal: SignalChain![InputGain, OutputGain, Oscillator],
    signal: SignalChain![InputGain, OutputGain],
}
