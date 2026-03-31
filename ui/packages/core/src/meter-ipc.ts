/**
 * Meter polling API for audio visualization (IPC-based)
 */

import { IpcBridge } from './ipc/IpcBridge';
import { IpcMethods } from './ipc/constants';
import type { GetMeterFrameResult, MeterFrame } from './types/metering';

/**
 * Get the latest meter frame from the audio engine
 */
export async function getMeterFrame(): Promise<MeterFrame | null> {
  const bridge = IpcBridge.getInstance();
  const result = await bridge.invoke<GetMeterFrameResult>(IpcMethods.GET_METER_FRAME);
  return result.frame;
}

/**
 * Get the latest Passthrough-local meter frame from the audio engine
 */
export async function getPassthroughMeterFrame(): Promise<MeterFrame | null> {
  const bridge = IpcBridge.getInstance();
  const result = await bridge.invoke<GetMeterFrameResult>(IpcMethods.GET_PASSTHROUGH_METER_FRAME);
  return result.frame;
}
