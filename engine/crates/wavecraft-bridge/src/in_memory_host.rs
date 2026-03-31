//! In-memory ParameterHost implementation for dev tools and tests.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::{BridgeError, ParameterHost};
use wavecraft_protocol::{
    AudioRuntimeStatus, GetHardwareInputSelectionResult, GetInputSourceResult,
    HardwareInputChannelOption, HardwareInputDeviceOption, InputSourceKind, InputSourceOption,
    MeterFrame, OscilloscopeFrame, ParameterInfo, SetHardwareInputSelectionParams, SignalChainSlot,
};

/// Provides metering data for an in-memory host.
pub trait MeterProvider: Send + Sync {
    /// Return the latest meter frame, if available.
    fn get_meter_frame(&self) -> Option<MeterFrame>;
}

/// Provides oscilloscope frame data for an in-memory host.
pub trait OscilloscopeProvider: Send + Sync {
    /// Return the latest oscilloscope frame, if available.
    fn get_oscilloscope_frame(&self) -> Option<OscilloscopeFrame>;
}

/// In-memory host for storing parameter values and optional meter data.
///
/// This is intended for development tools (like the CLI dev server) and tests.
pub struct InMemoryParameterHost {
    parameters: RwLock<Vec<ParameterInfo>>,
    values: RwLock<HashMap<String, f32>>,
    meter_provider: Option<Arc<dyn MeterProvider>>,
    oscilloscope_provider: Option<Arc<dyn OscilloscopeProvider>>,
    /// Active signal chain order as a list of slots.
    signal_chain_order: RwLock<Vec<SignalChainSlot>>,
    input_source: RwLock<InputSourceKind>,
    selected_hardware_input_device_id: RwLock<Option<String>>,
    selected_hardware_input_channel_id: RwLock<Option<String>>,
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
            oscilloscope_provider: None,
            signal_chain_order: RwLock::new(Vec::new()),
            input_source: RwLock::new(InputSourceKind::HardwareInput),
            selected_hardware_input_device_id: RwLock::new(Some(
                DEFAULT_HARDWARE_INPUT_DEVICE_ID.to_string(),
            )),
            selected_hardware_input_channel_id: RwLock::new(Some(
                DEFAULT_HARDWARE_INPUT_CHANNEL_ID.to_string(),
            )),
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

    /// Create a new in-memory host with an oscilloscope provider.
    pub fn with_oscilloscope_provider(
        parameters: Vec<ParameterInfo>,
        oscilloscope_provider: Arc<dyn OscilloscopeProvider>,
    ) -> Self {
        let mut host = Self::new(parameters);
        host.oscilloscope_provider = Some(oscilloscope_provider);
        host
    }

    /// Create a new in-memory host with both meter and oscilloscope providers.
    pub fn with_providers(
        parameters: Vec<ParameterInfo>,
        meter_provider: Option<Arc<dyn MeterProvider>>,
        oscilloscope_provider: Option<Arc<dyn OscilloscopeProvider>>,
    ) -> Self {
        let mut host = Self::new(parameters);
        host.meter_provider = meter_provider;
        host.oscilloscope_provider = oscilloscope_provider;
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

    fn materialize_parameter(&self, param: &ParameterInfo) -> ParameterInfo {
        ParameterInfo {
            id: param.id.clone(),
            name: param.name.clone(),
            param_type: param.param_type,
            value: self.current_value(&param.id, param.default),
            default: param.default,
            min: param.min,
            max: param.max,
            unit: param.unit.clone(),
            group: param.group.clone(),
            variants: param.variants.clone(),
        }
    }
}

impl ParameterHost for InMemoryParameterHost {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        let parameters = self.parameters.read().ok()?;
        let param = parameters.iter().find(|p| p.id == id)?;

        Some(self.materialize_parameter(param))
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

        let Some(param) = parameters
            .as_ref()
            .and_then(|p| p.iter().find(|param| param.id == id))
        else {
            return Err(BridgeError::ParameterNotFound(id.to_string()));
        };

        if !(param.min..=param.max).contains(&value) {
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
            .map(|param| self.materialize_parameter(param))
            .collect()
    }

    fn get_meter_frame(&self) -> Option<MeterFrame> {
        self.meter_provider
            .as_ref()
            .and_then(|provider| provider.get_meter_frame())
    }

