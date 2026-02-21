/**
 * useWindowResizeSync - Automatic window resize sync to host
 */

import { useEffect } from 'react';
import { IpcBridge } from '../ipc/IpcBridge';
import { IpcMethods } from '../ipc/constants';
import { logger } from '../logger';

const LEGACY_RESIZE_HANDLE_ID = 'wavecraft-legacy-resize-handle';

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

  const result = await bridge.invoke<RequestResizeResult>(IpcMethods.REQUEST_RESIZE, {
    width,
    height,
  });

  return result.accepted;
}

function maybeMountLegacyResizeHandle(): () => void {
  if (typeof document === 'undefined') {
    return (): void => {};
  }

  // New templates render the React ResizeHandle component. Keep this fallback
  // for older generated projects that only call useWindowResizeSync().
  if (document.querySelector('[data-testid="resize-handle"]')) {
    return (): void => {};
  }

  if (document.getElementById(LEGACY_RESIZE_HANDLE_ID)) {
    return (): void => {};
  }

  const handle = document.createElement('button');
  handle.id = LEGACY_RESIZE_HANDLE_ID;
  handle.setAttribute('aria-label', 'Resize window');
  handle.title = 'Resize window';
  handle.style.position = 'fixed';
  handle.style.right = '8px';
  handle.style.bottom = '8px';
  handle.style.zIndex = '9999';
  handle.style.width = '44px';
  handle.style.height = '44px';
  handle.style.borderRadius = '6px';
  handle.style.border = '1px solid rgba(255,255,255,0.25)';
  handle.style.background = 'rgba(0,0,0,0.45)';
  handle.style.color = 'rgba(255,255,255,0.85)';
  handle.style.cursor = 'nwse-resize';
  handle.style.display = 'flex';
  handle.style.alignItems = 'center';
  handle.style.justifyContent = 'center';
  handle.style.userSelect = 'none';
  handle.style.padding = '0';
  handle.style.fontSize = '18px';
  handle.style.lineHeight = '1';
  handle.textContent = '⇲';

  const dragStart = {
    x: 0,
    y: 0,
    width: 0,
    height: 0,
  };

  const onMouseMove = (moveEvent: MouseEvent): void => {
    const deltaX = moveEvent.clientX - dragStart.x;
    const deltaY = moveEvent.clientY - dragStart.y;
    const nextWidth = Math.max(400, dragStart.width + deltaX);
    const nextHeight = Math.max(300, dragStart.height + deltaY);

    requestResize(nextWidth, nextHeight).catch((error) => {
      logger.error('Legacy resize handle request failed', {
        error,
        width: nextWidth,
        height: nextHeight,
      });
    });
  };

  const onMouseUp = (): void => {
    handle.style.border = '1px solid rgba(255,255,255,0.25)';
    handle.style.background = 'rgba(0,0,0,0.45)';
    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);
  };

  const onMouseDown = (event: MouseEvent): void => {
    event.preventDefault();
    dragStart.x = event.clientX;
    dragStart.y = event.clientY;
    dragStart.width = window.innerWidth;
    dragStart.height = window.innerHeight;
    handle.style.border = '1px solid rgba(122, 162, 247, 0.75)';
    handle.style.background = 'rgba(122, 162, 247, 0.25)';

    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);
  };

  handle.addEventListener('mousedown', onMouseDown);
  document.body.appendChild(handle);

  return (): void => {
    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);
    handle.removeEventListener('mousedown', onMouseDown);
    if (handle.parentNode) {
      handle.parentNode.removeChild(handle);
    }
  };
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
    const cleanupLegacyHandle = maybeMountLegacyResizeHandle();

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
      cleanupLegacyHandle();
    };
  }, []);
}
