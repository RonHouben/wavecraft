use serde::{Deserialize, Serialize};

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

impl IpcError {
    /// Create a new error
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Try to create an error with additional data.
    pub fn try_with_data(
        code: i32,
        message: impl Into<String>,
        data: impl Serialize,
    ) -> serde_json::Result<Self> {
        Ok(Self {
            code,
            message: message.into(),
            data: Some(serde_json::to_value(data)?),
        })
    }

    /// Create an error with additional data
    pub fn with_data(code: i32, message: impl Into<String>, data: impl Serialize) -> Self {
        let message = message.into();
        match Self::try_with_data(code, message.clone(), data) {
            Ok(error) => error,
            Err(_) => Self {
                code,
                message,
                data: None,
            },
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
