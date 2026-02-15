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
use wavecraft_protocol::{AudioRuntimeStatus, ParameterInfo, ParameterType};

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

    fn parameter_info_from_ptr(param_id: &str, param_ptr: ParamPtr, group: &str) -> ParameterInfo {
        // SAFETY: ParamPtr values come from `self.params.param_map()`, and `self.params` is
        // kept alive by `Arc<P>` on this struct for the full bridge lifetime.
        let (name, unit_str, value, default, mut min, mut max) = unsafe {
            (
                param_ptr.name(),
                param_ptr.unit(),
                param_ptr.modulated_plain_value(),
                param_ptr.default_plain_value(),
                param_ptr.preview_plain(0.0),
                param_ptr.preview_plain(1.0),
            )
        };

        if min > max {
            std::mem::swap(&mut min, &mut max);
        }

        ParameterInfo {
            id: param_id.to_string(),
            name: name.to_string(),
            param_type: ParameterType::Float,
            value,
            default,
            min,
            max,
            unit: if unit_str.is_empty() {
                None
            } else {
                Some(unit_str.to_string())
            },
            group: if group.is_empty() {
                None
            } else {
                Some(group.to_string())
            },
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
                Some(Self::parameter_info_from_ptr(param_id, *param_ptr, _group))
            } else {
                None
            }
        })
    }

    fn set_parameter(&self, id: &str, plain_value: f32) -> Result<(), BridgeError> {
        // Use nih-plug's param_map to find the parameter
        let param_map = self.params.param_map();
        if let Some((_, param_ptr, _)) = param_map.iter().find(|(param_id, _, _)| param_id == id) {
            // SAFETY: ParamPtr is valid while `self.params` is alive (kept by Arc), and
            // `preview_normalized` is a pure conversion on the parameter's own range mapping.
            let normalized_value = unsafe { param_ptr.preview_normalized(plain_value) };
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
                Self::parameter_info_from_ptr(param_id, *param_ptr, _group)
            })
            .collect()
    }

    fn get_meter_frame(&self) -> Option<wavecraft_protocol::MeterFrame> {
        // Read latest meter frame from the shared consumer if available
        let consumer = self.meter_consumer.as_ref()?;
        let mut consumer = consumer.lock().unwrap();
        consumer.read_latest()
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

    fn get_audio_status(&self) -> Option<AudioRuntimeStatus> {
        None
    }
}

#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    #[derive(Params)]
    struct TestParams {
        #[id = "freq"]
        freq: FloatParam,

        #[id = "lvl"]
        level: FloatParam,

        #[id = "ena"]
        enabled: BoolParam,
    }

    impl Default for TestParams {
        fn default() -> Self {
            Self {
                freq: FloatParam::new(
                    "Frequency",
                    440.0,
                    FloatRange::Skewed {
                        min: 20.0,
                        max: 5_000.0,
                        factor: 2.5,
                    },
                )
                .with_unit(" Hz"),
                level: FloatParam::new("Level", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
                enabled: BoolParam::new("Enabled", true),
            }
        }
    }

    struct MockGuiContext {
        resize_accepted: bool,
        set_calls: Mutex<Vec<(ParamPtr, f32)>>,
    }

    impl MockGuiContext {
        fn new(resize_accepted: bool) -> Self {
            Self {
                resize_accepted,
                set_calls: Mutex::new(Vec::new()),
            }
        }
    }

    impl GuiContext for MockGuiContext {
        fn plugin_api(&self) -> PluginApi {
            PluginApi::Standalone
        }

        fn request_resize(&self) -> bool {
            self.resize_accepted
        }

        unsafe fn raw_begin_set_parameter(&self, _param: ParamPtr) {}

        unsafe fn raw_set_parameter_normalized(&self, param: ParamPtr, normalized: f32) {
            self.set_calls
                .lock()
                .expect("set_calls lock poisoned")
                .push((param, normalized));
        }

        unsafe fn raw_end_set_parameter(&self, _param: ParamPtr) {}

        fn get_state(&self) -> PluginState {
            PluginState {
                version: String::new(),
                params: BTreeMap::new(),
                fields: BTreeMap::new(),
            }
        }

        fn set_state(&self, _state: PluginState) {}
    }

    #[test]
    fn get_parameter_returns_plain_frequency_range_and_default() {
        let params = Arc::new(TestParams::default());
        let context = Arc::new(MockGuiContext::new(true));
        let bridge =
            PluginEditorBridge::new(params, context, None, Arc::new(Mutex::new((800, 600))));

        let frequency = bridge
            .get_parameter("freq")
            .expect("frequency parameter should exist");

        assert_eq!(frequency.id, "freq");
        assert_eq!(frequency.name, "Frequency");
        assert!((frequency.min - 20.0).abs() < 1e-5);
        assert!((frequency.max - 5_000.0).abs() < 1e-3);
        assert!((frequency.default - 440.0).abs() < 1e-4);
        assert!((frequency.value - 440.0).abs() < 1e-4);
    }

    #[test]
    fn set_parameter_converts_plain_to_normalized_before_host_call() {
        let params = Arc::new(TestParams::default());
        let context = Arc::new(MockGuiContext::new(true));

        let expected_normalized = {
            let param_map = params.param_map();
            let (_, param_ptr, _) = param_map
                .iter()
                .find(|(id, _, _)| id == "freq")
                .expect("freq pointer should exist");
            // SAFETY: ParamPtr originates from `params.param_map()` and `params` is alive.
            unsafe { param_ptr.preview_normalized(2_000.0) }
        };

        let bridge = PluginEditorBridge::new(
            params,
            context.clone(),
            None,
            Arc::new(Mutex::new((800, 600))),
        );

        bridge
            .set_parameter("freq", 2_000.0)
            .expect("setting frequency should succeed");

        let calls = context.set_calls.lock().expect("set_calls lock poisoned");
        let (_, normalized) = calls.last().expect("expected a set_parameter call");
        assert!((*normalized - expected_normalized).abs() < 1e-5);
    }
}
