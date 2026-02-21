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

#[path = "ipc/envelope.rs"]
mod envelope;
#[path = "ipc/errors.rs"]
mod errors;
#[path = "ipc/methods.rs"]
mod methods;

pub use envelope::{IpcNotification, IpcRequest, IpcResponse, RequestId};
pub use errors::{
    ERROR_INTERNAL, ERROR_INVALID_PARAMS, ERROR_INVALID_REQUEST, ERROR_METHOD_NOT_FOUND,
    ERROR_PARAM_NOT_FOUND, ERROR_PARAM_OUT_OF_RANGE, ERROR_PARSE, IpcError,
};
pub use methods::{
    AudioDiagnostic, AudioDiagnosticCode, AudioRuntimePhase, AudioRuntimeStatus,
    GetAllParametersResult, GetAudioStatusResult, GetMeterFrameResult, GetOscilloscopeFrameResult,
    GetParameterParams, GetParameterResult, METHOD_GET_ALL_PARAMETERS, METHOD_GET_AUDIO_STATUS,
    METHOD_GET_METER_FRAME, METHOD_GET_OSCILLOSCOPE_FRAME, METHOD_GET_PARAMETER,
    METHOD_REGISTER_AUDIO, METHOD_REQUEST_RESIZE, METHOD_SET_PARAMETER, MeterFrame,
    MeterUpdateNotification, NOTIFICATION_AUDIO_STATUS_CHANGED, NOTIFICATION_METER_UPDATE,
    NOTIFICATION_PARAMETER_CHANGED, OscilloscopeChannelView, OscilloscopeFrame,
    OscilloscopeTriggerMode, ParameterChangedNotification, ParameterInfo, ParameterType,
    ProcessorInfo, RegisterAudioParams, RegisterAudioResult, RequestResizeParams,
    RequestResizeResult, SetParameterParams, SetParameterResult,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde::ser::Error as _;

    struct FailingSerialize;

    impl Serialize for FailingSerialize {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(S::Error::custom("intentional serialization failure"))
        }
    }

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

    #[test]
    fn try_success_returns_error_on_serialize_failure() {
        let result = IpcResponse::try_success(RequestId::Number(1), FailingSerialize);
        assert!(result.is_err());
    }

    #[test]
    fn success_does_not_panic_on_serialize_failure() {
        let response = IpcResponse::success(RequestId::Number(1), FailingSerialize);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn try_notification_returns_error_on_serialize_failure() {
        let result = IpcNotification::try_new(NOTIFICATION_PARAMETER_CHANGED, FailingSerialize);
        assert!(result.is_err());
    }

    #[test]
    fn notification_new_does_not_panic_on_serialize_failure() {
        let notification = IpcNotification::new(NOTIFICATION_PARAMETER_CHANGED, FailingSerialize);
        assert!(notification.params.is_none());
    }

    #[test]
    fn try_with_data_returns_error_on_serialize_failure() {
        let result = IpcError::try_with_data(ERROR_INTERNAL, "test", FailingSerialize);
        assert!(result.is_err());
    }

    #[test]
    fn with_data_does_not_panic_on_serialize_failure() {
        let error = IpcError::with_data(ERROR_INTERNAL, "test", FailingSerialize);
        assert!(error.data.is_none());
        assert_eq!(error.message, "test");
    }
}
