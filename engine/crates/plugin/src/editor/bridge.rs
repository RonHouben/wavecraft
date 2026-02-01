//! Bridge between nih-plug and the IPC handler.
//!
//! Implements the ParameterHost trait for use with the bridge crate,
//! wrapping nih-plug's GuiContext for parameter automation.

#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::sync::{Arc, Mutex};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use bridge::{BridgeError, ParameterHost};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use metering::MeterConsumer;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use nih_plug::prelude::*;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use vstkit_protocol::{ParameterInfo, ParameterType};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::params::VstKitParams;

/// Bridge between nih-plug and the IPC handler.
///
/// This struct implements ParameterHost to allow the IPC handler to
/// interact with nih-plug's parameter system through GuiContext.
///
/// Only used on macOS/Windows where WebView is available.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub struct PluginEditorBridge {
    params: Arc<VstKitParams>,
    context: Arc<dyn GuiContext>,
    /// Shared meter consumer - same instance used across editor open/close cycles
    meter_consumer: Arc<Mutex<MeterConsumer>>,
    /// Shared editor size - updated when resize is requested
    editor_size: Arc<Mutex<(u32, u32)>>,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl PluginEditorBridge {
    /// Create a new bridge with the given parameters and context.
    pub fn new(
        params: Arc<VstKitParams>,
        context: Arc<dyn GuiContext>,
        meter_consumer: Arc<Mutex<MeterConsumer>>,
        editor_size: Arc<Mutex<(u32, u32)>>,
    ) -> Self {
        Self {
            params,
            context,
            meter_consumer,
            editor_size,
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
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

    fn get_meter_frame(&self) -> Option<vstkit_protocol::MeterFrame> {
        // Read latest meter frame from the shared consumer
        let mut consumer = self.meter_consumer.lock().unwrap();
        consumer.read_latest().map(|frame| vstkit_protocol::MeterFrame {
            peak_l: frame.peak_l,
            peak_r: frame.peak_r,
            rms_l: frame.rms_l,
            rms_r: frame.rms_r,
            timestamp: frame.timestamp,
        })
    }

    fn request_resize(&self, width: u32, height: u32) -> bool {
        // Update the editor's size field
        *self.editor_size.lock().unwrap() = (width, height);

        nih_log!("Resize requested: {}x{}", width, height);

        // Call GuiContext::request_resize() which notifies the host
        // The host will call Editor::size() to get the new size
        let accepted = self.context.request_resize();

        if accepted {
            nih_log!("Resize accepted by host");
        } else {
            nih_log!("Resize rejected by host");
            // Revert size if rejected
            // (In practice, most hosts just accept whatever size is reported)
        }

        accepted
    }
}

#[cfg(test)]
mod tests {
    // Note: Testing PluginEditorBridge requires a mock GuiContext,
    // which is complex. These tests are placeholders for future integration tests.

    #[test]
    fn test_placeholder() {
        // TODO: Add tests once we have a mock GuiContext
    }
}
