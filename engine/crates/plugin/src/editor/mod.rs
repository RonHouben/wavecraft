//! Plugin editor module - WebView and parameter integration.
//!
//! This module provides the nih-plug Editor implementation, bridging
//! the WebView UI with the plugin's parameter system and metering.

use std::any::Any;
use std::sync::{Arc, Mutex};

use metering::MeterConsumer;
use nih_plug::prelude::*;

use crate::params::VstKitParams;

mod assets;
mod bridge;
mod webview;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

pub use webview::{WebViewConfig, WebViewHandle, create_webview};

/// WebView-based editor for the plugin.
///
/// This editor creates a WebView that hosts the React UI and handles
/// bidirectional parameter synchronization and metering.
pub struct VstKitEditor {
    params: Arc<VstKitParams>,
    /// Shared meter consumer - cloned to each bridge instance
    meter_consumer: Arc<Mutex<MeterConsumer>>,
    size: Arc<Mutex<(u32, u32)>>,
    /// Handle to the WebView for resize operations
    webview_handle: Arc<Mutex<Option<Box<dyn WebViewHandle>>>>,
}

impl VstKitEditor {
    /// Create a new WebView editor.
    pub fn new(params: Arc<VstKitParams>, meter_consumer: Arc<Mutex<MeterConsumer>>) -> Self {
        Self {
            params,
            meter_consumer,
            size: Arc::new(Mutex::new((800, 800))), // Default size - increased to show all content
            webview_handle: Arc::new(Mutex::new(None)),
        }
    }
}

impl Editor for VstKitEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send> {
        // Clone the shared meter consumer for this editor instance
        let meter_consumer = self.meter_consumer.clone();

        let size = *self.size.lock().unwrap();

        let config = WebViewConfig {
            params: self.params.clone(),
            context,
            parent,
            width: size.0,
            height: size.1,
            meter_consumer,
            editor_size: self.size.clone(),
        };

        match create_webview(config) {
            Ok(webview) => {
                // Store the webview handle for resize operations
                *self.webview_handle.lock().unwrap() = Some(webview);

                // Return a dummy value that the host will hold
                Box::new(())
            }
            Err(e) => {
                nih_error!("Failed to create WebView editor: {}", e);
                Box::new(())
            }
        }
    }

    fn size(&self) -> (u32, u32) {
        *self.size.lock().unwrap()
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        // We don't support DPI scaling yet
        false
    }

    fn param_value_changed(&self, id: &str, normalized_value: f32) {
        // Log for debugging automation updates
        nih_log!(
            "param_value_changed called: id={}, value={}",
            id,
            normalized_value
        );

        // Push parameter update to WebView via JavaScript evaluation
        if let Ok(webview_lock) = self.webview_handle.lock() {
            if let Some(webview) = webview_lock.as_ref() {
                // Escape the id for safe JavaScript injection
                let id_escaped = id.replace('\\', "\\\\").replace('"', "\\\"");
                let js = format!(
                    "if (globalThis.__VSTKIT_IPC__ && globalThis.__VSTKIT_IPC__._onParamUpdate) {{ \
                        globalThis.__VSTKIT_IPC__._onParamUpdate({{ \
                            jsonrpc: \"2.0\", \
                            method: \"parameterChanged\", \
                            params: {{ id: \"{}\", value: {} }} \
                        }}); \
                    }}",
                    id_escaped, normalized_value
                );

                if let Err(e) = webview.evaluate_script(&js) {
                    nih_error!("Failed to evaluate parameter update script: {}", e);
                } else {
                    nih_log!("Successfully pushed parameter update to UI");
                }
            } else {
                nih_log!("WebView handle is None, cannot push update");
            }
        } else {
            nih_log!("Failed to lock webview_handle");
        }
    }

    fn param_values_changed(&self) {
        // Called when multiple parameters change at once
        // For now, we rely on individual param_value_changed calls
    }

    fn param_modulation_changed(&self, id: &str, modulation_offset: f32) {
        // Push modulation update to WebView via JavaScript evaluation
        if let Ok(webview_lock) = self.webview_handle.lock()
            && let Some(webview) = webview_lock.as_ref()
        {
            let id_escaped = id.replace('\\', "\\\\").replace('"', "\\\"");
            let js = format!(
                "if (globalThis.__VSTKIT_IPC__ && globalThis.__VSTKIT_IPC__._onParamUpdate) {{ \
                    globalThis.__VSTKIT_IPC__._onParamUpdate({{ \
                        jsonrpc: \"2.0\", \
                        method: \"paramModulation\", \
                        params: {{ id: \"{}\", offset: {} }} \
                    }}); \
                }}",
                id_escaped, modulation_offset
            );
            let _ = webview.evaluate_script(&js);
        }
    }
}

/// Create a WebView editor.
pub fn create_webview_editor(
    params: Arc<VstKitParams>,
    meter_consumer: Arc<Mutex<MeterConsumer>>,
) -> Option<Box<dyn Editor>> {
    Some(Box::new(VstKitEditor::new(params, meter_consumer)))
}
