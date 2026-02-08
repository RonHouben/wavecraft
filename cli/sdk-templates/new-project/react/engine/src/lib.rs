// Import everything from Wavecraft SDK
use wavecraft::prelude::*;

// Define the processor chain - in this case, just a Gain processor
wavecraft_processor!({{plugin_name_pascal}}Gain => Gain);

// Generate the complete plugin from DSL
// Plugin metadata (vendor, URL, email) is automatically derived from Cargo.toml
wavecraft_plugin! {
    name: "{{plugin_name_title}}",
    signal: SignalChain![{{plugin_name_pascal}}Gain],
}
