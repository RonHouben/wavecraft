//! Placeholder egui editor.
//!




























































































































}    }        assert_eq!(all[0].id, "gain");        assert_eq!(all.len(), 1);        let all = bridge.get_all_parameters();        let bridge = PluginEditorBridge::new(params, setter);        let setter = Arc::new(MockParamSetter);        let params = Arc::new(VstKitParams::default());    fn bridge_get_all_parameters() {    #[test]    }        assert!(bridge.set_parameter("unknown", 0.5).is_err());        // Should fail for unknown param        assert!(bridge.set_parameter("gain", 0.5).is_ok());        // Should succeed for valid param        let bridge = PluginEditorBridge::new(params, setter);        let setter = Arc::new(MockParamSetter);        let params = Arc::new(VstKitParams::default());    fn bridge_set_parameter() {    #[test]    }        assert_eq!(info.name, "Gain");        assert_eq!(info.id, "gain");        let info = bridge.get_parameter("gain").expect("gain param exists");        let bridge = PluginEditorBridge::new(params, setter);        let setter = Arc::new(MockParamSetter);        let params = Arc::new(VstKitParams::default());    fn bridge_get_parameter() {    #[test]    }        fn end_set_parameter<P: Param>(&self, _param: &P) {}        fn begin_set_parameter<P: Param>(&self, _param: &P) {}        fn set_parameter_normalized<P: Param>(&self, _param: &P, _normalized: f32) {}        fn set_parameter<P: Param>(&self, _param: &P, _value: P::Plain) {}    impl ParamSetter for MockParamSetter {    struct MockParamSetter;    // Mock ParamSetter for testing    use super::*;mod tests {#[cfg(test)]}    }        self.get_parameter("gain").into_iter().collect()        // For now, just return the gain parameter    fn get_all_parameters(&self) -> Vec<ParameterInfo> {    }        }            Err(BridgeError::ParameterNotFound(id.to_string()))        } else {            Ok(())                .set_parameter(&self.params.gain, linear_value);            self.param_setter            // Use nih-plug's parameter setter for proper host automation            let linear_value = normalized_to_linear(normalized_value, spec.min, spec.max);            let spec = &PARAM_SPECS[0];        if id == "gain" {        // For now, we only have the gain parameter    fn set_parameter(&self, id: &str, normalized_value: f32) -> Result<(), BridgeError> {    }        }            None        } else {            })                default: linear_to_normalized(spec.default, spec.min, spec.max),                max: 1.0,                min: 0.0,                display_value: format!("{:.1} dB", current_value),                value: normalized,                name: spec.name.to_string(),                id: spec.id.to_string(),            Some(ParameterInfo {            let normalized = linear_to_normalized(current_value, spec.min, spec.max);            let current_value = self.params.gain.value();            let spec = &PARAM_SPECS[0]; // Gain is first param        if id == "gain" {        // For now, we only have the gain parameter    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {impl ParameterHost for PluginEditorBridge {}    }        }            param_setter,            params,        Self {    pub fn new(params: Arc<VstKitParams>, param_setter: Arc<dyn ParamSetter>) -> Self {    /// Create a new bridge with the given parameters and setter.impl PluginEditorBridge {}    param_setter: Arc<dyn ParamSetter>,    params: Arc<VstKitParams>,pub struct PluginEditorBridge {/// interact with nih-plug's parameter system through ParamSetter./// This struct implements ParameterHost to allow the IPC handler to////// Bridge between nih-plug and the IPC handler.use crate::params::VstKitParams;use protocol::{linear_to_normalized, normalized_to_linear, ParameterInfo, PARAM_SPECS};use nih_plug::prelude::*;use bridge::{BridgeError, ParameterHost};use std::sync::Arc;//! wrapping nih-plug's ParamSetter for parameter automation.//! Implements the ParameterHost trait for use with the bridge crate,//! Provides a minimal native UI for parameter visualization and manipulation
//! until the WebView UI is implemented.

use std::sync::Arc;

use nih_plug::prelude::*;
use nih_plug_egui::egui::{CentralPanel, Slider};
use nih_plug_egui::{create_egui_editor, EguiState};

use crate::params::VstKitParams;

/// Default editor width in pixels.
const EDITOR_WIDTH: u32 = 400;
/// Default editor height in pixels.
const EDITOR_HEIGHT: u32 = 300;

/// Create the placeholder egui editor.
pub fn create_egui_editor(params: Arc<VstKitParams>) -> Option<Box<dyn Editor>> {
    let state = EguiState::from_size(EDITOR_WIDTH, EDITOR_HEIGHT);

    create_egui_editor(
        state,
        (),
        |_, _| {},
        move |ctx, setter, _state| {
            CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("VstKit — Placeholder UI");
                    ui.add_space(20.0);

                    // Gain slider - using proper drag interaction pattern
                    ui.label("Gain");
                    let mut gain_value = params.gain.value();
                    let slider = Slider::new(&mut gain_value, -24.0..=24.0)
                        .suffix(" dB")
                        .step_by(0.1);

                    let response = ui.add(slider);
                    
                    // Only notify host when user is actively dragging
                    if response.drag_started() {
                        setter.begin_set_parameter(&params.gain);
                    }
                    if response.dragged() || response.changed() && response.has_focus() {
                        setter.set_parameter(&params.gain, gain_value);
                    }
                    if response.drag_stopped() {
                        setter.end_set_parameter(&params.gain);
                    }

                    ui.add_space(10.0);

                    // Current value display
                    ui.label(format!("Current: {:.1} dB", params.gain.value()));

                    ui.add_space(30.0);
                    ui.label("(WebView UI coming soon)");
                });
            });
        },
    )
}
