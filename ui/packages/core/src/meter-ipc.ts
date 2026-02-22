/**
 * Meter polling API for audio visualization (IPC-based)
 */

import { IpcBridge } from './ipc/IpcBridge';
import { IpcMethods } from './ipc/constants';
import type { MeterFrame, GetMeterFrameResult } from './types/metering';

/**
 * Get the latest meter frame from the audio engine
 */
export async function getMeterFrame(): Promise<MeterFrame | null> {
  const bridge = IpcBridge.getInstance();
  const result = await bridge.invoke<GetMeterFrameResult>(IpcMethods.GET_METER_FRAME);
  return result.frame;
}
