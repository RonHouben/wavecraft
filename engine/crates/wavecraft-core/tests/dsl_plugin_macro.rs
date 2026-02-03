//! Tests for the new DSL-based wavecraft_plugin! macro (Phase 6).
//!
//! This tests the simplified syntax with automatic parameter generation.

use wavecraft_core::wavecraft_processor;
use wavecraft_dsp::Chain;

// Define a custom processor type wrapping the built-in Gain
wavecraft_processor!(TestGain => Gain);

// Use the new wavecraft_plugin! macro from wavecraft_macros
// Note: This uses the proc-macro version, not the old declarative macro
wavecraft_macros::wavecraft_plugin! {
    name: "Test DSL Plugin",
    vendor: "Test Vendor",
    url: "https://example.com",
    email: "test@example.com",
    signal: TestGain,
}

#[cfg(test)]
mod tests {
    use super::*;
    use nih_plug::prelude::Plugin;

    #[test]
    fn test_plugin_compiles() {
        // Verify the macro generates compilable code
        let _plugin = __WavecraftPlugin::default();
    }

    #[test]
    fn test_plugin_metadata() {
        assert_eq!(__WavecraftPlugin::NAME, "Test DSL Plugin");
        assert_eq!(__WavecraftPlugin::VENDOR, "Test Vendor");
        assert_eq!(__WavecraftPlugin::URL, "https://example.com");
        assert_eq!(__WavecraftPlugin::EMAIL, "test@example.com");
    }

    #[test]
    fn test_plugin_has_params() {
        let plugin = __WavecraftPlugin::default();
        let params = plugin.params();
        
        // Should have parameters from the Gain processor
        assert!(!params.param_map().is_empty());
    }
}
