//! Development server host implementing ParameterHost trait
//!
//! This module provides a ParameterHost implementation for the embedded
//! development server. It stores parameter values in memory and forwards
//! parameter changes to an optional AtomicParameterBridge for lock-free
//! audio-thread access.

#[cfg(feature = "audio")]
use std::sync::Arc;
use wavecraft_bridge::{BridgeError, InMemoryParameterHost, ParameterHost};
use wavecraft_protocol::{MeterFrame, ParameterInfo};

#[cfg(feature = "audio")]
use crate::audio::atomic_params::AtomicParameterBridge;

/// Development server host for browser-based UI testing
///
/// This implementation stores parameter values locally and optionally
/// forwards updates to an `AtomicParameterBridge` for lock-free reads
/// on the audio thread. Meter data is provided externally via the audio
/// server's meter channel (not generated synthetically).
///
/// # Thread Safety
///
/// Parameter state is protected by RwLock (in `InMemoryParameterHost`).
/// The `AtomicParameterBridge` uses lock-free atomics for audio thread.
pub struct DevServerHost {
    inner: InMemoryParameterHost,
    #[cfg(feature = "audio")]
    param_bridge: Option<Arc<AtomicParameterBridge>>,
}

impl DevServerHost {
    /// Create a new dev server host with parameter metadata.
    ///
    /// # Arguments
    ///
    /// * `parameters` - Parameter metadata loaded from the plugin FFI
    ///
    /// Used by tests and the non-audio build path. When `audio` is
    /// enabled (default), production code uses `with_param_bridge()` instead.
    #[cfg_attr(feature = "audio", allow(dead_code))]
    pub fn new(parameters: Vec<ParameterInfo>) -> Self {
        let inner = InMemoryParameterHost::new(parameters);

        Self {
            inner,
            #[cfg(feature = "audio")]
            param_bridge: None,
        }
    }

    /// Create a new dev server host with an `AtomicParameterBridge`.
    ///
    /// When a bridge is provided, `set_parameter()` will write updates
    /// to both the inner store and the bridge (for audio-thread reads).
    #[cfg(feature = "audio")]
    pub fn with_param_bridge(
        parameters: Vec<ParameterInfo>,
        bridge: Arc<AtomicParameterBridge>,
    ) -> Self {
        let inner = InMemoryParameterHost::new(parameters);

        Self {
            inner,
            param_bridge: Some(bridge),
        }
    }

    /// Replace all parameters with new metadata from a hot-reload.
    ///
    /// Preserves values for parameters with matching IDs. New parameters
    /// get their default values. This is used by the hot-reload pipeline
    /// to update parameter definitions without restarting the server.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter replacement fails (e.g., unrecoverable
    /// lock poisoning).
    pub fn replace_parameters(&self, new_params: Vec<ParameterInfo>) -> Result<(), String> {
        self.inner.replace_parameters(new_params)
    }
}

impl ParameterHost for DevServerHost {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        self.inner.get_parameter(id)
    }

    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
        let result = self.inner.set_parameter(id, value);

        // Forward to atomic bridge for audio-thread access (lock-free)
        #[cfg(feature = "audio")]
        if result.is_ok() && let Some(ref bridge) = self.param_bridge {
            bridge.write(id, value);
        }

        result
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        self.inner.get_all_parameters()
    }

    fn get_meter_frame(&self) -> Option<MeterFrame> {
        // Meters are now provided externally via the audio server's meter
        // channel → WebSocket broadcast. No synthetic generation.
        None
    }

    fn request_resize(&self, _width: u32, _height: u32) -> bool {
        self.inner.request_resize(_width, _height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wavecraft_protocol::ParameterType;

    fn test_params() -> Vec<ParameterInfo> {
        vec![
            ParameterInfo {
                id: "gain".to_string(),
                name: "Gain".to_string(),
                param_type: ParameterType::Float,
                value: 0.5,
                default: 0.5,
                unit: Some("dB".to_string()),
                group: Some("Input".to_string()),
            },
            ParameterInfo {
                id: "mix".to_string(),
                name: "Mix".to_string(),
                param_type: ParameterType::Float,
                value: 1.0,
                default: 1.0,
                unit: Some("%".to_string()),
                group: None,
            },
        ]
    }

    #[test]
    fn test_get_parameter() {
        let host = DevServerHost::new(test_params());

        let param = host.get_parameter("gain").expect("should find gain");
        assert_eq!(param.id, "gain");
        assert_eq!(param.name, "Gain");
        assert!((param.value - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_get_parameter_not_found() {
        let host = DevServerHost::new(test_params());
        assert!(host.get_parameter("nonexistent").is_none());
    }

    #[test]
    fn test_set_parameter() {
        let host = DevServerHost::new(test_params());

        host.set_parameter("gain", 0.75).expect("should set gain");

        let param = host.get_parameter("gain").expect("should find gain");
        assert!((param.value - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_parameter_invalid_id() {
        let host = DevServerHost::new(test_params());
        let result = host.set_parameter("invalid", 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_parameter_out_of_range() {
        let host = DevServerHost::new(test_params());

        let result = host.set_parameter("gain", 1.5);
        assert!(result.is_err());

        let result = host.set_parameter("gain", -0.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_parameters() {
        let host = DevServerHost::new(test_params());

        let params = host.get_all_parameters();
        assert_eq!(params.len(), 2);
        assert!(params.iter().any(|p| p.id == "gain"));
        assert!(params.iter().any(|p| p.id == "mix"));
    }

    #[test]
    fn test_get_meter_frame() {
        let host = DevServerHost::new(test_params());
        // Meters are now provided externally — no synthetic generation
        assert!(host.get_meter_frame().is_none());
    }
}
