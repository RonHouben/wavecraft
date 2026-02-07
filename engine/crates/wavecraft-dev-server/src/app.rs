//! Application state with atomic parameter storage
//!
//! This simulates plugin parameter state using thread-safe atomics,
//! demonstrating the pattern that will be used in the actual plugin.

use std::sync::Arc;
use wavecraft_bridge::{
    BridgeError, InMemoryParameterHost, ParameterHost, ParameterInfo, ParameterType,
};

/// Application state with simulated plugin parameters.
#[derive(Clone)]
pub struct AppState {
    host: Arc<InMemoryParameterHost>,
}

impl AppState {
    /// Create new application state with default values
    pub fn new() -> Self {
        let parameters = vec![
            ParameterInfo {
                id: "gain".to_string(),
                name: "Gain".to_string(),
                param_type: ParameterType::Float,
                value: 0.7,
                default: 0.7,
                unit: Some("dB".to_string()),
                group: None,
            },
            ParameterInfo {
                id: "bypass".to_string(),
                name: "Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: 0.0,
                default: 0.0,
                unit: None,
                group: None,
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
        ];

        Self {
            host: Arc::new(InMemoryParameterHost::new(parameters)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterHost for AppState {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        self.host.get_parameter(id)
    }

    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
        self.host.set_parameter(id, value)
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        self.host.get_all_parameters()
    }

    fn get_meter_frame(&self) -> Option<wavecraft_protocol::MeterFrame> {
        self.host.get_meter_frame()
    }

    fn request_resize(&self, _width: u32, _height: u32) -> bool {
        self.host.request_resize(_width, _height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let state = AppState::new();

        let gain = state.get_parameter("gain").unwrap();
        assert_eq!(gain.value, 0.7);

        let bypass = state.get_parameter("bypass").unwrap();
        assert_eq!(bypass.value, 0.0);

        let mix = state.get_parameter("mix").unwrap();
        assert_eq!(mix.value, 1.0);
    }

    #[test]
    fn test_set_parameter() {
        let state = AppState::new();

        state.set_parameter("gain", 0.5).unwrap();
        let gain = state.get_parameter("gain").unwrap();
        assert_eq!(gain.value, 0.5);
    }

    #[test]
    fn test_set_parameter_out_of_range() {
        let state = AppState::new();

        let result = state.set_parameter("gain", 1.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_parameters() {
        let state = AppState::new();

        let params = state.get_all_parameters();
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_unknown_parameter() {
        let state = AppState::new();

        assert!(state.get_parameter("unknown").is_none());
        assert!(state.set_parameter("unknown", 0.5).is_err());
    }
}
