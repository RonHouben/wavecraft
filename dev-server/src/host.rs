//! Development server host implementing ParameterHost trait
//!
//! This module provides a ParameterHost implementation for the embedded
//! development server. It stores parameter values in memory and forwards
//! parameter changes to an optional AtomicParameterBridge for lock-free
//! audio-thread access.

#[cfg(feature = "audio")]
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use wavecraft_bridge::{BridgeError, InMemoryParameterHost, ParameterHost};
use wavecraft_protocol::{
    AudioRuntimePhase, AudioRuntimeStatus, GetHardwareInputSelectionResult, GetInputSourceResult,
    InputSourceKind, MeterFrame, MeterUpdateNotification, OscilloscopeFrame, ParameterInfo,
    SetHardwareInputSelectionParams, SignalChainSlot,
};

#[cfg(feature = "audio")]
use crate::audio::atomic_params::AtomicParameterBridge;
#[cfg(feature = "audio")]
use crate::audio::{
    FfiRuntimeControl, HardwareInputRouting, SharedHardwareInputRoutingSelection,
    SharedInputSourceSelection, build_hardware_input_selection, routing_from_channel_id,
};

#[cfg(feature = "audio")]
const INPUT_TRIM_LEVEL_PARAM_ID: &str = "input_trim_level";
#[cfg(feature = "audio")]
const LEGACY_INPUT_GAIN_LEVEL_PARAM_ID: &str = "input_gain_level";

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
    latest_meter_frame: Arc<RwLock<Option<MeterFrame>>>,
    latest_oscilloscope_frame: Arc<RwLock<Option<OscilloscopeFrame>>>,
    audio_status: Arc<RwLock<AudioRuntimeStatus>>,
    hardware_input_selection: Arc<RwLock<GetHardwareInputSelectionResult>>,
    #[cfg(feature = "audio")]
    param_bridge: Option<Arc<AtomicParameterBridge>>,
    #[cfg(feature = "audio")]
    input_source_selection: SharedInputSourceSelection,
    #[cfg(feature = "audio")]
    hardware_input_routing_selection: SharedHardwareInputRoutingSelection,
    #[cfg(feature = "audio")]
    hardware_input_reconfigure_callback: Arc<RwLock<Option<Arc<HardwareInputReconfigureCallback>>>>,
    #[cfg(feature = "audio")]
    runtime_control: Arc<RwLock<Option<FfiRuntimeControl>>>,
}

#[cfg(feature = "audio")]
type HardwareInputReconfigureCallback = dyn Fn(Option<String>) -> Result<(), String> + Send + Sync;

struct SharedState {
    latest_meter_frame: Arc<RwLock<Option<MeterFrame>>>,
    latest_oscilloscope_frame: Arc<RwLock<Option<OscilloscopeFrame>>>,
    audio_status: Arc<RwLock<AudioRuntimeStatus>>,
    hardware_input_selection: Arc<RwLock<GetHardwareInputSelectionResult>>,
}

impl DevServerHost {
    fn initialize_shared_state() -> SharedState {
        let latest_meter_frame = Arc::new(RwLock::new(None));
        let latest_oscilloscope_frame = Arc::new(RwLock::new(None));
        let audio_status = Arc::new(RwLock::new(AudioRuntimeStatus {
            phase: AudioRuntimePhase::Disabled,
            diagnostic: None,
            sample_rate: None,
            buffer_size: None,
            updated_at_ms: now_millis(),
        }));
        let hardware_input_selection = Arc::new(RwLock::new(initial_hardware_input_selection()));

        SharedState {
            latest_meter_frame,
            latest_oscilloscope_frame,
            audio_status,
            hardware_input_selection,
        }
    }

    #[cfg(feature = "audio")]
    fn initial_hardware_input_routing(
        selection: &GetHardwareInputSelectionResult,
    ) -> HardwareInputRouting {
        selection
            .selected_channel_id
            .as_deref()
            .and_then(routing_from_channel_id)
            .unwrap_or(HardwareInputRouting {
                left_channel: 0,
                right_channel: Some(1),
            })
    }

