import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { IpcBridge } from '../ipc/IpcBridge';
import { IpcMethods } from '../ipc/constants';
import { useWindowResizeSync } from './useWindowResizeSync';

describe('useWindowResizeSync', () => {
  const mockInvoke = vi.fn().mockResolvedValue({ accepted: true });

  beforeEach(() => {
    vi.spyOn(IpcBridge, 'getInstance').mockReturnValue({
      invoke: mockInvoke,
      onConnectionChange: vi.fn(),
      on: vi.fn(),
    } as unknown as IpcBridge);

    Object.defineProperty(window, 'innerWidth', {
      configurable: true,
      writable: true,
      value: 1000,
    });
    Object.defineProperty(window, 'innerHeight', {
      configurable: true,
      writable: true,
      value: 700,
    });
  });

  afterEach(() => {
    mockInvoke.mockClear();
    vi.restoreAllMocks();
  });

  it('does not inject a legacy resize handle into the DOM', () => {
    renderHook(() => useWindowResizeSync());

    expect(document.getElementById('wavecraft-legacy-resize-handle')).not.toBeInTheDocument();
  });

  it('sends resize requests on window resize events', async () => {
    renderHook(() => useWindowResizeSync());

    window.innerWidth = 1040;
    window.innerHeight = 730;
    window.dispatchEvent(new Event('resize'));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(IpcMethods.REQUEST_RESIZE, {
        width: 1040,
        height: 730,
      });
    });
  });
});
