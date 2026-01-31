//! Placeholder egui editor.
//!
//! Provides a minimal native UI for parameter visualization and manipulation
//! until the WebView UI is implemented.

use std::sync::Arc;

use nih_plug::prelude::*;
use nih_plug_egui::egui::{CentralPanel, Slider};
use nih_plug_egui::{EguiState, create_egui_editor as nih_create_egui_editor};

use crate::params::VstKitParams;

/// Create the placeholder egui editor.
#[allow(dead_code)] // Placeholder for alternative UI backend
pub fn create_egui_editor(params: Arc<VstKitParams>) -> Option<Box<dyn Editor>> {
    let state = EguiState::from_size(400, 300);

    nih_create_egui_editor(
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
