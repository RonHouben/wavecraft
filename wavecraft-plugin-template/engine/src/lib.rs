// Import everything from Wavecraft SDK
use wavecraft_core::prelude::*;

// Define the processor chain - in this case, just a Gain processor
wavecraft_processor!(MyGain => Gain);

// Generate the complete plugin from DSL
wavecraft_plugin! {
    name: "My Plugin",
    vendor: "My Company",
    url: "https://example.com",
    email: "contact@example.com",
    signal: MyGain,
}

