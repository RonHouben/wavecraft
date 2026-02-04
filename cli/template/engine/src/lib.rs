// Import everything from Wavecraft SDK
use wavecraft_core::prelude::*;

// Define the processor chain - in this case, just a Gain processor
wavecraft_processor!({{plugin_name_pascal}}Gain => Gain);

// Generate the complete plugin from DSL
wavecraft_plugin! {
    name: "{{plugin_name_title}}",
    vendor: "{{vendor}}",
    url: "{{url}}",
    email: "{{email}}",
    signal: {{plugin_name_pascal}}Gain,
}

