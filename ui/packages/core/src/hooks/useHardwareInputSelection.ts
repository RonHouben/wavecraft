import { useCallback, useEffect, useMemo, useState } from 'react';

import { HardwareInputClient } from '../ipc/HardwareInputClient';
import { IpcBridge } from '../ipc/IpcBridge';
import type {
  GetHardwareInputSelectionResult,
  HardwareInputChannelOption,
  HardwareInputDeviceOption,
} from '../types/hardware-input';
import type { IpcError } from '../types/ipc';

export interface UseHardwareInputSelectionResult {
  selectedDeviceId: string | null;
  selectedDevice: HardwareInputDeviceOption | null;
  availableDevices: HardwareInputDeviceOption[];
  selectedChannelId: string | null;
  availableChannels: HardwareInputChannelOption[];
  setSelectedChannel: (selectedChannelId: string) => Promise<void>;
  isLoading: boolean;
  error: IpcError | null;
}

export function useHardwareInputSelection(): UseHardwareInputSelectionResult {
  const [selection, setSelection] = useState<GetHardwareInputSelectionResult>({
    selected_device_id: null,
    available_devices: [],
    selected_channel_id: null,
    available_channels: [],
  });
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<IpcError | null>(null);

  const client = HardwareInputClient.getInstance();
  const bridge = IpcBridge.getInstance();

  useEffect(() => {
    let cancelled = false;

    const fetchSelection = async (): Promise<void> => {
      setIsLoading(true);
      setError(null);

      try {
        const nextSelection = await client.getHardwareInputSelection();
        if (!cancelled) {
          setSelection(nextSelection);
          setIsLoading(false);
        }
      } catch (err: unknown) {
        if (!cancelled) {
          setError(toIpcError(err));
          setIsLoading(false);
        }
      }
    };

    const unsubscribeConnection = bridge.onConnectionChange((connected) => {
      if (!connected) {
        if (!cancelled) {
          setSelection({
            selected_device_id: null,
            available_devices: [],
            selected_channel_id: null,
            available_channels: [],
          });
          setIsLoading(false);
        }
        return;
      }

      void fetchSelection();
    });

    const unsubscribeChanged = client.onHardwareInputSelectionChanged((nextSelection) => {
      if (!cancelled) {
        setSelection(nextSelection);
        setError(null);
      }
    });

    void fetchSelection();

    return () => {
      cancelled = true;
      unsubscribeChanged();
      unsubscribeConnection();
    };
  }, [bridge, client]);

  const setSelectedChannel = useCallback(
    async (selectedChannelId: string) => {
      const previousSelection = selection;
      setSelection((current) => ({
        ...current,
        selected_channel_id: selectedChannelId,
      }));
      setError(null);

      try {
        await client.setHardwareInputSelection({
          selected_channel_id: selectedChannelId,
        });
      } catch (err: unknown) {
        setSelection(previousSelection);
        setError(toIpcError(err));
        throw err;
      }
    },
    [client, selection]
  );

  const selectedDevice = useMemo(
    () =>
      selection.available_devices.find((device) => device.id === selection.selected_device_id) ??
      null,
    [selection.available_devices, selection.selected_device_id]
  );

  return {
    selectedDeviceId: selection.selected_device_id,
    selectedDevice,
    availableDevices: selection.available_devices,
    selectedChannelId: selection.selected_channel_id,
    availableChannels: selection.available_channels,
    setSelectedChannel,
    isLoading,
    error,
  };
}

function toIpcError(err: unknown): IpcError {
  if (err && typeof err === 'object' && 'code' in err && 'message' in err) {
    return err as IpcError;
  }

  if (err instanceof Error) {
    return {
      code: -32000,
      message: err.message,
    };
  }

  if (typeof err === 'object' && err !== null) {
    return {
      code: -32000,
      message: JSON.stringify(err),
    };
  }

  return {
    code: -32000,
    message: typeof err === 'string' ? err : JSON.stringify(err),
  };
}
