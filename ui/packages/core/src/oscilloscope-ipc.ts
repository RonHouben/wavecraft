/**
 * Oscilloscope polling API for waveform visualization (IPC-based)
 */

import { IpcBridge } from './ipc/IpcBridge';
import { METHOD_GET_OSCILLOSCOPE_FRAME } from './types/ipc';
import type { GetOscilloscopeFrameResult, OscilloscopeFrame } from './types/oscilloscope';

/**
 * Get the latest oscilloscope frame from the audio engine
 */
export async function getOscilloscopeFrame(): Promise<OscilloscopeFrame | null> {
  const bridge = IpcBridge.getInstance();
  const result = await bridge.invoke<GetOscilloscopeFrameResult>(METHOD_GET_OSCILLOSCOPE_FRAME);
  return result.frame;
}
