import { IpcEvents, IpcMethods } from '../ipc/constants';

export type InputSourceKind = 'hardwareInput' | 'testTone';

export interface InputSourceOption {
  id: InputSourceKind;
  label: string;
  description?: string;
}

export interface GetInputSourceResult {
  selected: InputSourceKind;
  available: InputSourceOption[];
}

export interface SetInputSourceParams {
  selected: InputSourceKind;
}

export type SetInputSourceResult = Record<string, never>;

export interface InputSourceChangedNotification {
  selected: InputSourceKind;
}

export const METHOD_GET_INPUT_SOURCE = IpcMethods.GET_INPUT_SOURCE;
export const METHOD_SET_INPUT_SOURCE = IpcMethods.SET_INPUT_SOURCE;
export const NOTIFICATION_INPUT_SOURCE_CHANGED = IpcEvents.INPUT_SOURCE_CHANGED;
