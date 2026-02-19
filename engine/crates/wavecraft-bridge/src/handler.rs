//! IPC request handler.

use crate::error::BridgeError;
use crate::host::ParameterHost;
use serde::Serialize;
use serde::de::DeserializeOwned;
use wavecraft_protocol::{
    GetAllParametersResult, GetAudioStatusResult, GetMeterFrameResult, GetOscilloscopeFrameResult,
    GetParameterParams, GetParameterResult, IpcRequest, IpcResponse, METHOD_GET_ALL_PARAMETERS,
    METHOD_GET_AUDIO_STATUS, METHOD_GET_METER_FRAME, METHOD_GET_OSCILLOSCOPE_FRAME,
    METHOD_GET_PARAMETER, METHOD_REQUEST_RESIZE, METHOD_SET_PARAMETER, RequestId,
    RequestResizeParams, RequestResizeResult, SetParameterParams, SetParameterResult,
};

/// IPC message handler that dispatches requests to a ParameterHost
pub struct IpcHandler<H: ParameterHost> {
    host: H,
}

impl<H: ParameterHost> IpcHandler<H> {
    /// Create a new IPC handler with the given parameter host
    pub fn new(host: H) -> Self {
        Self { host }
    }

    /// Handle an incoming IPC request and produce a response
    ///
    /// This is the main entry point for processing messages from the UI.
    /// It dispatches to appropriate handlers based on the method name.
    pub fn handle_request(&self, request: IpcRequest) -> IpcResponse {
        let result = match request.method.as_str() {
            METHOD_GET_PARAMETER => self.handle_get_parameter(&request),
            METHOD_SET_PARAMETER => self.handle_set_parameter(&request),
            METHOD_GET_ALL_PARAMETERS => self.handle_get_all_parameters(&request),
            METHOD_GET_METER_FRAME => self.handle_get_meter_frame(&request),
            METHOD_GET_OSCILLOSCOPE_FRAME => self.handle_get_oscilloscope_frame(&request),
            METHOD_GET_AUDIO_STATUS => self.handle_get_audio_status(&request),
            METHOD_REQUEST_RESIZE => self.handle_request_resize(&request),
            "ping" => self.handle_ping(&request),
            _ => Err(BridgeError::UnknownMethod(request.method.clone())),
        };

        match result {
            Ok(response) => response,
            Err(err) => IpcResponse::error(request.id, err.to_ipc_error()),
        }
    }

    /// Handle a raw JSON string request
    ///
    /// Convenience method that parses JSON and dispatches to handle_request.
    pub fn handle_json(&self, json: &str) -> String {
        // Parse request
        let request: IpcRequest = match serde_json::from_str(json) {
            Ok(req) => req,
            Err(_e) => {
                // Can't extract ID from malformed request, use a synthetic ID.
                let response = IpcResponse::error(
                    RequestId::Number(0),
                    wavecraft_protocol::IpcError::parse_error(),
                );
                // IpcResponse serialization is infallible: all fields are simple types
                // (RequestId, Option<Value>, Option<IpcError>) that serde_json always handles
                return serde_json::to_string(&response)
                    .expect("IpcResponse serialization is infallible");
            }
        };

        // Handle request
        let response = self.handle_request(request);

        // Serialize response - infallible for well-typed IpcResponse
        serde_json::to_string(&response).expect("IpcResponse serialization is infallible")
    }

    fn parse_required_params<T>(
        &self,
        request: &IpcRequest,
        method: &'static str,
    ) -> Result<T, BridgeError>
    where
        T: DeserializeOwned,
    {
        match &request.params {
            Some(value) => Ok(serde_json::from_value(value.clone())?),
            None => Err(BridgeError::InvalidParams {
                method: method.to_string(),
                reason: "Missing params".to_string(),
            }),
        }
    }

    // ------------------------------------------------------------------------
    // Method Handlers
    // ------------------------------------------------------------------------

    fn handle_get_parameter(&self, request: &IpcRequest) -> Result<IpcResponse, BridgeError> {
        let params: GetParameterParams =
            self.parse_required_params(request, METHOD_GET_PARAMETER)?;

        // Get parameter from host
        let param_info = self
            .host
            .get_parameter(&params.id)
            .ok_or_else(|| BridgeError::ParameterNotFound(params.id.clone()))?;

        // Build result
        let result = GetParameterResult {
            id: param_info.id,
            value: param_info.value,
        };

        Ok(IpcResponse::success(request.id.clone(), result))
    }

