//! Development server host implementing ParameterHost trait
//!
//! This module provides a ParameterHost implementation for the embedded
//! development server. It stores parameter values in memory and generates
//! synthetic metering data for UI testing.

use std::collections::HashMap;
use std::sync::RwLock;
use wavecraft_bridge::{BridgeError, ParameterHost};
use wavecraft_protocol::{MeterFrame, ParameterInfo};

use super::MeterGenerator;

/// Development server host for browser-based UI testing
///
/// This implementation stores parameter values locally and provides
/// synthetic metering data. It's designed for rapid UI iteration
/// without requiring a full audio engine.
///
/// # Thread Safety
///
/// All state is protected by RwLock for concurrent access from
/// the IPC handler and any background tasks.
pub struct DevServerHost {
    /// Parameter metadata loaded from the plugin
    parameters: Vec<ParameterInfo>,
    /// Current parameter values (normalized 0.0-1.0)
    values: RwLock<HashMap<String, f32>>,
    /// Synthetic meter generator
    meter_generator: RwLock<MeterGenerator>,
}

impl DevServerHost {
    /// Create a new dev server host with parameter metadata
    ///
    /// # Arguments
    ///
    /// * `parameters` - Parameter metadata loaded from the plugin FFI
    pub fn new(parameters: Vec<ParameterInfo>) -> Self {
        // Initialize values map with defaults
        let values: HashMap<String, f32> = parameters
            .iter()
            .map(|p| (p.id.clone(), p.default))
            .collect();

        Self {
            parameters,
            values: RwLock::new(values),
            meter_generator: RwLock::new(MeterGenerator::new()),
        }
    }

    /// Update meter animation state
    ///
    /// Call this periodically (e.g., 60 Hz) to animate the synthetic meters.
    #[allow(dead_code)] // Reserved for future meter animation feature
    pub fn tick_meters(&self) {
        if let Ok(mut gen) = self.meter_generator.write() {
            gen.tick();
        }
    }
}

impl ParameterHost for DevServerHost {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        // Find the parameter metadata
        let param = self.parameters.iter().find(|p| p.id == id)?;

        // Get current value
        let value = self
            .values
            .read()
            .ok()?
            .get(id)
            .copied()
            .unwrap_or(param.default);

        // Return updated parameter info with current value
        Some(ParameterInfo {
            id: param.id.clone(),
            name: param.name.clone(),
            param_type: param.param_type,
            value,
            default: param.default,
            unit: param.unit.clone(),
            group: param.group.clone(),
        })
    }

    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
        // Validate parameter exists
        if !self.parameters.iter().any(|p| p.id == id) {
            return Err(BridgeError::ParameterNotFound(id.to_string()));
        }

        // Validate value range (normalized 0.0-1.0)
        if !(0.0..=1.0).contains(&value) {
            return Err(BridgeError::ParameterOutOfRange {
                id: id.to_string(),
                value,
            });
        }

        // Update value
        if let Ok(mut values) = self.values.write() {
            values.insert(id.to_string(), value);
        }

        Ok(())
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        let values = self.values.read().ok();

        self.parameters
            .iter()
            .map(|p| {
                let value = values
                    .as_ref()
                    .and_then(|v| v.get(&p.id).copied())
                    .unwrap_or(p.default);

                ParameterInfo {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    param_type: p.param_type,
                    value,
                    default: p.default,
                    unit: p.unit.clone(),
                    group: p.group.clone(),
                }
            })
            .collect()
    }

    fn get_meter_frame(&self) -> Option<MeterFrame> {
        self.meter_generator.read().ok().map(|gen| gen.frame())
    }

    fn request_resize(&self, _width: u32, _height: u32) -> bool {
        // Dev server doesn't support window resize
        false
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
        let frame = host.get_meter_frame().expect("should have meter frame");

        // Synthetic meters should have reasonable values
        assert!(frame.left_peak >= 0.0);
        assert!(frame.right_peak >= 0.0);
    }
}
