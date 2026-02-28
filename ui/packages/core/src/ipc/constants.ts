export const IpcMethods = {
  GET_PARAMETER: 'getParameter',
  SET_PARAMETER: 'setParameter',
  GET_ALL_PARAMETERS: 'getAllParameters',
  GET_METER_FRAME: 'getMeterFrame',
  GET_AUDIO_STATUS: 'getAudioStatus',
  GET_OSCILLOSCOPE_FRAME: 'getOscilloscopeFrame',
  REQUEST_RESIZE: 'requestResize',
  PING: 'ping',
  GET_SIGNAL_CHAIN_ORDER: 'getSignalChainOrder',
  SET_SIGNAL_CHAIN_ORDER: 'setSignalChainOrder',
} as const;

export type IpcMethod = (typeof IpcMethods)[keyof typeof IpcMethods];

export const IpcEvents = {
  AUDIO_STATUS_CHANGED: 'audioStatusChanged',
  PARAM_UPDATE: 'paramUpdate',
  METER_FRAME: 'meterFrame',
  PARAMETER_CHANGED: 'parameterChanged',
  PARAMETERS_CHANGED: 'parametersChanged',
  SIGNAL_CHAIN_ORDER_CHANGED: 'signalChainOrderChanged',
} as const;

export type IpcEvent = (typeof IpcEvents)[keyof typeof IpcEvents];
