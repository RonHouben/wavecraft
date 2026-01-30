//! Plugin editor module - WebView and parameter integration.
//!
//! This module provides the nih-plug Editor implementation, bridging
//! the WebView UI with the plugin's parameter system and metering.

use std::any::Any;
use std::sync::{Arc, Mutex};

use nih_plug::prelude::*;

use crate::params::VstKitParams;

mod assets;
mod bridge;
mod egui;
mod webview;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

pub use self::egui::create_egui_editor as create_editor;
pub use bridge::PluginEditorBridge;
pub use webview::{create_webview, WebViewConfig};

/// WebView-based editor for the plugin.
///
/// This editor creates a WebView that hosts the React UI and handles
/// bidirectional parameter synchronization and metering.
pub struct VstKitEditor {
    params: Arc<VstKitParams>,
    size: (u32, u32),
}

impl VstKitEditor {
    /// Create a new WebView editor.
    pub fn new(params: Arc<VstKitParams>) -> Self {
        Self {
            params,
            size: (800, 600), // Default size
        }
    }
}

impl Editor for VstKitEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send> {
        let config = WebViewConfig {
            params: self.params.clone(),
            context,
            parent,
            width: self.size.0,
            height: self.size.1,
        };

        match create_webview(config) {
            Ok(webview) => {
                // Return the webview handle as Any + Send
                // The host will drop it when the editor is closed
                webview as Box<dyn Any + Send>
            }
            Err(e) => {
                nih_error!("Failed to create WebView editor: {}", e);
                Box::new(())
            }
        }
    }

    fn size(&self) -> (u32, u32) {
        self.size
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        // We don't support DPI scaling yet
        false
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {
        // TODO: Push parameter update to the WebView
        // We'll need to implement this once we have a way to communicate with the webview
    }

    fn param_values_changed(&self) {
        // Called when multiple parameters change at once
        // Not implemented yet
    }

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {
        // Not implemented yet
    }
}

/// Create a WebView editor.
pub fn create_webview_editor(params: Arc<VstKitParams>) -> Option<Box<dyn Editor>> {
    Some(Box::new(VstKitEditor::new(params)))
}
