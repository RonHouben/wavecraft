/**
 * Meter polling API for audio visualization (IPC-based)
 */

import { IpcBridge } from './IpcBridge';
import type { MeterFrame, GetMeterFrameResult } from './meters';

/**
 * Get the latest meter frame from the audio engine
 */
export async function getMeterFrame(): Promise<MeterFrame | null> {
  const bridge = IpcBridge.getInstance();
  const result = await bridge.invoke<GetMeterFrameResult>('getMeterFrame');
  return result.frame;
}
