//! IPC message contracts for WebView ↔ Rust communication
//!
//! This module defines JSON-RPC 2.0 style messages used for bidirectional
//! communication between the React UI (running in WebView) and the Rust
//! application logic.
//!
//! # Architecture
//!
//! - **Request/Response**: UI initiates, Rust responds (e.g., setParameter, getParameter)
//! - **Notifications**: Rust pushes updates to UI (e.g., parameter changes from host)
//!
//! # JSON-RPC 2.0 Compatibility
//!
//! Messages follow JSON-RPC 2.0 conventions:
//! - Requests have `id`, `method`, and `params`
//! - Responses have `id` and either `result` or `error`
//! - Notifications have `method` and `params` but no `id`

use serde::{Deserialize, Serialize};

#[path = "ipc/envelope.rs"]
mod envelope;
#[path = "ipc/errors.rs"]
mod errors;

pub use envelope::{IpcNotification, IpcRequest, IpcResponse, RequestId};
pub use errors::{
    ERROR_INTERNAL, ERROR_INVALID_PARAMS, ERROR_INVALID_REQUEST, ERROR_METHOD_NOT_FOUND,
    ERROR_PARAM_NOT_FOUND, ERROR_PARAM_OUT_OF_RANGE, ERROR_PARSE, IpcError,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = IpcRequest::new(
            RequestId::Number(1),
            METHOD_GET_PARAMETER,
            Some(serde_json::json!({"id": "gain"})),
        );

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"getParameter\""));
    }

    #[test]
    fn test_response_serialization() {
        let resp = IpcResponse::success(
            RequestId::Number(1),
            GetParameterResult {
                id: "gain".to_string(),
                value: 0.5,
            },
        );

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_error_response() {
        let resp = IpcResponse::error(
            RequestId::String("test".to_string()),
            IpcError::method_not_found("unknownMethod"),
        );

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"error\""));
        assert!(!json.contains("\"result\""));
    }

    #[test]
    fn test_notification_serialization() {
        let notif = IpcNotification::new(
            NOTIFICATION_PARAMETER_CHANGED,
            ParameterChangedNotification {
                id: "gain".to_string(),
                value: 0.8,
            },
        );

        let json = serde_json::to_string(&notif).unwrap();
        println!("Notification JSON: {}", json);
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"parameterChanged\""));
        // The ParameterChangedNotification has an "id" field, which is OK
        // We're checking that the notification itself doesn't have a request id
    }

    #[test]
    fn test_register_audio_serialization() {
        let req = IpcRequest::new(
            RequestId::String("audio-1".to_string()),
            METHOD_REGISTER_AUDIO,
            Some(serde_json::json!({
                "client_id": "dev-audio",
                "sample_rate": 44100.0,
                "buffer_size": 512
            })),
        );

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"method\":\"registerAudio\""));
        assert!(json.contains("\"sample_rate\":44100"));
    }

    #[test]
    fn test_meter_update_notification() {
        let notif = IpcNotification::new(
            NOTIFICATION_METER_UPDATE,
            MeterUpdateNotification {
                timestamp_us: 1000,
                left_peak: 0.5,
                left_rms: 0.3,
                right_peak: 0.6,
                right_rms: 0.4,
            },
        );

        let json = serde_json::to_string(&notif).unwrap();
        assert!(json.contains("\"method\":\"meterUpdate\""));
        assert!(json.contains("\"left_peak\":0.5"));
    }

    #[test]
    fn test_audio_status_serialization() {
        let result = GetAudioStatusResult {
            status: Some(AudioRuntimeStatus {
                phase: AudioRuntimePhase::RunningFullDuplex,
                diagnostic: None,
                sample_rate: Some(44100.0),
                buffer_size: Some(512),
                updated_at_ms: 123,
            }),
        };

        let json = serde_json::to_string(&result).expect("status result should serialize");
        assert!(json.contains("\"phase\":\"runningFullDuplex\""));
        assert!(json.contains("\"sample_rate\":44100"));
    }

    #[test]
    fn test_oscilloscope_frame_serialization() {
        let result = GetOscilloscopeFrameResult {
            frame: Some(OscilloscopeFrame {
                points_l: vec![0.0; 1024],
                points_r: vec![0.0; 1024],
                sample_rate: 44100.0,
                timestamp: 7,
                no_signal: true,
                trigger_mode: OscilloscopeTriggerMode::RisingZeroCrossing,
            }),
        };

        let json = serde_json::to_string(&result).expect("oscilloscope result should serialize");
        assert!(json.contains("\"sample_rate\":44100"));
        assert!(json.contains("\"trigger_mode\":\"risingZeroCrossing\""));
    }

    #[test]
    fn parameter_info_with_variants_serializes_correctly() {
        let info = ParameterInfo {
            id: "osc_waveform".to_string(),
            name: "Waveform".to_string(),
            param_type: ParameterType::Enum,
            value: 0.0,
            default: 0.0,
            min: 0.0,
            max: 3.0,
            unit: None,
            group: None,
            variants: Some(vec![
                "Sine".to_string(),
                "Square".to_string(),
                "Saw".to_string(),
                "Triangle".to_string(),
            ]),
        };

        let json = serde_json::to_string(&info).expect("parameter info should serialize");
        assert!(json.contains("\"variants\""));

        let deserialized: ParameterInfo =
            serde_json::from_str(&json).expect("parameter info should deserialize");
        assert_eq!(
            deserialized.variants.expect("variants should exist").len(),
            4
        );
    }

    #[test]
    fn parameter_info_without_variants_omits_field() {
        let info = ParameterInfo {
            id: "gain".to_string(),
            name: "Gain".to_string(),
            param_type: ParameterType::Float,
            value: 0.5,
            default: 0.5,
            min: 0.0,
            max: 1.0,
            unit: Some("dB".to_string()),
            group: None,
            variants: None,
        };

        let json = serde_json::to_string(&info).expect("parameter info should serialize");
        assert!(!json.contains("\"variants\""));
    }
}

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
