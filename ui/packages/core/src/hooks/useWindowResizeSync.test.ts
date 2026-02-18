import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { IpcBridge } from '../ipc/IpcBridge';
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
    document.getElementById('wavecraft-legacy-resize-handle')?.remove();
    document.querySelector('[data-testid="resize-handle"]')?.remove();
  });

  it('mounts legacy resize handle when template resize handle is missing', () => {
    const { unmount } = renderHook(() => useWindowResizeSync());

    expect(document.getElementById('wavecraft-legacy-resize-handle')).toBeInTheDocument();

    unmount();

    expect(document.getElementById('wavecraft-legacy-resize-handle')).not.toBeInTheDocument();
  });

  it('does not mount legacy handle when modern ResizeHandle already exists', () => {
    const modernHandle = document.createElement('button');
    modernHandle.setAttribute('data-testid', 'resize-handle');
    document.body.appendChild(modernHandle);

    renderHook(() => useWindowResizeSync());

    expect(document.getElementById('wavecraft-legacy-resize-handle')).not.toBeInTheDocument();
  });

  it('sends resize requests while dragging the legacy handle', async () => {
    renderHook(() => useWindowResizeSync());

    const legacyHandle = document.getElementById('wavecraft-legacy-resize-handle');
    expect(legacyHandle).toBeInTheDocument();

    legacyHandle?.dispatchEvent(
      new MouseEvent('mousedown', { clientX: 100, clientY: 100, bubbles: true })
    );
    document.dispatchEvent(
      new MouseEvent('mousemove', { clientX: 140, clientY: 130, bubbles: true })
    );
    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('requestResize', { width: 1040, height: 730 });
    });
  });
});