    fn get_oscilloscope_frame(&self) -> Option<OscilloscopeFrame> {
        self.oscilloscope_provider
            .as_ref()
            .and_then(|provider| provider.get_oscilloscope_frame())
    }

    fn request_resize(&self, _width: u32, _height: u32) -> bool {
        false
    }

    fn get_audio_status(&self) -> Option<AudioRuntimeStatus> {
        None
    }

    fn get_input_source(&self) -> Option<GetInputSourceResult> {
        let selected = self
            .input_source
            .read()
            .map(|guard| *guard)
            .unwrap_or(InputSourceKind::HardwareInput);

        Some(GetInputSourceResult {
            selected,
            available: default_input_source_options(),
        })
    }

    fn set_input_source(&self, source: InputSourceKind) -> Result<(), BridgeError> {
        *self.input_source.write().unwrap() = source;
        Ok(())
    }

    fn get_hardware_input_selection(&self) -> Option<GetHardwareInputSelectionResult> {
        let available_devices = default_hardware_input_device_options();
        let available_channels = default_hardware_input_channel_options();
        let selected_device_id = self
            .selected_hardware_input_device_id
            .read()
            .ok()
            .and_then(|guard| guard.clone())
            .filter(|selected| {
                available_devices
                    .iter()
                    .any(|device| &device.id == selected)
            })
            .or_else(|| Some(DEFAULT_HARDWARE_INPUT_DEVICE_ID.to_string()));
        let selected_channel_id = self
            .selected_hardware_input_channel_id
            .read()
            .ok()
            .and_then(|guard| guard.clone())
            .filter(|selected| {
                available_channels
                    .iter()
                    .any(|channel| &channel.id == selected)
            })
            .or_else(|| Some(DEFAULT_HARDWARE_INPUT_CHANNEL_ID.to_string()));

        Some(GetHardwareInputSelectionResult {
            selected_device_id,
            available_devices,
            selected_channel_id,
            available_channels,
        })
    }

    fn set_hardware_input_selection(
        &self,
        selection: SetHardwareInputSelectionParams,
    ) -> Result<(), BridgeError> {
        let available_devices = default_hardware_input_device_options();
        let available_channels = default_hardware_input_channel_options();

        if let Some(selected_device_id) = selection.selected_device_id {
            if !available_devices
                .iter()
                .any(|device| device.id == selected_device_id)
            {
                return Err(BridgeError::InvalidParams {
                    method: "setHardwareInputSelection".to_string(),
                    reason: format!("Unknown hardware input device: {}", selected_device_id),
                });
            }

            *self.selected_hardware_input_device_id.write().unwrap() = Some(selected_device_id);
        }

        if let Some(selected_channel_id) = selection.selected_channel_id {
            if !available_channels
                .iter()
                .any(|channel| channel.id == selected_channel_id)
            {
                return Err(BridgeError::InvalidParams {
                    method: "setHardwareInputSelection".to_string(),
                    reason: format!("Unknown hardware input routing: {}", selected_channel_id),
                });
            }

            *self.selected_hardware_input_channel_id.write().unwrap() = Some(selected_channel_id);
        }

        Ok(())
    }

