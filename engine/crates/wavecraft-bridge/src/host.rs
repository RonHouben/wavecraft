//! Parameter host trait - abstraction for plugin parameter management.
//!
//! The `ParameterHost` trait provides the interface between the IPC bridge
//! and the actual parameter storage (typically in the plugin or DAW host).

use crate::error::BridgeError;
use wavecraft_protocol::{MeterFrame, ParameterInfo};

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
/// use wavecraft_protocol::{MeterFrame, ParameterInfo, ParameterType};
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
///             unit: None,
///             group: None,
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
///     fn request_resize(&self, _width: u32, _height: u32) -> bool {
///         false
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
    /// Updates the parameter to the given normalized value [0.0, 1.0].
    /// The implementation should convert this to the parameter's actual range.
    ///
    /// # Arguments
    /// * `id` - The parameter identifier
    /// * `value` - Normalized value (0.0 = min, 1.0 = max)
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
}
