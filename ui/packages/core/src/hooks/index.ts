/**
 * Hooks - React integration for Wavecraft SDK
 *
 * All hooks are exported from this barrel file for clean imports.
 */

// Parameter hooks
export { useParameter } from './useParameter';
export type { UseParameterResult } from './useParameter';

export { useAllParameters } from './useAllParameters';
export type { UseAllParametersResult } from './useAllParameters';

export { useParameterGroups } from './useParameterGroups';
export type { ParameterGroup } from './useParameterGroups';

// Connection & diagnostics
export { useConnectionStatus } from './useConnectionStatus';
export type { ConnectionStatus, TransportType } from './useConnectionStatus';

export { useLatencyMonitor } from './useLatencyMonitor';
export type { UseLatencyMonitorResult } from './useLatencyMonitor';

// Metering
export { useMeterFrame } from './useMeterFrame';

// Window resize
export { requestResize, useWindowResizeSync } from './useWindowResizeSync';
export type { RequestResizeParams, RequestResizeResult } from './useWindowResizeSync';

export { useRequestResize } from './useRequestResize';
