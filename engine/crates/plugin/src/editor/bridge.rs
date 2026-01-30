//! Bridge between nih-plug and the IPC handler.
//!
//! Implements the ParameterHost trait for use with the bridge crate,
//! wrapping nih-plug's GuiContext for parameter automation.

use std::sync::Arc;

use bridge::{BridgeError, ParameterHost};
use nih_plug::prelude::*;
use protocol::{ParameterInfo, ParameterType};

use crate::params::VstKitParams;

/// Bridge between nih-plug and the IPC handler.
///
/// This struct implements ParameterHost to allow the IPC handler to
/// interact with nih-plug's parameter system through GuiContext.
pub struct PluginEditorBridge {
    params: Arc<VstKitParams>,
    context: Arc<dyn GuiContext>,
}

impl PluginEditorBridge {
    /// Create a new bridge with the given parameters and context.
    pub fn new(params: Arc<VstKitParams>, context: Arc<dyn GuiContext>) -> Self {
        Self { params, context }
    }
}

impl ParameterHost for PluginEditorBridge {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        // For now, we only have the gain parameter
        if id == "gain" {
            let normalized = self.params.gain.modulated_normalized_value();
            Some(ParameterInfo {
                id: "gain".to_string(),
                name: "Gain".to_string(),
                param_type: ParameterType::Float,
                value: normalized,
                default: self.params.gain.default_normalized_value(),
                unit: Some("dB".to_string()),
            })
        } else {
            None
        }
    }

    fn set_parameter(&self, id: &str, normalized_value: f32) -> Result<(), BridgeError> {
        // For now, we only have the gain parameter
        if id == "gain" {
            // Use nih-plug's GuiContext for proper host automation
            let param_ptr = self.params.gain.as_ptr();
            unsafe {
                self.context.raw_begin_set_parameter(param_ptr);
                self.context
                    .raw_set_parameter_normalized(param_ptr, normalized_value);
                self.context.raw_end_set_parameter(param_ptr);
            }
            Ok(())
        } else {
            Err(BridgeError::ParameterNotFound(id.to_string()))
        }
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        // For now, just return the gain parameter
        self.get_parameter("gain").into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Testing PluginEditorBridge requires a mock GuiContext,
    // which is complex. These tests are placeholders for future integration tests.

    #[test]
    fn test_placeholder() {
        // TODO: Add tests once we have a mock GuiContext
        assert!(true);
    }
}
