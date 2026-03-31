use serde::{Deserialize, Serialize};

// ============================================================================
// Method-Specific Types
// ============================================================================

// ----------------------------------------------------------------------------
// getParameter
// ----------------------------------------------------------------------------

/// Parameters for getParameter request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParameterParams {
    /// Parameter ID to retrieve
    pub id: String,
}

/// Result of getParameter request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParameterResult {
    /// Parameter ID
    pub id: String,
    /// Current parameter value in the parameter's declared range.
    pub value: f32,
}

// ----------------------------------------------------------------------------
// setParameter
// ----------------------------------------------------------------------------

/// Parameters for setParameter request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParameterParams {
    /// Parameter ID to update
    pub id: String,
    /// New parameter value in the parameter's declared range.
    pub value: f32,
}

/// Result of setParameter request (empty success)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParameterResult {}

// ----------------------------------------------------------------------------
// getAllParameters
// ----------------------------------------------------------------------------

/// Result of getAllParameters request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAllParametersResult {
    /// List of all parameters with their metadata and current values
    pub parameters: Vec<ParameterInfo>,
}

/// Information about a single parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    /// Parameter ID (unique identifier)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Parameter type (float, bool, enum, etc.)
    #[serde(rename = "type")]
    pub param_type: ParameterType,
    /// Current parameter value in the parameter's declared range.
    pub value: f32,
    /// Default parameter value in the parameter's declared range.
    pub default: f32,
    /// Minimum value for this parameter.
    pub min: f32,
    /// Maximum value for this parameter.
    pub max: f32,
    /// Unit suffix for display (e.g., "dB", "%", "Hz")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Group name for UI organization (e.g., "Input", "Processing", "Output")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Variant labels for enum parameters (e.g., ["Sine", "Square", "Saw", "Triangle"]).
    /// Only present when `param_type` is `Enum`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<String>>,
}

/// Information about a discovered processor in the signal chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorInfo {
    /// Canonical processor ID (snake_case type-derived identifier).
    pub id: String,
}

/// Selectable input source for the dev/runtime audio path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InputSourceKind {
    HardwareInput,
    TestTone,
}

/// UI-facing description of an available input source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSourceOption {
    /// Stable source identifier.
    pub id: InputSourceKind,
    /// Human-readable label.
    pub label: String,
    /// Optional helper text for UI affordances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Result for getInputSource method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetInputSourceResult {
    /// Currently selected input source.
    pub selected: InputSourceKind,
    /// Available input sources for this runtime.
    pub available: Vec<InputSourceOption>,
}

/// Parameters for setInputSource request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetInputSourceParams {
    /// Newly selected input source.
    pub selected: InputSourceKind,
}

/// Result of a successful setInputSource request (empty success).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetInputSourceResult {}

/// Notification sent when the active input source changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSourceChangedNotification {
    /// Newly selected input source.
    pub selected: InputSourceKind,
}

/// UI-facing description of an available hardware input device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInputDeviceOption {
    /// Stable device identifier for the current runtime session.
    pub id: String,
    /// Human-readable device label.
    pub label: String,
    /// Number of input channels available from the device's default config.
    pub channel_count: u16,
    /// Optional helper text for UI affordances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// UI-facing description of an available hardware input channel routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInputChannelOption {
    /// Stable routing identifier for the current device.
    pub id: String,
    /// Human-readable routing label.
    pub label: String,
    /// Optional helper text for UI affordances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Result for getHardwareInputSelection method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHardwareInputSelectionResult {
    /// Currently selected device identifier, if any input device is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_device_id: Option<String>,
    /// Available hardware input devices.
    pub available_devices: Vec<HardwareInputDeviceOption>,
    /// Currently selected channel routing identifier, if any routing is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_channel_id: Option<String>,
    /// Available channel routings for the selected device.
    pub available_channels: Vec<HardwareInputChannelOption>,
}

/// Parameters for setHardwareInputSelection request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetHardwareInputSelectionParams {
    /// Newly selected device identifier, if changing the active hardware device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_device_id: Option<String>,
    /// Newly selected channel routing identifier, if changing the active routing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_channel_id: Option<String>,
}

/// Result of a successful setHardwareInputSelection request (empty success).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetHardwareInputSelectionResult {}

/// Notification sent when the hardware input device/routing selection changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInputSelectionChangedNotification {
    /// Currently selected device identifier, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_device_id: Option<String>,
    /// Available hardware input devices.
    pub available_devices: Vec<HardwareInputDeviceOption>,
    /// Currently selected channel routing identifier, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_channel_id: Option<String>,
    /// Available channel routings for the selected device.
    pub available_channels: Vec<HardwareInputChannelOption>,
}

/// Parameter type discriminator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    Float,
    Bool,
    Enum,
}

