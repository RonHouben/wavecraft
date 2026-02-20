//! Error types for the IPC bridge layer

use thiserror::Error;
use wavecraft_protocol::IpcError;

/// Bridge-specific errors that occur during IPC handling
#[derive(Debug, Error)]
pub enum BridgeError {
    /// JSON parsing failed
    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Parameter not found in host
    #[error("Parameter not found: {0}")]
    ParameterNotFound(String),

    /// Parameter value out of valid range
    #[error("Parameter value out of range: {id} = {value}")]
    ParameterOutOfRange { id: String, value: f32 },

    /// Method not supported by handler
    #[error("Unknown method: {0}")]
    UnknownMethod(String),

    /// Invalid parameters for method
    #[error("Invalid params for method {method}: {reason}")]
    InvalidParams { method: String, reason: String },

    /// Internal error in bridge logic
    #[error("Internal bridge error: {0}")]
    Internal(String),
}

impl BridgeError {
    /// Convert BridgeError to IpcError from protocol
    pub fn to_ipc_error(&self) -> IpcError {
        match self {
            Self::JsonParse(_) => IpcError::parse_error(),
            Self::ParameterNotFound(id) => IpcError::param_not_found(id),
            Self::ParameterOutOfRange { id, value } => IpcError::param_out_of_range(id, *value),
            Self::UnknownMethod(method) => IpcError::method_not_found(method),
            Self::InvalidParams { reason, .. } => IpcError::invalid_params(reason),
            Self::Internal(reason) => IpcError::internal_error(reason),
        }
    }
}
