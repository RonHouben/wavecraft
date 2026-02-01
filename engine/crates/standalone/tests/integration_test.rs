//! Integration tests for IPC communication

use bridge::{IpcHandler, ParameterHost};
use vstkit_protocol::{
    IpcRequest, METHOD_GET_ALL_PARAMETERS, METHOD_GET_PARAMETER, METHOD_SET_PARAMETER, RequestId,
};
use standalone::AppState;

#[test]
fn test_get_all_parameters_integration() {
    let state = AppState::new();
    let handler = IpcHandler::new(state);

    let request = IpcRequest::new(RequestId::Number(1), METHOD_GET_ALL_PARAMETERS, None);

    let response = handler.handle_request(request);

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result: vstkit_protocol::GetAllParametersResult =
        serde_json::from_value(response.result.unwrap()).unwrap();

    assert_eq!(result.parameters.len(), 3);
    assert!(result.parameters.iter().any(|p| p.id == "gain"));
    assert!(result.parameters.iter().any(|p| p.id == "bypass"));
    assert!(result.parameters.iter().any(|p| p.id == "mix"));
}

#[test]
fn test_get_parameter_integration() {
    let state = AppState::new();
    let handler = IpcHandler::new(state);

    let request = IpcRequest::new(
        RequestId::Number(2),
        METHOD_GET_PARAMETER,
        Some(serde_json::json!({"id": "gain"})),
    );

    let response = handler.handle_request(request);

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result: vstkit_protocol::GetParameterResult =
        serde_json::from_value(response.result.unwrap()).unwrap();

    assert_eq!(result.id, "gain");
    assert_eq!(result.value, 0.7); // Default value
}

#[test]
fn test_set_parameter_integration() {
    let state = AppState::new();
    let handler = IpcHandler::new(state.clone());

    // Set parameter
    let request = IpcRequest::new(
        RequestId::Number(3),
        METHOD_SET_PARAMETER,
        Some(serde_json::json!({"id": "gain", "value": 0.5})),
    );

    let response = handler.handle_request(request);
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    // Verify parameter was set
    let param = state.get_parameter("gain").unwrap();
    assert_eq!(param.value, 0.5);
}

#[test]
fn test_json_roundtrip() {
    let state = AppState::new();
    let handler = IpcHandler::new(state);

    let request_json = r#"{"jsonrpc":"2.0","id":1,"method":"getParameter","params":{"id":"mix"}}"#;

    let response_json = handler.handle_json(request_json);

    // Parse response
    let response: vstkit_protocol::IpcResponse = serde_json::from_str(&response_json).unwrap();

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result: vstkit_protocol::GetParameterResult =
        serde_json::from_value(response.result.unwrap()).unwrap();

    assert_eq!(result.id, "mix");
    assert_eq!(result.value, 1.0); // Default value
}

#[test]
fn test_ping_integration() {
    let state = AppState::new();
    let handler = IpcHandler::new(state);

    let request = IpcRequest::new(RequestId::String("ping-1".to_string()), "ping", None);

    let response = handler.handle_request(request);

    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_parameter_value_validation() {
    let state = AppState::new();
    let handler = IpcHandler::new(state);

    // Try to set out of range value
    let request = IpcRequest::new(
        RequestId::Number(4),
        METHOD_SET_PARAMETER,
        Some(serde_json::json!({"id": "gain", "value": 1.5})),
    );

    let response = handler.handle_request(request);

    assert!(response.error.is_some());
    assert!(response.result.is_none());

    let error = response.error.unwrap();
    assert_eq!(error.code, vstkit_protocol::ERROR_PARAM_OUT_OF_RANGE);
}