    #[cfg(feature = "audio")]
    fn bridge_missing_parameter_ids(
        bridge: &AtomicParameterBridge,
        parameters: &[ParameterInfo],
    ) -> Vec<String> {
        parameters
            .iter()
            .filter(|parameter| bridge.read(&parameter.id).is_none())
            .map(|parameter| parameter.id.clone())
            .collect()
    }

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
        let shared_state = Self::initialize_shared_state();
        #[cfg(feature = "audio")]
        let initial_hardware_input_routing = {
            let selection = shared_state
                .hardware_input_selection
                .read()
                .expect("hardware_input_selection lock poisoned")
                .clone();
            Self::initial_hardware_input_routing(&selection)
        };

        Self {
            inner,
            latest_meter_frame: shared_state.latest_meter_frame,
            latest_oscilloscope_frame: shared_state.latest_oscilloscope_frame,
            audio_status: shared_state.audio_status,
            hardware_input_selection: shared_state.hardware_input_selection,
            #[cfg(feature = "audio")]
            param_bridge: None,
            #[cfg(feature = "audio")]
            input_source_selection: SharedInputSourceSelection::default(),
            #[cfg(feature = "audio")]
            hardware_input_routing_selection: SharedHardwareInputRoutingSelection::new(
                initial_hardware_input_routing,
            ),
            #[cfg(feature = "audio")]
            hardware_input_reconfigure_callback: Arc::new(RwLock::new(None)),
            #[cfg(feature = "audio")]
            runtime_control: Arc::new(RwLock::new(None)),
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
        let shared_state = Self::initialize_shared_state();
        let initial_hardware_input_routing = {
            let selection = shared_state
                .hardware_input_selection
                .read()
                .expect("hardware_input_selection lock poisoned")
                .clone();
            Self::initial_hardware_input_routing(&selection)
        };

        Self {
            inner,
            latest_meter_frame: shared_state.latest_meter_frame,
            latest_oscilloscope_frame: shared_state.latest_oscilloscope_frame,
            audio_status: shared_state.audio_status,
            hardware_input_selection: shared_state.hardware_input_selection,
            param_bridge: Some(bridge),
            input_source_selection: SharedInputSourceSelection::default(),
            hardware_input_routing_selection: SharedHardwareInputRoutingSelection::new(
                initial_hardware_input_routing,
            ),
            hardware_input_reconfigure_callback: Arc::new(RwLock::new(None)),
            runtime_control: Arc::new(RwLock::new(None)),
        }
    }

    #[cfg(feature = "audio")]
    pub fn set_runtime_control(&self, runtime_control: FfiRuntimeControl) {
        *self
            .runtime_control
            .write()
            .expect("runtime_control lock poisoned") = Some(runtime_control);
    }

    #[cfg(feature = "audio")]
    pub fn clear_runtime_control(&self) {
        *self
            .runtime_control
            .write()
            .expect("runtime_control lock poisoned") = None;
    }

    #[cfg(feature = "audio")]
    pub fn input_source_selection(&self) -> SharedInputSourceSelection {
        self.input_source_selection.clone()
    }

    #[cfg(feature = "audio")]
    pub fn hardware_input_routing_selection(&self) -> SharedHardwareInputRoutingSelection {
        self.hardware_input_routing_selection.clone()
    }

