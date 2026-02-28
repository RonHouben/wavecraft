/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */

// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
/**
 * @wavecraft/core - Core SDK for Wavecraft WebView ↔ Rust communication
 *
 * Public exports for application code.
 */
// =============================================================================
// Environment Detection
// =============================================================================
export { isBrowserEnvironment, isWebViewEnvironment } from './utils/environment';

// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
// =============================================================================
// Types
// =============================================================================
export type {
    // IPC types
    AudioDiagnostic,
    AudioDiagnosticCode,
    AudioRuntimePhase,
    AudioRuntimeStatus,
    GetAudioStatusResult,
    IpcError,
    IpcNotification,
    IpcRequest,
    IpcResponse,
    RequestId
} from './types/ipc';

export type {
    GetAllParametersResult,
    GetParameterParams,
    GetParameterResult,
    ParameterChangedNotification,
    ParameterId,
    ParameterIdMap,
    // Parameter types
    ParameterInfo,
    ParameterType,
    ParameterValue,
    ParameterVariant,
    SetParameterParams,
    SetParameterResult
} from './types/parameters';

export type {
    // Processor types
    ProcessorId,
    ProcessorIdMap
} from './types/processors';

export type {
    BypassProcessorId,
    LevelProcessorId,
    ParameterIdForProcessorSuffix,
    PassthroughBypassParameterId,
    PassthroughParameterIds,
    PassthroughProcessorId,
    ProcessorIdForParameterSuffix,
    SoftClipBypassParameterId,
    SoftClipDriveDbParameterId,
    SoftClipMixParameterId,
    SoftClipOutputDbParameterId,
    SoftClipParameterIds,
    SoftClipProcessorId,
    SoftClipToneParameterId,
    TestToneBypassParameterId,
    TestToneEnabledParameterId,
    TestToneFrequencyParameterId,
    TestToneLevelParameterId,
    TestToneParameterIds,
    TestToneProcessorId,
    ToneFilterBypassParameterId,
    ToneFilterCutoffHzParameterId,
    ToneFilterModeParameterId,
    ToneFilterParameterIds,
    ToneFilterProcessorId,
    ToneFilterResonanceQParameterId
} from './types/processor-parameter-ids';

export type {
    GetMeterFrameResult,
    // Metering types
    MeterFrame
} from './types/metering';

export type {
    GetOscilloscopeFrameResult,
    OscilloscopeChannelView,
    OscilloscopeFrame,
    // Oscilloscope types
    OscilloscopeTriggerMode
} from './types/oscilloscope';

export type {
    AudioSignalTapId,
    // Signal chain order types
    SignalChainOrder,
    SlotType
} from './types/signal-chain';

// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
// IPC error codes
export {
    ERROR_INTERNAL,
    ERROR_INVALID_PARAMS,
    ERROR_INVALID_REQUEST,
    ERROR_METHOD_NOT_FOUND,
    ERROR_PARAM_NOT_FOUND,
    ERROR_PARAM_OUT_OF_RANGE,
    ERROR_PARSE,
    METHOD_GET_AUDIO_STATUS,
    METHOD_GET_OSCILLOSCOPE_FRAME,
    NOTIFICATION_AUDIO_STATUS_CHANGED,
    isAudioRuntimeStatus,
    isIpcError,
    isIpcNotification,
    isIpcResponse
} from './types/ipc';

// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
// IPC method names
export {
    METHOD_GET_ALL_PARAMETERS,
    METHOD_GET_PARAMETER,
    METHOD_SET_PARAMETER,
    NOTIFICATION_PARAMETER_CHANGED
} from './types/parameters';

export { IpcEvents, IpcMethods } from './ipc/constants';
export type { IpcEvent, IpcMethod } from './ipc/constants';

// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
// =============================================================================
// Core Classes (advanced use)
// =============================================================================
export { IpcBridge } from './ipc/IpcBridge';
export { ParameterClient } from './ipc/ParameterClient';
export { SignalChainOrderClient } from './ipc/SignalChainOrderClient';
export type {
    GetSignalChainOrderResult,
    SetSignalChainOrderParams,
    SignalChainOrderChangedNotification
} from './ipc/SignalChainOrderClient';

// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
// =============================================================================
// React Hooks (primary API)
// =============================================================================
export { WavecraftProvider } from './context/WavecraftProvider';
export type { WavecraftProviderProps } from './context/WavecraftProvider';

export { useParameter } from './hooks/useParameter';
export type { UseParameterResult } from './hooks/useParameter';

export { useAllParameters } from './hooks/useAllParameters';
export type { UseAllParametersResult } from './hooks/useAllParameters';

export { useParametersForProcessor } from './hooks/useAllParameterFor';
export type { UseParametersForProcessorResult } from './hooks/useAllParameterFor';

export { useConnectionStatus } from './hooks/useConnectionStatus';
export type { ConnectionStatus, TransportType } from './hooks/useConnectionStatus';

export { useLatencyMonitor } from './hooks/useLatencyMonitor';
export type { UseLatencyMonitorResult } from './hooks/useLatencyMonitor';

export { useAudioStatus } from './hooks/useAudioStatus';
export type { UseAudioStatusResult } from './hooks/useAudioStatus';
export { useMeterFrame } from './hooks/useMeterFrame';
export {
    getMeterClipWarningIntensity,
    getMeterSignalIntensity,
    getMeterSignalLevel,
    useMeterSignalActivity
} from './hooks/useMeterSignalActivity';
export type {
    MeterClipWarningRange,
    MeterSignalActivitySmoothing,
    MeterSignalActivityState,
    MeterSignalIntensityRange,
    UseMeterSignalActivityOptions
} from './hooks/useMeterSignalActivity';
export { useOscilloscopeFrame } from './hooks/useOscilloscopeFrame';

export { useAvailableProcessors } from './hooks/useAvailableProcessors';
export { useHasProcessorInSignalChain } from './hooks/useHasProcessor';
export { useProcessorBypass } from './hooks/useProcessorBypass';
export type { UseProcessorBypassResult } from './hooks/useProcessorBypass';
export { useRequestResize } from './hooks/useRequestResize';

export { useSignalChainOrder } from './hooks/useSignalChainOrder';
export type { UseSignalChainOrderResult } from './hooks/useSignalChainOrder';

export { requestResize, useWindowResizeSync } from './hooks/useWindowResizeSync';
export type { RequestResizeParams, RequestResizeResult } from './hooks/useWindowResizeSync';

// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
export {
    PROCESSOR_BYPASS_SUFFIX,
    getProcessorBypassParamId,
    isBypassParameterId
} from './processors/bypass';
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
// Runtime processor registry (used by generated processors module)
export { registerAvailableProcessors } from './processors/registry';

// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
// =============================================================================
// Metering API
// =============================================================================
export { getMeterFrame } from './meter-ipc';
export { getOscilloscopeFrame } from './oscilloscope-ipc';
export { dbToLinear, linearToDb } from './utils/audio-math';

// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
// =============================================================================
// Logger
// =============================================================================
export { LogLevel, Logger, logger } from './logger/Logger';
export type { LogContext } from './logger/Logger';

// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
export { NativeTransport, WebSocketTransport } from './transports';
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
// =============================================================================
// Transports (advanced use)
// =============================================================================
export type { NotificationCallback, Transport } from './transports';
