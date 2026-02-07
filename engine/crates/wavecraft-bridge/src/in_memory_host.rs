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
    parameters: Vec<ParameterInfo>,
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
            parameters,
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
        let param = self.parameters.iter().find(|p| p.id == id)?;

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
        if !self.parameters.iter().any(|p| p.id == id) {
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
        self.parameters
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
}
