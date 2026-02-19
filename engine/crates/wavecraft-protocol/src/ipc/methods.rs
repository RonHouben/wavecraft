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
/// Method: Get current oscilloscope frame (1024-point waveform)
pub const METHOD_GET_OSCILLOSCOPE_FRAME: &str = "getOscilloscopeFrame";
/// Method: Get current audio runtime status
pub const METHOD_GET_AUDIO_STATUS: &str = "getAudioStatus";
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
