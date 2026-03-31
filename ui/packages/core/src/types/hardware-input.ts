import { IpcEvents, IpcMethods } from '../ipc/constants';

export interface HardwareInputDeviceOption {
  id: string;
  label: string;
  channel_count: number;
  description?: string;
}

export interface HardwareInputChannelOption {
  id: string;
  label: string;
  description?: string;
}

export interface GetHardwareInputSelectionResult {
  selected_device_id: string | null;
  available_devices: HardwareInputDeviceOption[];
  selected_channel_id: string | null;
  available_channels: HardwareInputChannelOption[];
}

export interface SetHardwareInputSelectionParams {
  selected_device_id?: string;
  selected_channel_id?: string;
}

export type SetHardwareInputSelectionResult = Record<string, never>;

export interface HardwareInputSelectionChangedNotification {
  selected_device_id: string | null;
  available_devices: HardwareInputDeviceOption[];
  selected_channel_id: string | null;
  available_channels: HardwareInputChannelOption[];
}

export const METHOD_GET_HARDWARE_INPUT_SELECTION = IpcMethods.GET_HARDWARE_INPUT_SELECTION;
export const METHOD_SET_HARDWARE_INPUT_SELECTION = IpcMethods.SET_HARDWARE_INPUT_SELECTION;
export const NOTIFICATION_HARDWARE_INPUT_SELECTION_CHANGED =
  IpcEvents.HARDWARE_INPUT_SELECTION_CHANGED;
