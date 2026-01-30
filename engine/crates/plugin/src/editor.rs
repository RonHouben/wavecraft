//! Placeholder egui editor.
//!
//! Provides a minimal native UI for parameter visualization and manipulation
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
pub fn create_editor(params: Arc<VstKitParams>) -> Option<Box<dyn Editor>> {
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

                    // Gain slider
                    ui.label("Gain");
                    let mut gain_value = params.gain.value();
                    let slider = Slider::new(&mut gain_value, -24.0..=24.0)
                        .suffix(" dB")
                        .step_by(0.1);

                    if ui.add(slider).changed() {
                        setter.begin_set_parameter(&params.gain);
                        setter.set_parameter(&params.gain, gain_value);
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
