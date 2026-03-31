//! Parameter host trait - abstraction for plugin parameter management.
//!
//! The `ParameterHost` trait provides the interface between the IPC bridge
//! and the actual parameter storage (typically in the plugin or DAW host).

use crate::error::BridgeError;
use std::sync::Arc;
use wavecraft_protocol::{
    AudioRuntimeStatus, GetHardwareInputSelectionResult, GetInputSourceResult, InputSourceKind,
    MeterFrame, OscilloscopeFrame, ParameterInfo, SetHardwareInputSelectionParams, SignalChainSlot,
};

/// Trait for objects that store and manage parameters.
///
/// This trait abstracts parameter storage, allowing the bridge to work with
/// both standalone applications and plugin hosts. Implementations provide
/// access to parameter values, metadata, and metering data for the UI.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to work with the IPC bridge, which
/// operates across multiple threads (audio thread, UI thread, IPC thread).
///
/// # Example
///
/// ```rust,no_run
/// use wavecraft_bridge::{BridgeError, ParameterHost};
/// use wavecraft_protocol::{
///     AudioRuntimeStatus, MeterFrame, OscilloscopeFrame, ParameterInfo, ParameterType,
/// };
/// use std::sync::{Arc, Mutex};
///
/// struct MyHost {
///     params: Arc<Mutex<Vec<f32>>>,
/// }
///
/// impl ParameterHost for MyHost {
///     fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
///         let idx: usize = id.parse().ok()?;
///         let params = self.params.lock().unwrap();
///         Some(ParameterInfo {
///             id: id.to_string(),
///             name: format!("Param {}", idx),
///             param_type: ParameterType::Float,
///             value: params.get(idx).copied()?,
///             default: 0.5,
///             min: 0.0,
///             max: 1.0,
///             unit: None,
///             group: None,
///             variants: None,
///         })
///     }
///
///     fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
///         let idx: usize = id.parse().map_err(|_| BridgeError::InvalidParams {
///             method: "setParameter".to_string(),
///             reason: format!("Invalid parameter id: {}", id),
///         })?;
///         let mut params = self.params.lock().unwrap();
///         if let Some(param) = params.get_mut(idx) {
///             *param = value;
///             Ok(())
///         } else {
///             Err(BridgeError::ParameterNotFound(id.to_string()))
///         }
///     }
///
///     fn get_all_parameters(&self) -> Vec<ParameterInfo> {
///         // Return all parameters...
///         vec![]
///     }
///
///     fn get_meter_frame(&self) -> Option<MeterFrame> {
///         None
///     }
///
///     fn get_oscilloscope_frame(&self) -> Option<OscilloscopeFrame> {
///         None
///     }
///
///     fn request_resize(&self, _width: u32, _height: u32) -> bool {
///         false
///     }
///
///     fn get_audio_status(&self) -> Option<AudioRuntimeStatus> {
///         None
///     }
/// }
/// ```
pub trait ParameterHost: Send + Sync {
    /// Get information about a single parameter.
    ///
    /// Returns parameter metadata and current value for the given ID.
    ///
    /// # Arguments
    /// * `id` - The parameter identifier (typically a string representation of the enum variant)
    ///
    /// # Returns
    /// The parameter information, or `None` if the ID is invalid.
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo>;

    /// Set a parameter value.
    ///
    /// Updates the parameter to the given value in its declared `[min, max]` range.
    ///
    /// # Arguments
    /// * `id` - The parameter identifier
    /// * `value` - Parameter value in its declared range
    ///
    /// # Returns
    /// `Ok(())` if the parameter was updated, or an error if the ID is invalid
    /// or the value is out of range.
    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError>;

    /// Get all parameters with their current values and metadata.
    ///
    /// This is typically called once when the UI initializes to populate
    /// all controls with their current state.
    ///
    /// # Returns
    /// A vector of all parameter information.
    fn get_all_parameters(&self) -> Vec<ParameterInfo>;

    /// Get the latest meter frame for UI visualization.
    ///
    /// Returns real-time metering data (e.g., peak levels) for display in the UI.
    /// This is typically polled frequently (e.g., 30-60 Hz) for smooth meter updates.
    ///
    /// # Returns
    /// The latest meter data, or `None` if metering is not available.
    fn get_meter_frame(&self) -> Option<MeterFrame>;

    /// Get the latest Passthrough-local meter frame for UI visualization.
    ///
    /// Returns metering data captured at the Passthrough processor's current
    /// runtime location in the signal chain.
    fn get_passthrough_meter_frame(&self) -> Option<MeterFrame> {
        None
    }

    /// Get the latest oscilloscope frame for UI visualization.
    ///
    /// Returns waveform snapshot data for display in the oscilloscope UI.
    /// This is typically polled at render cadence (e.g., requestAnimationFrame).
    ///
    /// # Returns
    /// The latest oscilloscope frame, or `None` if no frame is available.
    fn get_oscilloscope_frame(&self) -> Option<OscilloscopeFrame>;