// ----------------------------------------------------------------------------
// Notification: parameterChanged
// ----------------------------------------------------------------------------

/// Notification sent when a parameter changes (e.g., from host automation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterChangedNotification {
    /// Parameter ID that changed
    pub id: String,
    /// New parameter value in the parameter's declared range.
    pub value: f32,
}

// ============================================================================
// Method Name Constants
// ============================================================================

/// Method: Get single parameter value
pub const METHOD_GET_PARAMETER: &str = "getParameter";
/// Method: Set single parameter value
pub const METHOD_SET_PARAMETER: &str = "setParameter";
/// Method: Get all parameters with metadata
pub const METHOD_GET_ALL_PARAMETERS: &str = "getAllParameters";
/// Method: Get current meter frame (peak/RMS levels)
pub const METHOD_GET_METER_FRAME: &str = "getMeterFrame";
/// Method: Get current passthrough-local meter frame (peak/RMS levels)
pub const METHOD_GET_PASSTHROUGH_METER_FRAME: &str = "getPassthroughMeterFrame";
/// Method: Get current oscilloscope frame (1024-point waveform)
pub const METHOD_GET_OSCILLOSCOPE_FRAME: &str = "getOscilloscopeFrame";
/// Method: Get current audio runtime status
pub const METHOD_GET_AUDIO_STATUS: &str = "getAudioStatus";
/// Method: Get current selected input source and options
pub const METHOD_GET_INPUT_SOURCE: &str = "getInputSource";
/// Method: Set current input source
pub const METHOD_SET_INPUT_SOURCE: &str = "setInputSource";
/// Method: Get current selected hardware input device/routing and options
pub const METHOD_GET_HARDWARE_INPUT_SELECTION: &str = "getHardwareInputSelection";
/// Method: Set current hardware input device/routing
pub const METHOD_SET_HARDWARE_INPUT_SELECTION: &str = "setHardwareInputSelection";
/// Method: Request resize of editor window
pub const METHOD_REQUEST_RESIZE: &str = "requestResize";
/// Method: Register audio client with dev server
pub const METHOD_REGISTER_AUDIO: &str = "registerAudio";
/// Notification: Parameter changed (push from Rust to UI)
pub const NOTIFICATION_PARAMETER_CHANGED: &str = "parameterChanged";
/// Notification: Meter update from audio binary (push to browser)
pub const NOTIFICATION_METER_UPDATE: &str = "meterUpdate";
/// Notification: Audio runtime status changed
pub const NOTIFICATION_AUDIO_STATUS_CHANGED: &str = "audioStatusChanged";
/// Notification: Input source changed
pub const NOTIFICATION_INPUT_SOURCE_CHANGED: &str = "inputSourceChanged";
/// Notification: Hardware input selection changed
pub const NOTIFICATION_HARDWARE_INPUT_SELECTION_CHANGED: &str = "hardwareInputSelectionChanged";
/// Method: Get current signal chain order (processors + taps)
pub const METHOD_GET_SIGNAL_CHAIN_ORDER: &str = "getSignalChainOrder";
/// Method: Set signal chain order
pub const METHOD_SET_SIGNAL_CHAIN_ORDER: &str = "setSignalChainOrder";
/// Notification: Signal chain order changed (push from Rust to UI)
pub const NOTIFICATION_SIGNAL_CHAIN_ORDER_CHANGED: &str = "signalChainOrderChanged";

// ============================================================================
// Metering Types
// ============================================================================

/// Meter frame data for UI visualization.
///
/// All values are in linear scale (not dB).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MeterFrame {
    /// Left channel peak (linear, 0.0 to 1.0+)
    pub peak_l: f32,
    /// Right channel peak (linear, 0.0 to 1.0+)
    pub peak_r: f32,
    /// Left channel RMS (linear, 0.0 to 1.0+)
    pub rms_l: f32,
    /// Right channel RMS (linear, 0.0 to 1.0+)
    pub rms_r: f32,
    /// Sample timestamp (monotonic)
    pub timestamp: u64,
}

/// Result for getMeterFrame method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMeterFrameResult {
    /// Latest meter frame, or null if no data available
    pub frame: Option<MeterFrame>,
}

// ============================================================================
// Oscilloscope Types
// ============================================================================

/// Trigger mode for oscilloscope frame alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OscilloscopeTriggerMode {
    RisingZeroCrossing,
}

/// Channel view mode for oscilloscope visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OscilloscopeChannelView {
    Overlay,
    Left,
    Right,
}

