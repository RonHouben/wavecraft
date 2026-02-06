//! Bridge between nih-plug and the IPC handler.
//!
//! Implements the ParameterHost trait for use with the bridge crate,
//! wrapping nih-plug's GuiContext for parameter automation.

#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::sync::{Arc, Mutex};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use nih_plug::prelude::*;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use wavecraft_bridge::{BridgeError, ParameterHost};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use wavecraft_metering::MeterConsumer;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use wavecraft_protocol::{ParameterInfo, ParameterType};

/// Bridge between nih-plug and the IPC handler.
///
/// This struct implements ParameterHost to allow the IPC handler to
/// interact with nih-plug's parameter system through GuiContext.
///
/// Generic over `P` which must implement nih-plug's `Params` trait.
///
/// Only used on macOS/Windows where WebView is available.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub struct PluginEditorBridge<P: Params> {
    params: Arc<P>,
    context: Arc<dyn GuiContext>,
    /// Optional meter consumer - may be None if metering is disabled
    meter_consumer: Option<Arc<Mutex<MeterConsumer>>>,
    /// Shared editor size - updated when resize is requested
    editor_size: Arc<Mutex<(u32, u32)>>,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl<P: Params> PluginEditorBridge<P> {
    /// Create a new bridge with the given parameters and context.
    pub fn new(
        params: Arc<P>,
        context: Arc<dyn GuiContext>,
        meter_consumer: Option<MeterConsumer>,
        editor_size: Arc<Mutex<(u32, u32)>>,
    ) -> Self {
        Self {
            params,
            context,
            meter_consumer: meter_consumer.map(|c| Arc::new(Mutex::new(c))),
            editor_size,
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl<P: Params> ParameterHost for PluginEditorBridge<P> {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        // Use nih-plug's param_map to find the parameter
        let param_map = self.params.param_map();
        param_map.iter().find_map(|(param_id, param_ptr, _group)| {
            if param_id == id {
                // Access metadata directly from ParamPtr
                let name = unsafe { param_ptr.name() };
                let value = unsafe { param_ptr.modulated_normalized_value() };
                let default = unsafe { param_ptr.default_normalized_value() };
                let unit_str = unsafe { param_ptr.unit() };

                Some(ParameterInfo {
                    id: param_id.clone(),
                    name: name.to_string(),
                    param_type: ParameterType::Float, // For now, assume float
                    value,
                    default,
                    // Convert empty string to None
                    unit: if unit_str.is_empty() {
                        None
                    } else {
                        Some(unit_str.to_string())
                    },
                    // Use group from param_map (third element of tuple)
                    group: if _group.is_empty() {
                        None
                    } else {
                        Some(_group.to_string())
                    },
                })
            } else {
                None
            }
        })
    }

    fn set_parameter(&self, id: &str, normalized_value: f32) -> Result<(), BridgeError> {
        // Use nih-plug's param_map to find the parameter
        let param_map = self.params.param_map();
        if let Some((_, param_ptr, _)) = param_map.iter().find(|(param_id, _, _)| param_id == id) {
            // Use nih-plug's GuiContext for proper host automation
            unsafe {
                self.context.raw_begin_set_parameter(*param_ptr);
                self.context
                    .raw_set_parameter_normalized(*param_ptr, normalized_value);
                self.context.raw_end_set_parameter(*param_ptr);
            }
            Ok(())
        } else {
            Err(BridgeError::ParameterNotFound(id.to_string()))
        }
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        // Iterate over all parameters in the param_map
        let param_map = self.params.param_map();
        param_map
            .iter()
            .map(|(param_id, param_ptr, _group)| {
                // Access metadata directly from ParamPtr
                let name = unsafe { param_ptr.name() };
                let value = unsafe { param_ptr.modulated_normalized_value() };
                let default = unsafe { param_ptr.default_normalized_value() };
                let unit_str = unsafe { param_ptr.unit() };

                ParameterInfo {
                    id: param_id.clone(),
                    name: name.to_string(),
                    param_type: ParameterType::Float, // For now, assume float
                    value,
                    default,
                    // Convert empty string to None
                    unit: if unit_str.is_empty() {
                        None
                    } else {
                        Some(unit_str.to_string())
                    },
                    // Use group from param_map (third element of tuple)
                    group: if _group.is_empty() {
                        None
                    } else {
                        Some(_group.to_string())
                    },
                }
            })
            .collect()
    }

    fn get_meter_frame(&self) -> Option<wavecraft_protocol::MeterFrame> {
        // Read latest meter frame from the shared consumer if available
        let consumer = self.meter_consumer.as_ref()?;
        let mut consumer = consumer.lock().unwrap();
        consumer
            .read_latest()
            .map(|frame| wavecraft_protocol::MeterFrame {
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