    fn handle_set_parameter(&self, request: &IpcRequest) -> Result<IpcResponse, BridgeError> {
        let params: SetParameterParams =
            self.parse_required_params(request, METHOD_SET_PARAMETER)?;

        // Set parameter
        self.host.set_parameter(&params.id, params.value)?;

        // Build result (empty success)
        Ok(IpcResponse::success(
            request.id.clone(),
            SetParameterResult {},
        ))
    }

    fn handle_get_all_parameters(&self, request: &IpcRequest) -> Result<IpcResponse, BridgeError> {
        let parameters = self.host.get_all_parameters();

        let result = GetAllParametersResult { parameters };

        Ok(IpcResponse::success(request.id.clone(), result))
    }

    fn handle_get_meter_frame(&self, request: &IpcRequest) -> Result<IpcResponse, BridgeError> {
        // Get meter frame from host
        let frame = self.host.get_meter_frame();

        let result = GetMeterFrameResult { frame };

        Ok(IpcResponse::success(request.id.clone(), result))
    }

    fn handle_get_oscilloscope_frame(
        &self,
        request: &IpcRequest,
    ) -> Result<IpcResponse, BridgeError> {
        let frame = self.host.get_oscilloscope_frame();

        let result = GetOscilloscopeFrameResult { frame };

        Ok(IpcResponse::success(request.id.clone(), result))
    }

    fn handle_request_resize(&self, request: &IpcRequest) -> Result<IpcResponse, BridgeError> {
        let params: RequestResizeParams =
            self.parse_required_params(request, METHOD_REQUEST_RESIZE)?;

        // Request resize from host
        let accepted = self.host.request_resize(params.width, params.height);

        let result = RequestResizeResult { accepted };

        Ok(IpcResponse::success(request.id.clone(), result))
    }

    fn handle_get_audio_status(&self, request: &IpcRequest) -> Result<IpcResponse, BridgeError> {
        let result = GetAudioStatusResult {
            status: self.host.get_audio_status(),
        };

        Ok(IpcResponse::success(request.id.clone(), result))
    }

    fn handle_ping(&self, request: &IpcRequest) -> Result<IpcResponse, BridgeError> {
        // Simple ping/pong for testing connectivity
        #[derive(Serialize)]
        struct PingResult {
            pong: bool,
        }

        Ok(IpcResponse::success(
            request.id.clone(),
            PingResult { pong: true },
        ))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use wavecraft_protocol::{
        AudioRuntimePhase, AudioRuntimeStatus, MeterFrame, OscilloscopeFrame, ParameterInfo,
        ParameterType, RequestId,
    };

    // Mock ParameterHost for testing
    struct MockHost {
        params: Vec<ParameterInfo>,
    }

    impl MockHost {
        fn new() -> Self {
            Self {
                params: vec![
                    ParameterInfo {
                        id: "gain".to_string(),
                        name: "Gain".to_string(),
                        param_type: ParameterType::Float,
                        value: 0.5,
                        default: 0.7,
                        min: 0.0,
                        max: 1.0,
                        unit: Some("dB".to_string()),
                        group: None,
                        variants: None,
                    },
                    ParameterInfo {
                        id: "bypass".to_string(),
                        name: "Bypass".to_string(),
                        param_type: ParameterType::Bool,
                        value: 0.0,
                        default: 0.0,
                        min: 0.0,
                        max: 1.0,
                        unit: None,
                        group: None,
                        variants: None,
                    },
                ],
            }
        }
    }

    impl ParameterHost for MockHost {
        fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
            self.params.iter().find(|p| p.id == id).cloned()
        }

        fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
            let Some(param) = self.params.iter().find(|p| p.id == id) else {
                return Err(BridgeError::ParameterNotFound(id.to_string()));
            };

            if !(param.min..=param.max).contains(&value) {
                return Err(BridgeError::ParameterOutOfRange {
                    id: id.to_string(),
                    value,
                });
            }

            // In real implementation, would update atomic value
            Ok(())
        }

        fn get_all_parameters(&self) -> Vec<ParameterInfo> {
            self.params.clone()
        }

        fn get_meter_frame(&self) -> Option<MeterFrame> {
            // Mock returns None
            None
        }

        fn get_oscilloscope_frame(&self) -> Option<OscilloscopeFrame> {
            None
        }

        fn request_resize(&self, _width: u32, _height: u32) -> bool {
            // Mock always accepts resize requests
            true
        }

