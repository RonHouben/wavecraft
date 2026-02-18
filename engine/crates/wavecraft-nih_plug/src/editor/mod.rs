//! Plugin editor module - WebView and parameter integration.
//!
//! This module provides the nih-plug Editor implementation, bridging
//! the WebView UI with the plugin's parameter system and metering.

#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::any::Any;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::sync::{Arc, Mutex};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use nih_plug::prelude::*;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use wavecraft_metering::MeterConsumer;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use wavecraft_processors::OscilloscopeFrameConsumer;

mod assets;
mod bridge;
mod webview;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use webview::{WebViewConfig, WebViewHandle, create_webview};

/// WebView-based editor for the plugin.
///
/// This editor creates a WebView that hosts the React UI and handles
/// bidirectional parameter synchronization and metering.
///
/// Generic over `P` which must implement nih-plug's `Params` trait.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub struct WavecraftEditor<P: Params> {
    params: Arc<P>,
    /// Meter consumer for audio metering - taken on first editor spawn
    meter_consumer: Mutex<Option<MeterConsumer>>,
    /// Oscilloscope consumer for waveform snapshots - taken on first editor spawn
    oscilloscope_consumer: Mutex<Option<OscilloscopeFrameConsumer>>,
    size: Arc<Mutex<(u32, u32)>>,
    /// Handle to the WebView for resize operations
    webview_handle: Arc<Mutex<Option<Box<dyn WebViewHandle>>>>,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl<P: Params> WavecraftEditor<P> {
    /// Create a new WebView editor with specified dimensions.
    ///
    /// # Arguments
    ///
    /// * `params` - Shared parameter state
    /// * `meter_consumer` - Optional meter consumer for audio metering
    /// * `width` - Initial editor width in pixels
    /// * `height` - Initial editor height in pixels
    pub fn new(
        params: Arc<P>,
        meter_consumer: Option<MeterConsumer>,
        oscilloscope_consumer: Option<OscilloscopeFrameConsumer>,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            params,
            meter_consumer: Mutex::new(meter_consumer),
            oscilloscope_consumer: Mutex::new(oscilloscope_consumer),
            size: Arc::new(Mutex::new((width, height))),
            webview_handle: Arc::new(Mutex::new(None)),
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl<P: Params> Editor for WavecraftEditor<P> {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send> {
        // Take the meter consumer (only works for first editor instance)
        let meter_consumer = self.meter_consumer.lock().unwrap().take();
        let oscilloscope_consumer = self.oscilloscope_consumer.lock().unwrap().take();

        let size = *self.size.lock().unwrap();

        let config = WebViewConfig {
            params: self.params.clone(),
            context,
            parent,
            width: size.0,
            height: size.1,
            meter_consumer,
            oscilloscope_consumer,
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
        let plain_value = self
            .params
            .param_map()
            .iter()
            .find(|(param_id, _, _)| param_id == id)
            .map(|(_, param_ptr, _)| {
                // SAFETY: ParamPtr values come from `self.params.param_map()` and remain valid
                // for the lifetime of `self.params` (held by Arc on this editor).
                unsafe { param_ptr.preview_plain(normalized_value) }
            })
            .unwrap_or(normalized_value);

        // Log for debugging automation updates
        nih_log!(
            "param_value_changed called: id={}, normalized={}, plain={}",
            id,
            normalized_value,
            plain_value
        );

        // Push parameter update to WebView via JavaScript evaluation
        if let Ok(webview_lock) = self.webview_handle.lock() {
            if let Some(webview) = webview_lock.as_ref() {
                // Escape the id for safe JavaScript injection
                let id_escaped = id.replace('\\', "\\\\").replace('"', "\\\"");
                let js = format!(
                    "if (globalThis.__WAVECRAFT_IPC__ && globalThis.__WAVECRAFT_IPC__._onParamUpdate) {{ \
                        globalThis.__WAVECRAFT_IPC__._onParamUpdate({{ \
                            jsonrpc: \"2.0\", \
                            method: \"parameterChanged\", \
                            params: {{ id: \"{}\", value: {} }} \
                        }}); \
                    }}",
                    id_escaped, plain_value
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
                "if (globalThis.__WAVECRAFT_IPC__ && globalThis.__WAVECRAFT_IPC__._onParamUpdate) {{ \
                    globalThis.__WAVECRAFT_IPC__._onParamUpdate({{ \
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
///
/// Generic over `P` which must implement nih-plug's `Params` trait.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn create_webview_editor<P: Params + 'static>(
    params: Arc<P>,
    meter_consumer: Option<MeterConsumer>,
    oscilloscope_consumer: Option<OscilloscopeFrameConsumer>,
    width: u32,
    height: u32,
) -> Option<Box<dyn Editor>> {
    Some(Box::new(WavecraftEditor::new(
        params,
        meter_consumer,
        oscilloscope_consumer,
        width,
        height,
    )))
}