    #[cfg(feature = "audio")]
    pub fn set_hardware_input_reconfigure_callback(
        &self,
        callback: Arc<HardwareInputReconfigureCallback>,
    ) {
        *self
            .hardware_input_reconfigure_callback
            .write()
            .expect("hardware_input_reconfigure_callback lock poisoned") = Some(callback);
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
        #[cfg(feature = "audio")]
        if let Some(ref bridge) = self.param_bridge {
            let missing_param_ids = Self::bridge_missing_parameter_ids(bridge, &new_params);
            if !missing_param_ids.is_empty() {
                return Err(format!(
                    "Parameter schema changed during hot-reload, but audio bridge cannot map new IDs yet. Missing IDs in bridge: {}. Restart `wavecraft start` to apply the new parameter schema.",
                    missing_param_ids.join(", ")
                ));
            }
        }

        self.inner.replace_parameters(new_params)?;

        #[cfg(feature = "audio")]
        if let Some(ref bridge) = self.param_bridge {
            for parameter in self.inner.get_all_parameters() {
                bridge.write(&parameter.id, parameter.value);

                // Keep both legacy and canonical input trim aliases synchronized
                // across hot-reloads to prevent stale bridge slots.
                if parameter.id == INPUT_TRIM_LEVEL_PARAM_ID {
                    bridge.write(LEGACY_INPUT_GAIN_LEVEL_PARAM_ID, parameter.value);
                } else if parameter.id == LEGACY_INPUT_GAIN_LEVEL_PARAM_ID {
                    bridge.write(INPUT_TRIM_LEVEL_PARAM_ID, parameter.value);
                }
            }
        }

        Ok(())
    }

    /// Store the latest metering snapshot for polling-based consumers.
    pub fn set_latest_meter_frame(&self, update: &MeterUpdateNotification) {
        let mut meter = self
            .latest_meter_frame
            .write()
            .expect("latest_meter_frame lock poisoned");
        *meter = Some(MeterFrame {
            peak_l: update.left_peak,
            peak_r: update.right_peak,
            rms_l: update.left_rms,
            rms_r: update.right_rms,
            timestamp: update.timestamp_us,
        });
    }

    /// Store the latest oscilloscope frame for polling-based consumers.
    pub fn set_latest_oscilloscope_frame(&self, frame: OscilloscopeFrame) {
        let mut oscilloscope = self
            .latest_oscilloscope_frame
            .write()
            .expect("latest_oscilloscope_frame lock poisoned");
        *oscilloscope = Some(frame);
    }

    /// Update the shared audio runtime status.
    pub fn set_audio_status(&self, status: AudioRuntimeStatus) {
        let mut current = self
            .audio_status
            .write()
            .expect("audio_status lock poisoned");
        *current = status;
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
        if result.is_ok()
            && let Some(ref bridge) = self.param_bridge
        {
            bridge.write(id, value);

            // Hot-reload compatibility: if the parameter model is renamed from
            // input_gain_level -> input_trim_level (or vice-versa), the bridge
            // may still contain the old key until full restart. Mirror writes
            // across both IDs so audible gain control remains live.
            if id == INPUT_TRIM_LEVEL_PARAM_ID {
                bridge.write(LEGACY_INPUT_GAIN_LEVEL_PARAM_ID, value);
            } else if id == LEGACY_INPUT_GAIN_LEVEL_PARAM_ID {
                bridge.write(INPUT_TRIM_LEVEL_PARAM_ID, value);
            }
        }

        result
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        self.inner.get_all_parameters()
    }

    fn get_meter_frame(&self) -> Option<MeterFrame> {
        *self
            .latest_meter_frame
            .read()
            .expect("latest_meter_frame lock poisoned")
    }

    fn get_oscilloscope_frame(&self) -> Option<OscilloscopeFrame> {
        self.latest_oscilloscope_frame
            .read()
            .expect("latest_oscilloscope_frame lock poisoned")
            .clone()
    }

    fn request_resize(&self, width: u32, height: u32) -> bool {
        self.inner.request_resize(width, height)
    }

    fn get_audio_status(&self) -> Option<AudioRuntimeStatus> {
        Some(
            self.audio_status
                .read()
                .expect("audio_status lock poisoned")
                .clone(),
        )
    }

    fn get_input_source(&self) -> Option<GetInputSourceResult> {
        self.inner.get_input_source()
    }

    fn set_input_source(&self, source: InputSourceKind) -> Result<(), BridgeError> {
        self.inner.set_input_source(source)?;
        #[cfg(feature = "audio")]
        self.input_source_selection.store(source);
        Ok(())
    }

    fn get_hardware_input_selection(&self) -> Option<GetHardwareInputSelectionResult> {
        self.hardware_input_selection
            .read()
            .ok()
            .map(|guard| guard.clone())
    }