    /// Request resize of the editor window.
    ///
    /// Asks the host (DAW or standalone window manager) to resize the plugin UI.
    /// The host is free to reject or adjust the requested size based on its policies.
    ///
    /// # Arguments
    /// * `width` - Requested width in logical pixels
    /// * `height` - Requested height in logical pixels
    ///
    /// # Returns
    /// `true` if the host accepted the resize request, `false` if rejected.
    fn request_resize(&self, width: u32, height: u32) -> bool;

    /// Get the current audio runtime status for browser/dev hosts.
    ///
    /// Implementers that do not expose runtime audio state should return `None`.
    fn get_audio_status(&self) -> Option<AudioRuntimeStatus>;

    /// Get the current input source selection and available source options.
    fn get_input_source(&self) -> Option<GetInputSourceResult> {
        None
    }

    /// Set the selected input source.
    fn set_input_source(&self, _source: InputSourceKind) -> Result<(), BridgeError> {
        Err(BridgeError::Internal(
            "input source selection not supported by this host".to_string(),
        ))
    }

    /// Get the current hardware input device/routing selection and options.
    fn get_hardware_input_selection(&self) -> Option<GetHardwareInputSelectionResult> {
        None
    }

    /// Update the selected hardware input device/routing.
    fn set_hardware_input_selection(
        &self,
        _selection: SetHardwareInputSelectionParams,
    ) -> Result<(), BridgeError> {
        Err(BridgeError::Internal(
            "hardware input selection not supported by this host".to_string(),
        ))
    }

    /// Get the current signal chain order as a list of slots.
    ///
    /// Returns an empty vec for hosts that don't support signal chain ordering.
    fn get_signal_chain_order(&self) -> Vec<SignalChainSlot> {
        vec![]
    }

    /// Set the desired signal chain order.
    ///
    /// Returns `Internal` error by default for hosts that don't support
    /// signal chain ordering. The real implementation is generated by the proc-macro
    /// on `__WavecraftParams` and forwarded through `PluginEditorBridge`.
    fn set_signal_chain_order(&self, _order: Vec<SignalChainSlot>) -> Result<(), BridgeError> {
        Err(BridgeError::Internal(
            "signal chain order not supported by this host".to_string(),
        ))
    }
}

/// Blanket implementation for Arc<T> where T: ParameterHost.
///
/// This allows IpcHandler and other components to hold Arc<Host> for shared
/// ownership, which is needed for hot-reload where multiple components need
/// mutable access to the host (e.g., IpcHandler for parameter updates,
/// RebuildPipeline for parameter replacement).
impl<T: ParameterHost> ParameterHost for std::sync::Arc<T> {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        forward_host(self).get_parameter(id)
    }

    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
        forward_host(self).set_parameter(id, value)
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        forward_host(self).get_all_parameters()
    }

    fn get_meter_frame(&self) -> Option<MeterFrame> {
        forward_host(self).get_meter_frame()
    }

    fn get_passthrough_meter_frame(&self) -> Option<MeterFrame> {
        forward_host(self).get_passthrough_meter_frame()
    }

    fn get_oscilloscope_frame(&self) -> Option<OscilloscopeFrame> {
        forward_host(self).get_oscilloscope_frame()
    }

    fn request_resize(&self, width: u32, height: u32) -> bool {
        forward_host(self).request_resize(width, height)
    }

    fn get_audio_status(&self) -> Option<AudioRuntimeStatus> {
        forward_host(self).get_audio_status()
    }

    fn get_input_source(&self) -> Option<GetInputSourceResult> {
        forward_host(self).get_input_source()
    }

    fn set_input_source(&self, source: InputSourceKind) -> Result<(), BridgeError> {
        forward_host(self).set_input_source(source)
    }

    fn get_hardware_input_selection(&self) -> Option<GetHardwareInputSelectionResult> {
        forward_host(self).get_hardware_input_selection()
    }

    fn set_hardware_input_selection(
        &self,
        selection: SetHardwareInputSelectionParams,
    ) -> Result<(), BridgeError> {
        forward_host(self).set_hardware_input_selection(selection)
    }

    fn get_signal_chain_order(&self) -> Vec<SignalChainSlot> {
        forward_host(self).get_signal_chain_order()
    }

    fn set_signal_chain_order(&self, order: Vec<SignalChainSlot>) -> Result<(), BridgeError> {
        forward_host(self).set_signal_chain_order(order)
    }
}

#[inline]
fn forward_host<T: ParameterHost>(host: &Arc<T>) -> &T {
    host.as_ref()
}

/// Trait for accessing the runtime signal chain order from a parameter struct.
///
/// Implemented by `__WavecraftParams` (generated by the proc-macro) and used
/// by `PluginEditorBridge` to read and write signal chain order via the bridge API.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync`. The generated impl uses `AtomicU8`
/// stores for the pending-order slots and a `Mutex<Vec<SignalChainSlot>>` for the canonical
/// order state, making it safe to call from any thread.
pub trait SignalChainOrderAccess: Send + Sync {
    /// Returns the current signal chain order as a list of slots.
    fn get_order(&self) -> Vec<SignalChainSlot>;

    /// Sets the desired signal chain order.
    ///
    /// Returns `InvalidSignalChainOrder` if the provided slots contain empty or duplicate IDs.
    fn set_order(&self, order: Vec<SignalChainSlot>) -> Result<(), BridgeError>;
}
