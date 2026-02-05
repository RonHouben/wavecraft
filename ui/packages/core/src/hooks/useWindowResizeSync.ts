/**
 * useWindowResizeSync - Automatic window resize sync to host
 */

import { useEffect } from 'react';
import { IpcBridge } from '../ipc/IpcBridge';
import { logger } from '../logger';

export interface RequestResizeParams {
  width: number;
  height: number;
}

export interface RequestResizeResult {
  accepted: boolean;
}

/**
 * Request resize of the editor window
 *
 * @param width - Desired width in logical pixels
 * @param height - Desired height in logical pixels
 * @returns Promise that resolves to true if accepted, false if rejected
 *
 * @example
 * ```ts
 * const accepted = await requestResize(1024, 768);
 * if (accepted) {
 *   console.log('Resize accepted by host');
 * } else {
 *   console.warn('Resize rejected by host');
 * }
 * ```
 */
export async function requestResize(width: number, height: number): Promise<boolean> {
  const bridge = IpcBridge.getInstance();

  const result = await bridge.invoke<RequestResizeResult>('requestResize', { width, height });

  return result.accepted;
}

/**
 * Hook that automatically syncs window resize events to the host
 *
 * This hook listens for browser window resize events and notifies the host
 * DAW of size changes. Useful when the user resizes the plugin window via
 * the DAW's window controls or edge dragging.
 *
 * @example
 * ```tsx
 * function App() {
 *   // Automatically sync window size changes to host
 *   useWindowResizeSync();
 *
 *   return <div>Plugin UI</div>;
 * }
 * ```
 */
export function useWindowResizeSync(): void {
  useEffect(() => {
    const handleResize = (): void => {
      const width = window.innerWidth;
      const height = window.innerHeight;

      requestResize(width, height).catch((err) => {
        logger.error('Failed to notify host of resize', { error: err, width, height });
      });
    };

    window.addEventListener('resize', handleResize);

    return (): void => {
      window.removeEventListener('resize', handleResize);
    };
  }, []);
}
