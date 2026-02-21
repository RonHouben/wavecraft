use serde::{Deserialize, Serialize};

use super::IpcError;

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
    /// Try to create a success response.
    pub fn try_success(id: RequestId, result: impl Serialize) -> serde_json::Result<Self> {
        Ok(Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::to_value(result)?),
            error: None,
        })
    }

    /// Create a success response
    pub fn success(id: RequestId, result: impl Serialize) -> Self {
        let id_for_fallback = id.clone();
        match Self::try_success(id, result) {
            Ok(response) => response,
            Err(err) => Self::error(
                id_for_fallback,
                IpcError::internal_error(format!("Failed to serialize success response: {err}")),
            ),
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
    /// Try to create a new notification.
    pub fn try_new(method: impl Into<String>, params: impl Serialize) -> serde_json::Result<Self> {
        Ok(Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: Some(serde_json::to_value(params)?),
        })
    }

    /// Create a new notification
    pub fn new(method: impl Into<String>, params: impl Serialize) -> Self {
        let method = method.into();
        match Self::try_new(method.clone(), params) {
            Ok(notification) => notification,
            Err(_) => Self {
                jsonrpc: "2.0".to_string(),
                method,
                params: None,
            },
        }
    }
}