    fn get_signal_chain_order(&self) -> Vec<SignalChainSlot> {
        self.signal_chain_order
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    fn set_signal_chain_order(&self, order: Vec<SignalChainSlot>) -> Result<(), BridgeError> {
        // Validate: no empty IDs, no duplicate IDs.
        let mut seen = std::collections::HashSet::new();
        for slot in &order {
            if slot.id.is_empty() {
                return Err(BridgeError::InvalidSignalChainOrder {
                    reason: "slot ID must not be empty".to_string(),
                });
            }
            if !seen.insert(slot.id.as_str()) {
                return Err(BridgeError::InvalidSignalChainOrder {
                    reason: format!("duplicate slot ID: {}", slot.id),
                });
            }
        }
        *self.signal_chain_order.write().unwrap() = order;
        Ok(())
    }
}

fn default_input_source_options() -> Vec<InputSourceOption> {
    vec![
        InputSourceOption {
            id: InputSourceKind::HardwareInput,
            label: "Soundcard input".to_string(),
            description: Some(
                "Use the active hardware input routed into the dev server".to_string(),
            ),
        },
        InputSourceOption {
            id: InputSourceKind::TestTone,
            label: "Test tone".to_string(),
            description: Some("Use the TestTone processor signal as the chain input".to_string()),
        },
    ]
}

const DEFAULT_HARDWARE_INPUT_DEVICE_ID: &str = "default-hardware-input";
const DEFAULT_HARDWARE_INPUT_CHANNEL_ID: &str = "stereo:0:1";

fn default_hardware_input_device_options() -> Vec<HardwareInputDeviceOption> {
    vec![HardwareInputDeviceOption {
        id: DEFAULT_HARDWARE_INPUT_DEVICE_ID.to_string(),
        label: "Default soundcard input".to_string(),
        channel_count: 2,
        description: Some("Fallback device used by bridge tests and non-audio hosts".to_string()),
    }]
}

fn default_hardware_input_channel_options() -> Vec<HardwareInputChannelOption> {
    vec![
        HardwareInputChannelOption {
            id: DEFAULT_HARDWARE_INPUT_CHANNEL_ID.to_string(),
            label: "Inputs 1 + 2 (stereo)".to_string(),
            description: Some("Route the first stereo pair into the signal chain".to_string()),
        },
        HardwareInputChannelOption {
            id: "mono:0".to_string(),
            label: "Input 1 (mono → dual mono)".to_string(),
            description: None,
        },
        HardwareInputChannelOption {
            id: "mono:1".to_string(),
            label: "Input 2 (mono → dual mono)".to_string(),
            description: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use wavecraft_protocol::{ParameterType, SlotType};

    struct StaticMeterProvider {
        frame: MeterFrame,
    }

    struct StaticOscilloscopeProvider {
        frame: OscilloscopeFrame,
    }

    impl MeterProvider for StaticMeterProvider {
        fn get_meter_frame(&self) -> Option<MeterFrame> {
            Some(self.frame)
        }
    }

    impl OscilloscopeProvider for StaticOscilloscopeProvider {
        fn get_oscilloscope_frame(&self) -> Option<OscilloscopeFrame> {
            Some(self.frame.clone())
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
                min: 0.0,
                max: 1.0,
                unit: Some("dB".to_string()),
                group: Some("Input".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "mix".to_string(),
                name: "Mix".to_string(),
                param_type: ParameterType::Float,
                value: 1.0,
                default: 1.0,
                min: 0.0,
                max: 1.0,
                unit: Some("%".to_string()),
                group: None,
                variants: None,
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
    fn test_get_oscilloscope_frame() {
        let frame = OscilloscopeFrame {
            points_l: vec![0.1; 1024],
            points_r: vec![0.2; 1024],
            sample_rate: 48_000.0,
            timestamp: 99,
            no_signal: false,
            trigger_mode: wavecraft_protocol::OscilloscopeTriggerMode::RisingZeroCrossing,
        };
        let provider = Arc::new(StaticOscilloscopeProvider { frame });
        let host = InMemoryParameterHost::with_oscilloscope_provider(test_params(), provider);

        let read = host
            .get_oscilloscope_frame()
            .expect("should have oscilloscope frame");
        assert_eq!(read.points_l.len(), 1024);
        assert_eq!(read.points_r.len(), 1024);
        assert_eq!(read.timestamp, 99);
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
                min: 0.0,
                max: 1.0,
                unit: Some("dB".to_string()),
                group: Some("Input".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "mix".to_string(),
                name: "Mix".to_string(),
                param_type: ParameterType::Float,
                value: 1.0,
                default: 1.0,
                min: 0.0,
                max: 1.0,
                unit: Some("%".to_string()),
                group: None,
                variants: None,
            },
            ParameterInfo {
                id: "freq".to_string(),
                name: "Frequency".to_string(),
                param_type: ParameterType::Float,
                value: 440.0,
                default: 440.0,
                min: 20.0,
                max: 20_000.0,
                unit: Some("Hz".to_string()),
                group: None,
                variants: None,
            },
        ];

        host.replace_parameters(new_params)
            .expect("should replace parameters");

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
            min: 0.0,
            max: 1.0,
            unit: Some("dB".to_string()),
            group: Some("Input".to_string()),
            variants: None,
        }];

        host.replace_parameters(new_params)
            .expect("should replace parameters");

        // Old parameter should be gone
        assert!(host.get_parameter("mix").is_none());

        // Kept parameter should still be accessible
        assert!(host.get_parameter("gain").is_some());
    }

    #[test]
    fn test_set_parameter_uses_declared_range_not_normalized_range() {
        let host = InMemoryParameterHost::new(vec![ParameterInfo {
            id: "test_tone_frequency".to_string(),
            name: "Frequency".to_string(),
            param_type: ParameterType::Float,
            value: 440.0,
            default: 440.0,
            min: 20.0,
            max: 20_000.0,
            unit: Some("Hz".to_string()),
            group: Some("Test Tone".to_string()),
            variants: None,
        }]);

        host.set_parameter("test_tone_frequency", 2_000.0)
            .expect("frequency in declared range should be accepted");

        let freq = host
            .get_parameter("test_tone_frequency")
            .expect("frequency should exist");
        assert!((freq.value - 2_000.0).abs() < f32::EPSILON);

        let too_low = host.set_parameter("test_tone_frequency", 10.0);
        assert!(too_low.is_err(), "value below min should be rejected");

        let too_high = host.set_parameter("test_tone_frequency", 30_000.0);
        assert!(too_high.is_err(), "value above max should be rejected");
    }

    // ── set_signal_chain_order validation ────────────────────────────────────────

    #[test]
    fn test_set_signal_chain_order_empty_vec_is_accepted() {
        let host = InMemoryParameterHost::new(vec![]);
        assert!(
            host.set_signal_chain_order(vec![]).is_ok(),
            "empty order should be accepted"
        );
        assert!(host.get_signal_chain_order().is_empty());
    }

    #[test]
    fn test_set_signal_chain_order_valid_slots() {
        let host = InMemoryParameterHost::new(vec![]);
        let order = vec![
            SignalChainSlot {
                id: "proc_0".to_string(),
                slot_type: SlotType::Processor,
            },
            SignalChainSlot {
                id: "tap_0".to_string(),
                slot_type: SlotType::Tap,
            },
        ];
        assert!(
            host.set_signal_chain_order(order.clone()).is_ok(),
            "valid slots should succeed"
        );
        assert_eq!(host.get_signal_chain_order(), order);
    }

    #[test]
    fn test_set_signal_chain_order_empty_id_rejected() {
        let host = InMemoryParameterHost::new(vec![]);
        let order = vec![SignalChainSlot {
            id: "".to_string(),
            slot_type: SlotType::Processor,
        }];
        let result = host.set_signal_chain_order(order);
        assert!(result.is_err(), "empty ID should be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("empty"), "error should mention empty: {msg}");
    }

    #[test]
    fn test_set_signal_chain_order_duplicate_id_rejected() {
        let host = InMemoryParameterHost::new(vec![]);
        let order = vec![
            SignalChainSlot {
                id: "slot_a".to_string(),
                slot_type: SlotType::Processor,
            },
            SignalChainSlot {
                id: "slot_a".to_string(),
                slot_type: SlotType::Tap,
            },
        ];
        let result = host.set_signal_chain_order(order);
        assert!(result.is_err(), "duplicate ID should be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("duplicate") || msg.contains("Duplicate"),
            "error should describe duplicate: {msg}"
        );
    }

    #[test]
    fn test_set_signal_chain_order_mixed_slot_types() {
        let host = InMemoryParameterHost::new(vec![]);
        let order = vec![
            SignalChainSlot {
                id: "p0".to_string(),
                slot_type: SlotType::Processor,
            },
            SignalChainSlot {
                id: "t0".to_string(),
                slot_type: SlotType::Tap,
            },
            SignalChainSlot {
                id: "p1".to_string(),
                slot_type: SlotType::Processor,
            },
        ];
        assert!(host.set_signal_chain_order(order.clone()).is_ok());
        assert_eq!(host.get_signal_chain_order(), order);
    }

    #[test]
    fn test_input_source_defaults_to_hardware_input() {
        let host = InMemoryParameterHost::new(vec![]);
        let input_source = host.get_input_source().expect("input source should exist");
        assert_eq!(input_source.selected, InputSourceKind::HardwareInput);
        assert_eq!(input_source.available.len(), 2);
    }

    #[test]
    fn test_set_input_source_updates_selection() {
        let host = InMemoryParameterHost::new(vec![]);
        host.set_input_source(InputSourceKind::TestTone)
            .expect("should set input source");

        let input_source = host.get_input_source().expect("input source should exist");
        assert_eq!(input_source.selected, InputSourceKind::TestTone);
    }
}
