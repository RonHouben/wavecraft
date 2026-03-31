import { act, fireEvent, render, renderHook, screen, waitFor } from '@testing-library/react';
import { type ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { useAllParameters } from '../hooks/useAllParameters';
import { IpcEvents } from '../ipc/constants';
import { IpcBridge } from '../ipc/IpcBridge';
import { ParameterClient } from '../ipc/ParameterClient';
import * as transportsModule from '../transports';
import { MockTransport } from '../transports/MockTransport';
import type { ParameterId, ParameterInfo, ParameterValue } from '../types/parameters';
import * as environmentModule from '../utils/environment';
import { useSettingsModal } from './useSettingsModal';
import { WavecraftProvider } from './WavecraftProvider';

const initialParams: ParameterInfo[] = [
  {
    id: 'gain' as ParameterId,
    name: 'Gain',
    type: 'float',
    value: 0.5,
    default: 0.5,
    min: 0,
    max: 1,
    unit: 'dB',
  },
];

function wrapper({ children }: Readonly<{ children: ReactNode }>) {
  return <WavecraftProvider>{children}</WavecraftProvider>;
}

function SettingsModalOpenButton(): React.JSX.Element {
  const { openSettingsModal } = useSettingsModal();

  return (
    <button type="button" onClick={openSettingsModal}>
      Open settings
    </button>
  );
}

function SettingsModalStateProbe(): React.JSX.Element {
  const { closeSettingsModal, isSettingsModalOpen } = useSettingsModal();

  return (
    <>
      <div data-testid="settings-modal-state">{String(isSettingsModalOpen)}</div>
      <button type="button" onClick={closeSettingsModal}>
        Close settings
      </button>
    </>
  );
}

describe('WavecraftProvider', () => {
  let mockTransport: MockTransport;
  let onParamChangedCallback: ((id: ParameterId, value: ParameterValue) => void) | null = null;

  beforeEach(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (IpcBridge as any).instance = null;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (ParameterClient as any).instance = null;

    mockTransport = new MockTransport();
    mockTransport.setConnected(true);

    vi.spyOn(transportsModule, 'getTransport').mockReturnValue(mockTransport);
    vi.spyOn(environmentModule, 'isWebViewEnvironment').mockReturnValue(false);
    vi.spyOn(ParameterClient.prototype, 'onParameterChanged').mockImplementation((callback) => {
      onParamChangedCallback = callback;
      return () => {
        onParamChangedCallback = null;
      };
    });
  });

  afterEach(() => {
    mockTransport?.dispose();
    vi.restoreAllMocks();
  });

  it('applies parameterChanged push updates to shared state', async () => {
    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue(initialParams);

    const { result } = renderHook(() => useAllParameters(), { wrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    const callback = onParamChangedCallback;
    if (!callback) {
      throw new Error('Expected parameter change callback to be registered');
    }

    await act(async () => {
      callback('gain' as ParameterId, 0.9);
    });

    expect(result.current.params[0]?.value).toBe(0.9);
  });

  it('shares settings modal state across consumers under one provider', async () => {
    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue(initialParams);

    render(
      <WavecraftProvider>
        <SettingsModalOpenButton />
        <SettingsModalStateProbe />
      </WavecraftProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('settings-modal-state')).toHaveTextContent('false');
    });

    fireEvent.click(screen.getByRole('button', { name: 'Open settings' }));
    expect(screen.getByTestId('settings-modal-state')).toHaveTextContent('true');

    fireEvent.click(screen.getByRole('button', { name: 'Close settings' }));
    expect(screen.getByTestId('settings-modal-state')).toHaveTextContent('false');
  });

  it('requires WavecraftProvider for settings modal state', () => {
    expect(() => renderHook(() => useSettingsModal())).toThrow(
      /WavecraftProvider is required. Wrap your app with <WavecraftProvider>./
    );
  });

  it('reloads on PARAMETERS_CHANGED notification', async () => {
    const client = ParameterClient.getInstance();
    const getAllSpy = vi
      .spyOn(client, 'getAllParameters')
      .mockResolvedValueOnce(initialParams)
      .mockResolvedValueOnce([
        ...initialParams,
        {
          id: 'drive' as ParameterId,
          name: 'Drive',
          type: 'float',
          value: 0.2,
          default: 0.2,
          min: 0,
          max: 1,
          unit: '',
        },
      ]);

    const { result } = renderHook(() => useAllParameters(), { wrapper });

    await waitFor(() => {
      expect(result.current.params).toHaveLength(1);
    });

    await act(async () => {
      mockTransport.simulateNotification({
        jsonrpc: '2.0',
        method: IpcEvents.PARAMETERS_CHANGED,
        params: {},
      });
    });

    await waitFor(() => {
      expect(result.current.params).toHaveLength(2);
    });

    expect(getAllSpy).toHaveBeenCalledTimes(2);
  });

  it('rolls back optimistic setParameter value when write fails', async () => {
    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue(initialParams);
    vi.spyOn(client, 'setParameter').mockRejectedValue(new Error('Write failed'));

    const { result } = renderHook(() => useAllParameters(), { wrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    let writeError: unknown;
    await act(async () => {
      try {
        await result.current.setParameter('gain' as ParameterId, 1);
      } catch (err) {
        writeError = err;
      }
    });

    expect(writeError).toBeInstanceOf(Error);
    expect((writeError as Error).message).toContain('Write failed');

    await waitFor(() => {
      const gain = result.current.params.find((param) => param.id === ('gain' as ParameterId));
      expect(gain?.value).toBe(0.5);
    });
  });

  it('does not clobber newer external state during failed optimistic rollback race', async () => {
    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue(initialParams);

    let rejectWrite: ((error: Error) => void) | null = null;
    vi.spyOn(client, 'setParameter').mockImplementation(
      () =>
        new Promise<void>((_, reject) => {
          rejectWrite = (error: Error) => reject(error);
        })
    );

    const { result } = renderHook(() => useAllParameters(), { wrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    const callback = onParamChangedCallback;
    if (!callback) {
      throw new Error('Expected parameter change callback to be registered');
    }

    let pendingWrite!: Promise<void>;
    act(() => {
      pendingWrite = result.current.setParameter('gain' as ParameterId, 1);
    });

    const writeErrorResult = (pendingWrite as Promise<void>).catch((error: unknown) => error);

    await waitFor(() => {
      const gain = result.current.params.find((param) => param.id === ('gain' as ParameterId));
      expect(gain?.value).toBe(1);
    });

    await act(async () => {
      callback('gain' as ParameterId, 0.9);
    });

    await waitFor(() => {
      const gain = result.current.params.find((param) => param.id === ('gain' as ParameterId));
      expect(gain?.value).toBe(0.9);
    });

    if (!rejectWrite) {
      throw new Error('Expected write promise reject handler to be captured');
    }

    await act(async () => {
      rejectWrite?.(new Error('Write failed'));
    });

    const writeError = await writeErrorResult;
    expect(writeError).toBeInstanceOf(Error);
    expect((writeError as Error).message).toContain('Write failed');

    await waitFor(() => {
      const gain = result.current.params.find((param) => param.id === ('gain' as ParameterId));
      expect(gain?.value).toBe(0.9);
    });
  });

  it('fetches parameters after reconnect when mounted disconnected', async () => {
    mockTransport.setConnected(false);
    const client = ParameterClient.getInstance();
    const getAllSpy = vi.spyOn(client, 'getAllParameters').mockResolvedValue(initialParams);

    const { result } = renderHook(() => useAllParameters(), { wrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(true);
    });
    expect(getAllSpy).not.toHaveBeenCalled();

    await act(async () => {
      mockTransport.setConnected(true);
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
      expect(result.current.params).toHaveLength(1);
    });

    expect(getAllSpy).toHaveBeenCalledTimes(1);
  });

  it('sets timeout error after 15 seconds when connection never becomes available', async () => {
    vi.useFakeTimers();

    try {
      mockTransport.setConnected(false);
      const client = ParameterClient.getInstance();
      const getAllSpy = vi.spyOn(client, 'getAllParameters').mockResolvedValue(initialParams);

      const { result } = renderHook(() => useAllParameters(), { wrapper });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(14_999);
      });

      expect(result.current.error).toBeNull();
      expect(result.current.isLoading).toBe(true);
      expect(getAllSpy).not.toHaveBeenCalled();

      await act(async () => {
        await vi.advanceTimersByTimeAsync(1);
      });

      expect(result.current.isLoading).toBe(false);
      expect(result.current.error?.message).toContain(
        'Could not connect to dev server within 15 seconds'
      );
    } finally {
      vi.useRealTimers();
    }
  });
});
