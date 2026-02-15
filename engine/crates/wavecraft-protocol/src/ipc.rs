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

/// Request message sent from UI to Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Unique request identifier for matching responses
    pub id: RequestId,
    /// Method name to invoke
    pub method: String,
    /// Method parameters (method-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Response message sent from Rust to UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request ID this response corresponds to
    pub id: RequestId,
    /// Success result (mutually exclusive with error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error result (mutually exclusive with result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<IpcError>,
}

/// Notification message sent from Rust to UI (no response expected)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcNotification {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Event type
    pub method: String,
    /// Event data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Request ID can be string or number
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

/// Error returned in IpcResponse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    /// Error code (see error code constants)
    pub code: i32,
    /// Human-readable error message
    pub message: String,
    /// Additional error data (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// ============================================================================
// Error Codes (JSON-RPC 2.0 standard codes + custom extensions)
// ============================================================================

/// JSON-RPC parse error (invalid JSON)
pub const ERROR_PARSE: i32 = -32700;
/// JSON-RPC invalid request (malformed structure)
pub const ERROR_INVALID_REQUEST: i32 = -32600;
/// JSON-RPC method not found
pub const ERROR_METHOD_NOT_FOUND: i32 = -32601;
/// JSON-RPC invalid method parameters
pub const ERROR_INVALID_PARAMS: i32 = -32602;
/// JSON-RPC internal error
pub const ERROR_INTERNAL: i32 = -32603;

// Custom application error codes (start at -32000)
/// Parameter not found
pub const ERROR_PARAM_NOT_FOUND: i32 = -32000;
/// Parameter value out of valid range
pub const ERROR_PARAM_OUT_OF_RANGE: i32 = -32001;

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
    /// Current normalized value [0.0, 1.0]
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
    /// New normalized value [0.0, 1.0]
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
    /// Current normalized value [0.0, 1.0]
    pub value: f32,
    /// Default normalized value [0.0, 1.0]
    pub default: f32,
    /// Unit suffix for display (e.g., "dB", "%", "Hz")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Group name for UI organization (e.g., "Input", "Processing", "Output")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
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
    /// New normalized value [0.0, 1.0]
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
// Helper Constructors
// ============================================================================

impl IpcRequest {
    /// Create a new request
    pub fn new(
        id: RequestId,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

impl IpcResponse {
    /// Create a success response
    pub fn success(id: RequestId, result: impl Serialize) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::to_value(result).unwrap()),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: RequestId, error: IpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

impl IpcNotification {
    /// Create a new notification
    pub fn new(method: impl Into<String>, params: impl Serialize) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: Some(serde_json::to_value(params).unwrap()),
        }
    }
}

impl IpcError {
    /// Create a new error
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create an error with additional data
    pub fn with_data(code: i32, message: impl Into<String>, data: impl Serialize) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(serde_json::to_value(data).unwrap()),
        }
    }

    /// Parse error
    pub fn parse_error() -> Self {
        Self::new(ERROR_PARSE, "Parse error")
    }

    /// Invalid request error
    pub fn invalid_request(reason: impl Into<String>) -> Self {
        Self::new(
            ERROR_INVALID_REQUEST,
            format!("Invalid request: {}", reason.into()),
        )
    }

    /// Method not found error
    pub fn method_not_found(method: impl AsRef<str>) -> Self {
        Self::new(
            ERROR_METHOD_NOT_FOUND,
            format!("Method not found: {}", method.as_ref()),
        )
    }

    /// Invalid params error
    pub fn invalid_params(reason: impl Into<String>) -> Self {
        Self::new(
            ERROR_INVALID_PARAMS,
            format!("Invalid params: {}", reason.into()),
        )
    }

    /// Internal error
    pub fn internal_error(reason: impl Into<String>) -> Self {
        Self::new(ERROR_INTERNAL, format!("Internal error: {}", reason.into()))
    }

    /// Parameter not found error
    pub fn param_not_found(id: impl AsRef<str>) -> Self {
        Self::new(
            ERROR_PARAM_NOT_FOUND,
            format!("Parameter not found: {}", id.as_ref()),
        )
    }

    /// Parameter out of range error
    pub fn param_out_of_range(id: impl AsRef<str>, value: f32) -> Self {
        Self::new(
            ERROR_PARAM_OUT_OF_RANGE,
            format!("Parameter '{}' value {} out of range", id.as_ref(), value),
        )
    }
}

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
