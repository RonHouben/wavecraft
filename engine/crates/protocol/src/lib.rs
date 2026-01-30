//! Protocol crate - shared parameter definitions and contracts.
//!
//! This crate contains the canonical definitions for all parameters,
//! ensuring consistency between DSP, plugin, and UI layers.

pub mod ipc;
pub mod params;

pub use params::{db_to_linear, ParamId, ParamSpec, PARAM_SPECS};

// Re-export key IPC types for convenience
pub use ipc::{
    GetAllParametersResult, GetParameterParams, GetParameterResult, IpcError, IpcNotification,
    IpcRequest, IpcResponse, ParameterChangedNotification, ParameterInfo, ParameterType,
    RequestId, SetParameterParams, SetParameterResult, ERROR_INTERNAL, ERROR_INVALID_PARAMS,
    ERROR_INVALID_REQUEST, ERROR_METHOD_NOT_FOUND, ERROR_PARAM_NOT_FOUND,
    ERROR_PARAM_OUT_OF_RANGE, ERROR_PARSE, METHOD_GET_ALL_PARAMETERS, METHOD_GET_PARAMETER,
    METHOD_SET_PARAMETER, NOTIFICATION_PARAMETER_CHANGED,
};
