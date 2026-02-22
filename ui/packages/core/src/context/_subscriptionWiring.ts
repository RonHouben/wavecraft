import type { ParameterId, ParameterValue } from '../types/parameters';
import { IpcBridge } from '../ipc/IpcBridge';
import { IpcEvents } from '../ipc/constants';
import { ParameterClient } from '../ipc/ParameterClient';
import { logger } from '../logger/Logger';

// private — do not export from index.ts
export function wireParameterChangedSubscription(
  onChanged: (changedId: ParameterId, value: ParameterValue) => void
): () => void {
  const client = ParameterClient.getInstance();
  return client.onParameterChanged(onChanged);
}

// private — do not export from index.ts
export function wireParametersChangedReload(reload: () => Promise<void>): () => void {
  const bridge = IpcBridge.getInstance();
  const unsubscribe = bridge.on(IpcEvents.PARAMETERS_CHANGED, () => {
    logger.info('Parameters changed on server (hot-reload), re-fetching...');
    void reload();
  });

  return unsubscribe;
}
