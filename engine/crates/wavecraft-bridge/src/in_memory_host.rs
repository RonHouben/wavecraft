//! In-memory ParameterHost implementation for dev tools and tests.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::{BridgeError, ParameterHost};
use wavecraft_protocol::{MeterFrame, ParameterInfo};

/// Provides metering data for an in-memory host.
pub trait MeterProvider: Send + Sync {
    /// Return the latest meter frame, if available.
    fn get_meter_frame(&self) -> Option<MeterFrame>;
}

/// In-memory host for storing parameter values and optional meter data.
///
/// This is intended for development tools (like the CLI dev server) and tests.
pub struct InMemoryParameterHost {
    parameters: RwLock<Vec<ParameterInfo>>,
    values: RwLock<HashMap<String, f32>>,
    meter_provider: Option<Arc<dyn MeterProvider>>,
}

impl InMemoryParameterHost {
    /// Create a new in-memory host with the given parameter metadata.
    pub fn new(parameters: Vec<ParameterInfo>) -> Self {
        let values = parameters
            .iter()
            .map(|p| (p.id.clone(), p.default))
            .collect();

        Self {
            parameters: RwLock::new(parameters),
            values: RwLock::new(values),
            meter_provider: None,
        }
    }

    /// Create a new in-memory host with a meter provider.
    pub fn with_meter_provider(
        parameters: Vec<ParameterInfo>,
        meter_provider: Arc<dyn MeterProvider>,
    ) -> Self {
        let mut host = Self::new(parameters);
        host.meter_provider = Some(meter_provider);
        host
    }

    /// Replace all parameters with new metadata from a fresh build.
    ///
    /// This method is used during hot-reload to update parameter definitions
    /// while preserving existing parameter values where possible. Parameters
    /// with matching IDs retain their current values; new parameters get
    /// their default values; removed parameters are dropped.
    ///
    /// # Thread Safety
    ///
    /// This method acquires write locks on both the parameters and values maps.
    /// If a lock is poisoned (from a previous panic), it recovers gracefully
    /// by clearing the poisoned lock and continuing.
    ///
    /// # Errors
    ///
    /// Returns an error if both lock recovery attempts fail.
    pub fn replace_parameters(&self, new_params: Vec<ParameterInfo>) -> Result<(), String> {
        // Acquire values lock with poison recovery
        let mut values = match self.values.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("⚠ Recovering from poisoned values lock");
                poisoned.into_inner()
            }
        };

        // Build new values map, preserving existing values where IDs match
        let mut new_values = HashMap::new();
        for param in &new_params {
            let value = values.get(&param.id).copied().unwrap_or(param.default);
            new_values.insert(param.id.clone(), value);
        }

        *values = new_values;
        drop(values); // Release values lock before acquiring parameters lock

        // Acquire parameters lock with poison recovery
        let mut params = match self.parameters.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("⚠ Recovering from poisoned parameters lock");
                poisoned.into_inner()
            }
        };

        *params = new_params;
        Ok(())
    }

    fn current_value(&self, id: &str, default: f32) -> f32 {
        self.values
            .read()
            .ok()
            .and_then(|values| values.get(id).copied())
            .unwrap_or(default)
    }
}

impl ParameterHost for InMemoryParameterHost {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        let parameters = self.parameters.read().ok()?;
        let param = parameters.iter().find(|p| p.id == id)?;

