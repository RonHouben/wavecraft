//! Protocol crate - shared parameter definitions and contracts.
//!
//! This crate contains the canonical definitions for all parameters,
//! ensuring consistency between DSP, plugin, and UI layers.

pub mod dev_audio_ffi;
pub mod ipc;
pub mod macros;
pub mod params;

pub use params::{
    PARAM_SPECS, ParamId, ParamSet, ParamSpec, WavecraftParamId, WavecraftParams, db_to_linear,
};

// Re-export dev audio FFI types for convenience
pub use dev_audio_ffi::{DEV_PROCESSOR_SYMBOL, DEV_PROCESSOR_VTABLE_VERSION, DevProcessorVTable};

// Re-export key IPC types for convenience
pub use ipc::{
    AudioDiagnostic, AudioDiagnosticCode, AudioRuntimePhase, AudioRuntimeStatus, ERROR_INTERNAL,
    ERROR_INVALID_PARAMS, ERROR_INVALID_REQUEST, ERROR_METHOD_NOT_FOUND, ERROR_PARAM_NOT_FOUND,
    ERROR_PARAM_OUT_OF_RANGE, ERROR_PARSE, GetAllParametersResult, GetAudioStatusResult,
    GetMeterFrameResult, GetOscilloscopeFrameResult, GetParameterParams, GetParameterResult,
    IpcError, IpcNotification, IpcRequest, IpcResponse, METHOD_GET_ALL_PARAMETERS,
    METHOD_GET_AUDIO_STATUS, METHOD_GET_METER_FRAME, METHOD_GET_OSCILLOSCOPE_FRAME,
    METHOD_GET_PARAMETER, METHOD_REGISTER_AUDIO, METHOD_REQUEST_RESIZE, METHOD_SET_PARAMETER,
    MeterFrame, MeterUpdateNotification, NOTIFICATION_AUDIO_STATUS_CHANGED,
    NOTIFICATION_METER_UPDATE, NOTIFICATION_PARAMETER_CHANGED, OscilloscopeChannelView,
    OscilloscopeFrame, OscilloscopeTriggerMode, ParameterChangedNotification, ParameterInfo,
    ParameterType, RegisterAudioParams, RegisterAudioResult, RequestId, RequestResizeParams,
    RequestResizeResult, SetParameterParams, SetParameterResult,
};