        fn get_audio_status(&self) -> Option<AudioRuntimeStatus> {
            Some(AudioRuntimeStatus {
                phase: AudioRuntimePhase::RunningFullDuplex,
                diagnostic: None,
                sample_rate: Some(44100.0),
                buffer_size: Some(512),
                updated_at_ms: 123,
            })
        }
    }

    #[test]
    fn test_get_parameter_success() {
        let handler = IpcHandler::new(MockHost::new());

        let request = IpcRequest::new(
            RequestId::Number(1),
            METHOD_GET_PARAMETER,
            Some(serde_json::json!({"id": "gain"})),
        );

        let response = handler.handle_request(request);

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result: GetParameterResult = serde_json::from_value(response.result.unwrap()).unwrap();
        assert_eq!(result.id, "gain");
        assert_eq!(result.value, 0.5);
    }

    #[test]
    fn test_get_parameter_not_found() {
        let handler = IpcHandler::new(MockHost::new());

        let request = IpcRequest::new(
            RequestId::Number(2),
            METHOD_GET_PARAMETER,
            Some(serde_json::json!({"id": "unknown"})),
        );

        let response = handler.handle_request(request);

        assert!(response.error.is_some());
        assert!(response.result.is_none());

        let error = response.error.unwrap();
        assert_eq!(error.code, wavecraft_protocol::ERROR_PARAM_NOT_FOUND);
    }

    #[test]
    fn test_set_parameter_success() {
        let handler = IpcHandler::new(MockHost::new());

        let request = IpcRequest::new(
            RequestId::Number(3),
            METHOD_SET_PARAMETER,
            Some(serde_json::json!({"id": "gain", "value": 0.8})),
        );

        let response = handler.handle_request(request);

        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_set_parameter_out_of_range() {
        let handler = IpcHandler::new(MockHost::new());

        let request = IpcRequest::new(
            RequestId::Number(4),
            METHOD_SET_PARAMETER,
            Some(serde_json::json!({"id": "gain", "value": 1.5})),
        );

        let response = handler.handle_request(request);

        assert!(response.error.is_some());
        assert!(response.result.is_none());

        let error = response.error.unwrap();
        assert_eq!(error.code, wavecraft_protocol::ERROR_PARAM_OUT_OF_RANGE);
    }

    #[test]
    fn test_unknown_method() {
        let handler = IpcHandler::new(MockHost::new());

        let request = IpcRequest::new(RequestId::Number(5), "unknownMethod", None);

        let response = handler.handle_request(request);

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, wavecraft_protocol::ERROR_METHOD_NOT_FOUND);
    }

    #[test]
    fn test_ping() {
        let handler = IpcHandler::new(MockHost::new());

        let request = IpcRequest::new(RequestId::String("ping-1".to_string()), "ping", None);

        let response = handler.handle_request(request);

        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_get_all_parameters() {
        let handler = IpcHandler::new(MockHost::new());

        let request = IpcRequest::new(RequestId::Number(6), METHOD_GET_ALL_PARAMETERS, None);

        let response = handler.handle_request(request);

        assert!(response.result.is_some());

        let result: GetAllParametersResult =
            serde_json::from_value(response.result.unwrap()).unwrap();
        assert_eq!(result.parameters.len(), 2);
    }

    #[test]
    fn test_handle_json() {
        let handler = IpcHandler::new(MockHost::new());

        let json = r#"{"jsonrpc":"2.0","id":1,"method":"getParameter","params":{"id":"gain"}}"#;
        let response_json = handler.handle_json(json);

        assert!(response_json.contains("\"result\""));
        assert!(!response_json.contains("\"error\""));
    }

    #[test]
    fn test_handle_json_parse_error() {
        let handler = IpcHandler::new(MockHost::new());

        let json = r#"{"invalid json"#;
        let response_json = handler.handle_json(json);

        assert!(response_json.contains("\"error\""));
    }

    #[test]
    fn test_get_audio_status() {
        let handler = IpcHandler::new(MockHost::new());

        let request = IpcRequest::new(RequestId::Number(7), METHOD_GET_AUDIO_STATUS, None);

        let response = handler.handle_request(request);
        assert!(response.result.is_some());

        let result: GetAudioStatusResult =
            serde_json::from_value(response.result.expect("audio status response should exist"))
                .expect("audio status result should deserialize");
        let status = result.status.expect("status should be present");
        assert_eq!(status.phase, AudioRuntimePhase::RunningFullDuplex);
    }

    #[test]
    fn test_get_oscilloscope_frame_none() {
        let handler = IpcHandler::new(MockHost::new());

        let request = IpcRequest::new(RequestId::Number(8), METHOD_GET_OSCILLOSCOPE_FRAME, None);

        let response = handler.handle_request(request);
        assert!(response.result.is_some());

        let result: GetOscilloscopeFrameResult =
            serde_json::from_value(response.result.expect("oscilloscope response should exist"))
                .expect("oscilloscope result should deserialize");
        assert!(result.frame.is_none());
    }
}
