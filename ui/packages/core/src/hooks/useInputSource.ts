import { useCallback, useEffect, useState } from 'react';

import { InputSourceClient } from '../ipc/InputSourceClient';
import { IpcBridge } from '../ipc/IpcBridge';
import type { InputSourceKind, InputSourceOption } from '../types/input-source';
import type { IpcError } from '../types/ipc';

export interface UseInputSourceResult {
  selected: InputSourceKind | null;
  available: InputSourceOption[];
  setSelected: (selected: InputSourceKind) => Promise<void>;
  isLoading: boolean;
  error: IpcError | null;
}

export function useInputSource(): UseInputSourceResult {
  const [selectedInputSource, setSelectedInputSource] = useState<InputSourceKind | null>(null);
  const [available, setAvailable] = useState<InputSourceOption[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<IpcError | null>(null);

  const client = InputSourceClient.getInstance();
  const bridge = IpcBridge.getInstance();

  useEffect(() => {
    let cancelled = false;

    const fetchSelection = async (): Promise<void> => {
      setIsLoading(true);
      setError(null);

      try {
        const result = await client.getInputSource();
        if (!cancelled) {
          setSelectedInputSource(result.selected);
          setAvailable(result.available);
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
          setSelectedInputSource(null);
          setAvailable([]);
          setIsLoading(false);
        }
        return;
      }

      void fetchSelection();
    });

    const unsubscribeChanged = client.onInputSourceChanged((next) => {
      if (!cancelled) {
        setSelectedInputSource(next);
        setError(null);
      }
    });

    void fetchSelection();

    return () => {
      cancelled = true;
      unsubscribeChanged();
      unsubscribeConnection();
    };
  }, [client]);

  const setSelected = useCallback(
    async (next: InputSourceKind) => {
      const previous = selectedInputSource;
      setSelectedInputSource(next);
      setError(null);

      try {
        await client.setInputSource(next);
      } catch (err: unknown) {
        setSelectedInputSource(previous);
        setError(toIpcError(err));
        throw err;
      }
    },
    [client, selectedInputSource]
  );

  return { selected: selectedInputSource, available, setSelected, isLoading, error };
}

function toIpcError(err: unknown): IpcError {
  if (err && typeof err === 'object' && 'code' in err && 'message' in err) {
    return err as IpcError;
  }

  return {
    code: -32000,
    message: err instanceof Error ? err.message : String(err),
  };
}