    fn set_hardware_input_selection(
        &self,
        selection: SetHardwareInputSelectionParams,
    ) -> Result<(), BridgeError> {
        #[cfg(feature = "audio")]
        {
            let current = self
                .hardware_input_selection
                .read()
                .map(|guard| guard.clone())
                .unwrap_or_else(|_| initial_hardware_input_selection());
            let next = build_hardware_input_selection(
                selection
                    .selected_device_id
                    .as_deref()
                    .or(current.selected_device_id.as_deref()),
                selection
                    .selected_channel_id
                    .as_deref()
                    .or(current.selected_channel_id.as_deref()),
            )
            .map_err(|error| BridgeError::Internal(error.to_string()))?;

            let device_changed = next.selected_device_id != current.selected_device_id;

            if device_changed {
                let callback = self
                    .hardware_input_reconfigure_callback
                    .read()
                    .expect("hardware_input_reconfigure_callback lock poisoned")
                    .clone();

                if let Some(callback) = callback {
                    callback(next.selected_device_id.clone()).map_err(BridgeError::Internal)?;
                } else {
                    return Err(BridgeError::Internal(
                        "Live hardware input device switching is not yet available in this host. Restart `wavecraft start` after selecting a different device.".to_string(),
                    ));
                }
            }

            if let Some(selected_channel_id) = next.selected_channel_id.as_deref()
                && let Some(routing) = routing_from_channel_id(selected_channel_id)
            {
                self.hardware_input_routing_selection.store(routing);
            }

            *self
                .hardware_input_selection
                .write()
                .expect("hardware_input_selection lock poisoned") = next;

            Ok(())
        }

        #[cfg(not(feature = "audio"))]
        {
            let current = self
                .hardware_input_selection
                .read()
                .map(|guard| guard.clone())
                .unwrap_or_else(|_| initial_hardware_input_selection());
            *self
                .hardware_input_selection
                .write()
                .expect("hardware_input_selection lock poisoned") =
                GetHardwareInputSelectionResult {
                    selected_device_id: selection.selected_device_id.or(current.selected_device_id),
                    available_devices: current.available_devices,
                    selected_channel_id: selection
                        .selected_channel_id
                        .or(current.selected_channel_id),
                    available_channels: current.available_channels,
                };
            Ok(())
        }
    }

    fn get_signal_chain_order(&self) -> Vec<SignalChainSlot> {
        self.inner.get_signal_chain_order()
    }