        Some(ParameterInfo {
            id: param.id.clone(),
            name: param.name.clone(),
            param_type: param.param_type,
            value: self.current_value(&param.id, param.default),
            default: param.default,
            unit: param.unit.clone(),
            group: param.group.clone(),
        })
    }

    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
        let parameters = self.parameters.read().ok();
        let param_exists = parameters
            .as_ref()
            .map(|p| p.iter().any(|param| param.id == id))
            .unwrap_or(false);
        
        if !param_exists {
            return Err(BridgeError::ParameterNotFound(id.to_string()));
        }

        if !(0.0..=1.0).contains(&value) {
            return Err(BridgeError::ParameterOutOfRange {
                id: id.to_string(),
                value,
            });
        }

        if let Ok(mut values) = self.values.write() {
            values.insert(id.to_string(), value);
        }

        Ok(())
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        let parameters = match self.parameters.read() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(), // Return empty on poisoned lock
        };
        
        parameters
            .iter()
            .map(|param| ParameterInfo {
                id: param.id.clone(),
                name: param.name.clone(),
                param_type: param.param_type,
                value: self.current_value(&param.id, param.default),
                default: param.default,
                unit: param.unit.clone(),
                group: param.group.clone(),
            })
            .collect()
    }

    fn get_meter_frame(&self) -> Option<MeterFrame> {
        self.meter_provider
            .as_ref()
            .and_then(|provider| provider.get_meter_frame())
    }

    fn request_resize(&self, _width: u32, _height: u32) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wavecraft_protocol::ParameterType;

    struct StaticMeterProvider {
        frame: MeterFrame,
    }

    impl MeterProvider for StaticMeterProvider {
        fn get_meter_frame(&self) -> Option<MeterFrame> {
            Some(self.frame)
        }
    }

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
        let host = InMemoryParameterHost::new(test_params());

        let param = host.get_parameter("gain").expect("should find gain");
        assert_eq!(param.id, "gain");
        assert_eq!(param.name, "Gain");
        assert!((param.value - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_parameter() {
        let host = InMemoryParameterHost::new(test_params());

        host.set_parameter("gain", 0.75).expect("should set gain");

        let param = host.get_parameter("gain").expect("should find gain");
        assert!((param.value - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_parameter_out_of_range() {
        let host = InMemoryParameterHost::new(test_params());

        let result = host.set_parameter("gain", 1.5);
        assert!(result.is_err());

        let result = host.set_parameter("gain", -0.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_parameters() {
        let host = InMemoryParameterHost::new(test_params());

        let params = host.get_all_parameters();
        assert_eq!(params.len(), 2);
        assert!(params.iter().any(|p| p.id == "gain"));
        assert!(params.iter().any(|p| p.id == "mix"));
    }

    #[test]
    fn test_get_meter_frame() {
        let frame = MeterFrame {
            peak_l: 0.7,
            rms_l: 0.5,
            peak_r: 0.6,
            rms_r: 0.4,
            timestamp: 0,
        };
        let provider = Arc::new(StaticMeterProvider { frame });
        let host = InMemoryParameterHost::with_meter_provider(test_params(), provider);

        let read = host.get_meter_frame().expect("should have meter frame");
        assert!((read.peak_l - 0.7).abs() < f32::EPSILON);
        assert!((read.rms_r - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_replace_parameters_preserves_values() {
        let host = InMemoryParameterHost::new(test_params());

        // Set custom values
        host.set_parameter("gain", 0.75).expect("should set gain");
        host.set_parameter("mix", 0.5).expect("should set mix");

        // Add a new parameter
        let new_params = vec![
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
            ParameterInfo {
                id: "freq".to_string(),
                name: "Frequency".to_string(),
                param_type: ParameterType::Float,
                value: 440.0,
                default: 440.0,
                unit: Some("Hz".to_string()),
                group: None,
            },
        ];

        host.replace_parameters(new_params).expect("should replace parameters");

        // Existing parameters should preserve their values
        let gain = host.get_parameter("gain").expect("should find gain");
        assert!((gain.value - 0.75).abs() < f32::EPSILON);

        let mix = host.get_parameter("mix").expect("should find mix");
        assert!((mix.value - 0.5).abs() < f32::EPSILON);

        // New parameter should have default value
        let freq = host.get_parameter("freq").expect("should find freq");
        assert!((freq.value - 440.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_replace_parameters_removes_old() {
        let host = InMemoryParameterHost::new(test_params());

        // Replace with fewer parameters
        let new_params = vec![ParameterInfo {
            id: "gain".to_string(),
            name: "Gain".to_string(),
            param_type: ParameterType::Float,
            value: 0.5,
            default: 0.5,
            unit: Some("dB".to_string()),
            group: Some("Input".to_string()),
        }];

        host.replace_parameters(new_params).expect("should replace parameters");

        // Old parameter should be gone
        assert!(host.get_parameter("mix").is_none());

        // Kept parameter should still be accessible
        assert!(host.get_parameter("gain").is_some());
    }
}
