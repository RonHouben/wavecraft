//! Development server host implementing ParameterHost trait
//!
//! This module provides a ParameterHost implementation for the embedded
//! development server. It stores parameter values in memory and generates
//! synthetic metering data for UI testing.

use std::sync::{Arc, RwLock};
use wavecraft_bridge::{BridgeError, InMemoryParameterHost, MeterProvider, ParameterHost};
use wavecraft_metering::dev::MeterGenerator;
use wavecraft_protocol::{MeterFrame, ParameterInfo};

struct MeterGeneratorProvider {
    generator: Arc<RwLock<MeterGenerator>>,
}

impl MeterProvider for MeterGeneratorProvider {
    fn get_meter_frame(&self) -> Option<MeterFrame> {
        self.generator.read().ok().map(|gen| gen.frame())
    }
}

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
    inner: InMemoryParameterHost,
    meter_generator: Arc<RwLock<MeterGenerator>>,
}

impl DevServerHost {
    /// Create a new dev server host with parameter metadata
    ///
    /// # Arguments
    ///
    /// * `parameters` - Parameter metadata loaded from the plugin FFI
    pub fn new(parameters: Vec<ParameterInfo>) -> Self {
        let meter_generator = Arc::new(RwLock::new(MeterGenerator::new()));
        let provider = Arc::new(MeterGeneratorProvider {
            generator: Arc::clone(&meter_generator),
        });
        let inner = InMemoryParameterHost::with_meter_provider(parameters, provider);

        Self {
            inner,
            meter_generator,
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
        self.inner.get_parameter(id)
    }

    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
        self.inner.set_parameter(id, value)
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        self.inner.get_all_parameters()
    }

    fn get_meter_frame(&self) -> Option<MeterFrame> {
        self.inner.get_meter_frame()
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
        let frame = host.get_meter_frame().expect("should have meter frame");

        // Synthetic meters should have reasonable values
        assert!(frame.peak_l >= 0.0);
        assert!(frame.peak_r >= 0.0);
    }
}