    fn set_signal_chain_order(&self, order: Vec<SignalChainSlot>) -> Result<(), BridgeError> {
        #[cfg(feature = "audio")]
        {
            let runtime_control = self
                .runtime_control
                .read()
                .expect("runtime_control lock poisoned")
                .clone();

            if let Some(runtime_control) = runtime_control
                && !runtime_control.set_signal_chain_order(&order)
            {
                return Err(BridgeError::Internal(
                    "Live browser-dev runtime rejected signal-chain order update".to_string(),
                ));
            }
        }

        self.inner.set_signal_chain_order(order)
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

fn initial_hardware_input_selection() -> GetHardwareInputSelectionResult {
    #[cfg(feature = "audio")]
    {
        build_hardware_input_selection(None, None).unwrap_or(GetHardwareInputSelectionResult {
            selected_device_id: None,
            available_devices: Vec::new(),
            selected_channel_id: None,
            available_channels: Vec::new(),
        })
    }

    #[cfg(not(feature = "audio"))]
    {
        GetHardwareInputSelectionResult {
            selected_device_id: None,
            available_devices: Vec::new(),
            selected_channel_id: None,
            available_channels: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "audio")]
    use crate::audio::FfiProcessor;
    #[cfg(feature = "audio")]
    use std::ffi::c_void;
    #[cfg(feature = "audio")]
    use std::sync::atomic::{AtomicBool, Ordering};
    use wavecraft_protocol::ParameterType;

    #[cfg(feature = "audio")]
    static RUNTIME_SET_ORDER_CALLED: AtomicBool = AtomicBool::new(false);

    #[cfg(feature = "audio")]
    extern "C" fn runtime_control_mock_create() -> *mut c_void {
        std::ptr::dangling_mut::<c_void>()
    }

    #[cfg(feature = "audio")]
    extern "C" fn runtime_control_mock_process(
        _instance: *mut c_void,
        _channels: *mut *mut f32,
        _num_channels: u32,
        _num_samples: u32,
    ) {
    }

    #[cfg(feature = "audio")]
    unsafe extern "C" fn runtime_control_mock_apply_plain_values(
        _instance: *mut c_void,
        _values_ptr: *const f32,
        _len: usize,
    ) {
    }

    #[cfg(feature = "audio")]
    unsafe extern "C" fn runtime_control_mock_set_order_accept(
        _instance: *mut c_void,
        _json_ptr: *const std::os::raw::c_char,
    ) -> bool {
        RUNTIME_SET_ORDER_CALLED.store(true, Ordering::SeqCst);
        true
    }

    #[cfg(feature = "audio")]
    unsafe extern "C" fn runtime_control_mock_set_order_reject(
        _instance: *mut c_void,
        _json_ptr: *const std::os::raw::c_char,
    ) -> bool {
        RUNTIME_SET_ORDER_CALLED.store(true, Ordering::SeqCst);
        false
    }

    #[cfg(feature = "audio")]
    extern "C" fn runtime_control_mock_take_latest_oscilloscope_frame_json(
        _instance: *mut c_void,
    ) -> *mut std::os::raw::c_char {
        std::ffi::CString::new("null").unwrap().into_raw()
    }

    #[cfg(feature = "audio")]
    extern "C" fn runtime_control_mock_set_sample_rate(_instance: *mut c_void, _sample_rate: f32) {}

    #[cfg(feature = "audio")]
    extern "C" fn runtime_control_mock_reset(_instance: *mut c_void) {}

    #[cfg(feature = "audio")]
    extern "C" fn runtime_control_mock_drop(_instance: *mut c_void) {}

    #[cfg(feature = "audio")]
    fn runtime_control_mock_vtable(
        set_order: unsafe extern "C" fn(*mut c_void, *const std::os::raw::c_char) -> bool,
    ) -> wavecraft_protocol::DevProcessorVTable {
        wavecraft_protocol::DevProcessorVTable {
            version: wavecraft_protocol::DEV_PROCESSOR_VTABLE_VERSION,
            create: runtime_control_mock_create,
            process: runtime_control_mock_process,
            apply_plain_values: runtime_control_mock_apply_plain_values,
            set_signal_chain_order_json: set_order,
            take_latest_oscilloscope_frame_json:
                runtime_control_mock_take_latest_oscilloscope_frame_json,
            set_sample_rate: runtime_control_mock_set_sample_rate,
            reset: runtime_control_mock_reset,
            drop: runtime_control_mock_drop,
        }
    }

    #[cfg(feature = "audio")]
    fn runtime_test_order(primary_processor_id: &str) -> Vec<SignalChainSlot> {
        vec![
            SignalChainSlot {
                id: primary_processor_id.to_string(),
                slot_type: wavecraft_protocol::SlotType::Processor,
            },
            SignalChainSlot {
                id: "oscilloscope_tap".to_string(),
                slot_type: wavecraft_protocol::SlotType::Tap,
            },
        ]
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
        // Initially no externally provided meter data.
        assert!(host.get_meter_frame().is_none());

        host.set_latest_meter_frame(&MeterUpdateNotification {
            timestamp_us: 42,
            left_peak: 0.9,
            left_rms: 0.4,
            right_peak: 0.8,
            right_rms: 0.3,
        });

        let frame = host
            .get_meter_frame()
            .expect("meter frame should be populated after update");
        assert!((frame.peak_l - 0.9).abs() < f32::EPSILON);
        assert!((frame.rms_r - 0.3).abs() < f32::EPSILON);
        assert_eq!(frame.timestamp, 42);
    }

    #[test]
    fn test_audio_status_roundtrip() {
        let host = DevServerHost::new(test_params());

        let status = AudioRuntimeStatus {
            phase: AudioRuntimePhase::RunningInputOnly,
            diagnostic: None,
            sample_rate: Some(44100.0),
            buffer_size: Some(512),
            updated_at_ms: 100,
        };

        host.set_audio_status(status.clone());

        let stored = host
            .get_audio_status()
            .expect("audio status should always be present in dev host");
        assert_eq!(stored.phase, status.phase);
        assert_eq!(stored.buffer_size, status.buffer_size);
    }

    #[test]
    fn test_input_source_roundtrip() {
        let host = DevServerHost::new(test_params());

        host.set_input_source(InputSourceKind::TestTone)
            .expect("should set input source");

        let selection = host.get_input_source().expect("selection should exist");
        assert_eq!(selection.selected, InputSourceKind::TestTone);
    }

    #[test]
    fn test_get_oscilloscope_frame() {
        let host = DevServerHost::new(test_params());
        assert!(host.get_oscilloscope_frame().is_none());

        host.set_latest_oscilloscope_frame(OscilloscopeFrame {
            points_l: vec![0.1; 1024],
            points_r: vec![0.2; 1024],
            sample_rate: 48_000.0,
            timestamp: 777,
            no_signal: false,
            trigger_mode: wavecraft_protocol::OscilloscopeTriggerMode::RisingZeroCrossing,
        });

        let frame = host
            .get_oscilloscope_frame()
            .expect("oscilloscope frame should be populated");
        assert_eq!(frame.points_l.len(), 1024);
        assert_eq!(frame.points_r.len(), 1024);
        assert_eq!(frame.timestamp, 777);
    }

    #[cfg(feature = "audio")]
    #[test]
    fn test_set_signal_chain_order_applies_runtime_before_mirroring_host_state() {
        let host = DevServerHost::new(test_params());
        let processor = FfiProcessor::new(&runtime_control_mock_vtable(
            runtime_control_mock_set_order_accept,
        ))
        .expect("mock runtime control should construct");
        host.set_runtime_control(processor.runtime_control());
        RUNTIME_SET_ORDER_CALLED.store(false, Ordering::SeqCst);

        let order = runtime_test_order("input_trim");
        host.set_signal_chain_order(order.clone())
            .expect("runtime-accepted order should succeed");

        assert!(
            RUNTIME_SET_ORDER_CALLED.load(Ordering::SeqCst),
            "live runtime should receive the order before the host mirrors it"
        );
        assert_eq!(host.get_signal_chain_order(), order);
    }

    #[cfg(feature = "audio")]
    #[test]
    fn test_set_signal_chain_order_rejection_does_not_mutate_host_state() {
        let host = DevServerHost::new(test_params());
        let baseline = runtime_test_order("test_tone");
        host.set_signal_chain_order(baseline.clone())
            .expect("baseline order should be accepted without runtime control");

        let processor = FfiProcessor::new(&runtime_control_mock_vtable(
            runtime_control_mock_set_order_reject,
        ))
        .expect("mock runtime control should construct");
        host.set_runtime_control(processor.runtime_control());
        RUNTIME_SET_ORDER_CALLED.store(false, Ordering::SeqCst);

        let rejected = runtime_test_order("input_trim");
        let error = host
            .set_signal_chain_order(rejected)
            .expect_err("runtime rejection should surface as an error");

        assert!(
            RUNTIME_SET_ORDER_CALLED.load(Ordering::SeqCst),
            "runtime control should have been consulted"
        );
        assert!(
            error
                .to_string()
                .contains("rejected signal-chain order update"),
            "error should describe runtime rejection: {error}"
        );
        assert_eq!(host.get_signal_chain_order(), baseline);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_set_audio_status_inside_runtime_does_not_panic() {
        let host = DevServerHost::new(test_params());

        host.set_audio_status(AudioRuntimeStatus {
            phase: AudioRuntimePhase::Initializing,
            diagnostic: None,
            sample_rate: Some(48000.0),
            buffer_size: Some(256),
            updated_at_ms: 200,
        });

        let stored = host
            .get_audio_status()
            .expect("audio status should always be present in dev host");
        assert_eq!(stored.phase, AudioRuntimePhase::Initializing);
        assert_eq!(stored.buffer_size, Some(256));
    }
    #[cfg(feature = "audio")]
    fn soft_clip_bridge_seed_params() -> Vec<ParameterInfo> {
        vec![
            ParameterInfo {
                id: "soft_clip_bypass".to_string(),
                name: "Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 1.0,
                unit: None,
                group: Some("Saturator".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_drive_db".to_string(),
                name: "Drive".to_string(),
                param_type: ParameterType::Float,
                value: 12.0,
                default: 12.0,
                min: 0.0,
                max: 30.0,
                unit: Some("dB".to_string()),
                group: Some("Saturator".to_string()),
                variants: None,
            },
        ]
    }

    #[cfg(feature = "audio")]
    fn soft_clip_expanded_params() -> Vec<ParameterInfo> {
        vec![
            ParameterInfo {
                id: "soft_clip_bypass".to_string(),
                name: "Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 1.0,
                unit: None,
                group: Some("Saturator".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_drive_db".to_string(),
                name: "Drive".to_string(),
                param_type: ParameterType::Float,
                value: 12.0,
                default: 12.0,
                min: 0.0,
                max: 30.0,
                unit: Some("dB".to_string()),
                group: Some("Saturator".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_output_db".to_string(),
                name: "Output".to_string(),
                param_type: ParameterType::Float,
                value: 0.0,
                default: 0.0,
                min: -24.0,
                max: 24.0,
                unit: Some("dB".to_string()),
                group: Some("Saturator".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_mix".to_string(),
                name: "Mix".to_string(),
                param_type: ParameterType::Float,
                value: 1.0,
                default: 1.0,
                min: 0.0,
                max: 1.0,
                unit: Some("%".to_string()),
                group: Some("Saturator".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_tone".to_string(),
                name: "Tone".to_string(),
                param_type: ParameterType::Float,
                value: 0.55,
                default: 0.55,
                min: 0.0,
                max: 1.0,
                unit: Some("%".to_string()),
                group: Some("Saturator".to_string()),
                variants: None,
            },
        ]
    }

    #[cfg(feature = "audio")]
    #[test]
    fn replace_parameters_rejects_bridge_schema_drift_for_new_soft_clip_controls() {
        let bridge = Arc::new(AtomicParameterBridge::new(&soft_clip_bridge_seed_params()));
        let host = DevServerHost::with_param_bridge(soft_clip_bridge_seed_params(), bridge);

        let result = host.replace_parameters(soft_clip_expanded_params());
        assert!(result.is_err(), "expected schema drift to be rejected");

        let error = result.expect_err("schema drift should return an error");
        assert!(error.contains("soft_clip_output_db"));
        assert!(error.contains("soft_clip_mix"));
        assert!(error.contains("soft_clip_tone"));

        // Existing bridge-backed controls remain available.
        assert!(host.get_parameter("soft_clip_drive_db").is_some());
        // New controls should not appear after rejected replacement.
        assert!(host.get_parameter("soft_clip_output_db").is_none());
        assert!(host.get_parameter("soft_clip_mix").is_none());
        assert!(host.get_parameter("soft_clip_tone").is_none());
    }

    #[cfg(feature = "audio")]
    #[test]
    fn replace_parameters_accepts_when_bridge_schema_matches() {
        let params = soft_clip_expanded_params();
        let bridge = Arc::new(AtomicParameterBridge::new(&params));
        let host = DevServerHost::with_param_bridge(params.clone(), bridge);

        host.replace_parameters(params)
            .expect("matching schema should replace parameters");

        assert!(host.get_parameter("soft_clip_output_db").is_some());
        assert!(host.get_parameter("soft_clip_mix").is_some());
        assert!(host.get_parameter("soft_clip_tone").is_some());
    }
}
