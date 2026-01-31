//! IPC request handler and parameter host trait

use crate::error::BridgeError;
use protocol::{
    GetAllParametersResult, GetMeterFrameResult, GetParameterParams, GetParameterResult,
    IpcRequest, IpcResponse, METHOD_GET_ALL_PARAMETERS, METHOD_GET_METER_FRAME,
    METHOD_GET_PARAMETER, METHOD_REQUEST_RESIZE, METHOD_SET_PARAMETER, MeterFrame, ParameterInfo,
    RequestId, RequestResizeParams, RequestResizeResult, SetParameterParams, SetParameterResult,
};
use serde::Serialize;

/// Trait for objects that store and manage parameters
///
/// This trait abstracts parameter storage, allowing the bridge to work with
/// both the desktop POC (atomic parameters) and future plugin hosts.
pub trait ParameterHost: Send + Sync {
    /// Get information about a single parameter
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo>;

    /// Set a parameter value (normalized [0.0, 1.0])
    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError>;

    /// Get all parameters with their current values and metadata
    fn get_all_parameters(&self) -> Vec<ParameterInfo>;

    /// Get the latest meter frame for UI visualization
    fn get_meter_frame(&self) -> Option<MeterFrame>;

    /// Request resize of the editor window
    ///
    /// Returns true if the host accepted the resize request.
    /// The host is free to reject or adjust the size.
    fn request_resize(&self, width: u32, height: u32) -> bool;
}

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
                // Can't extract ID from malformed request, use null
                let response =
                    IpcResponse::error(RequestId::Number(0), protocol::IpcError::parse_error());
                return serde_json::to_string(&response).unwrap();
            }
        };

        // Handle request
        let response = self.handle_request(request);

        // Serialize response
        serde_json::to_string(&response).unwrap()
    }

    // ------------------------------------------------------------------------
    // Method Handlers
    // ------------------------------------------------------------------------

    fn handle_get_parameter(&self, request: &IpcRequest) -> Result<IpcResponse, BridgeError> {
        // Parse params
        let params: GetParameterParams = match &request.params {
            Some(value) => serde_json::from_value(value.clone())?,
            None => {
                return Err(BridgeError::InvalidParams {
                    method: METHOD_GET_PARAMETER.to_string(),
                    reason: "Missing params".to_string(),
                });
            }
        };

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
        // Parse params
        let params: SetParameterParams = match &request.params {
            Some(value) => serde_json::from_value(value.clone())?,
            None => {
                return Err(BridgeError::InvalidParams {
                    method: METHOD_SET_PARAMETER.to_string(),
                    reason: "Missing params".to_string(),
                });
            }
        };

        // Validate range
        if !(0.0..=1.0).contains(&params.value) {
            return Err(BridgeError::ParameterOutOfRange {
                id: params.id.clone(),
                value: params.value,
            });
        }

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

    fn handle_request_resize(&self, request: &IpcRequest) -> Result<IpcResponse, BridgeError> {
        // Parse params
        let params: RequestResizeParams = match &request.params {
            Some(value) => serde_json::from_value(value.clone())?,
            None => {
                return Err(BridgeError::InvalidParams {
                    method: METHOD_REQUEST_RESIZE.to_string(),
                    reason: "Missing params".to_string(),
                });
            }
        };

        // Request resize from host
        let accepted = self.host.request_resize(params.width, params.height);

        let result = RequestResizeResult { accepted };

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
    use protocol::{ParameterType, RequestId};

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
                        unit: Some("dB".to_string()),
                    },
                    ParameterInfo {
                        id: "bypass".to_string(),
                        name: "Bypass".to_string(),
                        param_type: ParameterType::Bool,
                        value: 0.0,
                        default: 0.0,
                        unit: None,
                    },
                ],
            }
        }
    }

    impl ParameterHost for MockHost {
        fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
            self.params.iter().find(|p| p.id == id).cloned()
        }

        fn set_parameter(&self, id: &str, _value: f32) -> Result<(), BridgeError> {
            // Verify parameter exists
            if !self.params.iter().any(|p| p.id == id) {
                return Err(BridgeError::ParameterNotFound(id.to_string()));
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

        fn request_resize(&self, _width: u32, _height: u32) -> bool {
            // Mock always accepts resize requests
            true
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
        assert_eq!(error.code, protocol::ERROR_PARAM_NOT_FOUND);
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
        assert_eq!(error.code, protocol::ERROR_PARAM_OUT_OF_RANGE);
    }

    #[test]
    fn test_unknown_method() {
        let handler = IpcHandler::new(MockHost::new());

        let request = IpcRequest::new(RequestId::Number(5), "unknownMethod", None);

        let response = handler.handle_request(request);

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, protocol::ERROR_METHOD_NOT_FOUND);
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
}
