//! Protocol crate - shared parameter definitions and contracts.
//!
//! This crate contains the canonical definitions for all parameters,
//! ensuring consistency between DSP, plugin, and UI layers.

pub mod ipc;
pub mod params;

pub use params::{
    PARAM_SPECS, ParamId, ParamSet, ParamSpec, VstKitParamId, VstKitParams, db_to_linear,
};

// Re-export key IPC types for convenience
pub use ipc::{
    ERROR_INTERNAL, ERROR_INVALID_PARAMS, ERROR_INVALID_REQUEST, ERROR_METHOD_NOT_FOUND,
    ERROR_PARAM_NOT_FOUND, ERROR_PARAM_OUT_OF_RANGE, ERROR_PARSE, GetAllParametersResult,
    GetMeterFrameResult, GetParameterParams, GetParameterResult, IpcError, IpcNotification,
    IpcRequest, IpcResponse, METHOD_GET_ALL_PARAMETERS, METHOD_GET_METER_FRAME,
    METHOD_GET_PARAMETER, METHOD_REQUEST_RESIZE, METHOD_SET_PARAMETER, MeterFrame,
    NOTIFICATION_PARAMETER_CHANGED, ParameterChangedNotification, ParameterInfo, ParameterType,
    RequestId, RequestResizeParams, RequestResizeResult, SetParameterParams, SetParameterResult,
};
