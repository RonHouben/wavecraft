//! Application state with atomic parameter storage
//!
//! This simulates plugin parameter state using thread-safe atomics,
//! demonstrating the pattern that will be used in the actual plugin.

use atomic_float::AtomicF32;
use bridge::{BridgeError, ParameterHost, ParameterInfo, ParameterType};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Application state with simulated plugin parameters
#[derive(Clone)]
pub struct AppState {
    // Parameters (simulating a simple gain/mix plugin)
    gain: Arc<AtomicF32>,
    bypass: Arc<AtomicBool>,
    mix: Arc<AtomicF32>,
}

impl AppState {
    /// Create new application state with default values
    pub fn new() -> Self {
        Self {
            gain: Arc::new(AtomicF32::new(0.7)), // Default gain: 0.7 normalized (-2.6 dB)
            bypass: Arc::new(AtomicBool::new(false)),
            mix: Arc::new(AtomicF32::new(1.0)), // Default mix: 100% wet
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
        match id {
            "gain" => Some(ParameterInfo {
                id: "gain".to_string(),
                name: "Gain".to_string(),
                param_type: ParameterType::Float,
                value: self.gain.load(Ordering::Relaxed),
                default: 0.7,
                unit: Some("dB".to_string()),
            }),
            "bypass" => Some(ParameterInfo {
                id: "bypass".to_string(),
                name: "Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: if self.bypass.load(Ordering::Relaxed) {
                    1.0
                } else {
                    0.0
                },
                default: 0.0,
                unit: None,
            }),
            "mix" => Some(ParameterInfo {
                id: "mix".to_string(),
                name: "Mix".to_string(),
                param_type: ParameterType::Float,
                value: self.mix.load(Ordering::Relaxed),
                default: 1.0,
                unit: Some("%".to_string()),
            }),
            _ => None,
        }
    }

    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
        // Validate range
        if !(0.0..=1.0).contains(&value) {
            return Err(BridgeError::ParameterOutOfRange {
                id: id.to_string(),
                value,
            });
        }

        match id {
            "gain" => {
                self.gain.store(value, Ordering::Relaxed);
                Ok(())
            }
            "bypass" => {
                self.bypass.store(value >= 0.5, Ordering::Relaxed);
                Ok(())
            }
            "mix" => {
                self.mix.store(value, Ordering::Relaxed);
                Ok(())
            }
            _ => Err(BridgeError::ParameterNotFound(id.to_string())),
        }
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        vec![
            self.get_parameter("gain").unwrap(),
            self.get_parameter("bypass").unwrap(),
            self.get_parameter("mix").unwrap(),
        ]
    }

    fn get_meter_frame(&self) -> Option<vstkit_protocol::MeterFrame> {
        // Desktop POC doesn't have metering yet
        None
    }

    fn request_resize(&self, _width: u32, _height: u32) -> bool {
        // Desktop POC doesn't support dynamic resizing
        // (The desktop app has fixed window size)
        false
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