/// Oscilloscope waveform frame data for UI visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscilloscopeFrame {
    /// Left channel waveform points (length 1024).
    pub points_l: Vec<f32>,
    /// Right channel waveform points (length 1024).
    pub points_r: Vec<f32>,
    /// Sample rate in Hz used to capture the frame.
    pub sample_rate: f32,
    /// Sample timestamp (monotonic).
    pub timestamp: u64,
    /// True when signal amplitude stayed below threshold for full frame.
    pub no_signal: bool,
    /// Trigger mode used for alignment.
    pub trigger_mode: OscilloscopeTriggerMode,
}

/// Result for getOscilloscopeFrame method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOscilloscopeFrameResult {
    /// Latest oscilloscope frame, or null if no data available.
    pub frame: Option<OscilloscopeFrame>,
}

// ----------------------------------------------------------------------------
// getAudioStatus
// ----------------------------------------------------------------------------

/// Audio runtime phase as observed by browser dev mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioRuntimePhase {
    Disabled,
    Initializing,
    RunningFullDuplex,
    RunningInputOnly,
    Degraded,
    Failed,
}

/// Structured diagnostic code for audio startup/runtime issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioDiagnosticCode {
    LoaderUnavailable,
    VtableMissing,
    ProcessorCreateFailed,
    NoInputDevice,
    InputPermissionDenied,
    NoOutputDevice,
    StreamStartFailed,
    Unknown,
}

/// Optional diagnostic details for the current runtime status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDiagnostic {
    /// Machine-readable diagnostic code.
    pub code: AudioDiagnosticCode,
    /// Human-readable error/diagnostic message.
    pub message: String,
    /// Optional actionable hint for the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Current audio runtime status for browser dev mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRuntimeStatus {
    /// Current runtime phase.
    pub phase: AudioRuntimePhase,
    /// Optional startup/runtime diagnostic details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<AudioDiagnostic>,
    /// Active sample rate when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<f32>,
    /// Active audio buffer size when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_size: Option<u32>,
    /// Last update timestamp (milliseconds since UNIX epoch).
    pub updated_at_ms: u64,
}

/// Result for getAudioStatus method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAudioStatusResult {
    /// Current status if available on this host.
    pub status: Option<AudioRuntimeStatus>,
}

// ----------------------------------------------------------------------------
// requestResize
// ----------------------------------------------------------------------------

/// Parameters for requestResize request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestResizeParams {
    /// Desired width in logical pixels
    pub width: u32,
    /// Desired height in logical pixels
    pub height: u32,
}

/// Result of requestResize request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestResizeResult {
    /// Whether the host approved the resize
    pub accepted: bool,
}

// ----------------------------------------------------------------------------
// registerAudio
// ----------------------------------------------------------------------------

/// Parameters for registerAudio request (audio binary → dev server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAudioParams {
    /// Unique client identifier
    pub client_id: String,
    /// Audio sample rate (e.g., 44100.0)
    pub sample_rate: f32,
    /// Buffer size in samples
    pub buffer_size: u32,
}

/// Result of registerAudio request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAudioResult {
    /// Acknowledgment message
    pub status: String,
}

// ----------------------------------------------------------------------------
// getSignalChainOrder / setSignalChainOrder
// ----------------------------------------------------------------------------

/// Type discriminator for a signal chain slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SlotType {
    Processor,
    Tap,
}

/// A single slot in the signal chain order.
///
/// Each slot carries an explicit `slot_type` and an `id` matching the
/// processor or tap type name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalChainSlot {
    /// Processor or tap type identifier (e.g., "TestToneProcessor", "OscilloscopeTap").
    pub id: String,
    /// Whether this slot is a processor or a tap.
    #[serde(rename = "type")]
    pub slot_type: SlotType,
}

/// Result of getSignalChainOrder request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSignalChainOrderResult {
    /// Ordered list of signal chain slots (processors + taps).
    pub slots: Vec<SignalChainSlot>,
}

/// Parameters for setSignalChainOrder request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSignalChainOrderParams {
    /// Desired signal chain slot order.
    pub slots: Vec<SignalChainSlot>,
}

/// Result of a successful setSignalChainOrder request (empty body).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSignalChainOrderResult {}

// ----------------------------------------------------------------------------
// Notification: signalChainOrderChanged
// ----------------------------------------------------------------------------

/// Notification sent when the active signal chain order changes (server → client).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalChainOrderChangedNotification {
    /// New active signal chain slot order.
    pub slots: Vec<SignalChainSlot>,
}

// ----------------------------------------------------------------------------
// Notification: meterUpdate
// ----------------------------------------------------------------------------

/// Notification sent from audio binary to browser via dev server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterUpdateNotification {
    /// Timestamp in microseconds
    pub timestamp_us: u64,
    /// Left channel peak (linear scale)
    pub left_peak: f32,
    /// Left channel RMS (linear scale)
    pub left_rms: f32,
    /// Right channel peak (linear scale)
    pub right_peak: f32,
    /// Right channel RMS (linear scale)
    pub right_rms: f32,
}
