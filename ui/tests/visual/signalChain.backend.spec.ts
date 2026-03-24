import { expect, test, type Page } from '@playwright/test';

type SlotType = 'processor' | 'tap';

interface SignalChainSlot {
  id: string;
  type: SlotType;
}

interface GetSignalChainOrderResult {
  slots: SignalChainSlot[];
}

const DEFAULT_SIGNAL_CHAIN_ORDER: SignalChainSlot[] = [
  { id: 'TestTone', type: 'processor' },
  { id: 'InputTrim', type: 'processor' },
  { id: 'Passthrough', type: 'processor' },
  { id: 'ExampleProcessor', type: 'processor' },
  { id: 'ToneFilter', type: 'processor' },
  { id: 'SoftClip', type: 'processor' },
  { id: 'OutputGain', type: 'processor' },
  { id: 'OscilloscopeTap', type: 'tap' },
];

const REORDERED_SIGNAL_CHAIN_ORDER: SignalChainSlot[] = [
  { id: 'InputTrim', type: 'processor' },
  { id: 'TestTone', type: 'processor' },
  { id: 'Passthrough', type: 'processor' },
  { id: 'ExampleProcessor', type: 'processor' },
  { id: 'ToneFilter', type: 'processor' },
  { id: 'SoftClip', type: 'processor' },
  { id: 'OutputGain', type: 'processor' },
  { id: 'OscilloscopeTap', type: 'tap' },
];

const DEFAULT_SIGNAL_CHAIN_IDS = DEFAULT_SIGNAL_CHAIN_ORDER.map((slot) => slot.id);
const REORDERED_SIGNAL_CHAIN_IDS = REORDERED_SIGNAL_CHAIN_ORDER.map((slot) => slot.id);
const RENDERED_DEFAULT_SIGNAL_CHAIN_IDS = DEFAULT_SIGNAL_CHAIN_IDS.filter(
  (id) => id !== 'ExampleProcessor'
);
const RENDERED_REORDERED_SIGNAL_CHAIN_IDS = REORDERED_SIGNAL_CHAIN_IDS.filter(
  (id) => id !== 'ExampleProcessor'
);

test('reordering a processor from the UI updates the backend signal chain order', async ({
  page,
}) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'My Plugin' })).toBeVisible();
  await expect(page.getByRole('list', { name: 'Signal chain processor order' })).toBeVisible();

  try {
    await setBackendSignalChainOrder(page, DEFAULT_SIGNAL_CHAIN_ORDER);

    await expect.poll(() => getBackendSignalChainOrder(page)).toEqual(DEFAULT_SIGNAL_CHAIN_IDS);
    await expect
      .poll(() => getRenderedSignalChainOrder(page))
      .toEqual(RENDERED_DEFAULT_SIGNAL_CHAIN_IDS);

    await dragSignalChainItem(page, 'TestTone', 'InputTrim');

    await expect.poll(() => getBackendSignalChainOrder(page)).toEqual(REORDERED_SIGNAL_CHAIN_IDS);
    await expect
      .poll(() => getRenderedSignalChainOrder(page))
      .toEqual(RENDERED_REORDERED_SIGNAL_CHAIN_IDS);
  } finally {
    await setBackendSignalChainOrder(page, DEFAULT_SIGNAL_CHAIN_ORDER);
  }
});

async function getBackendSignalChainOrder(page: Page): Promise<string[]> {
  const result = await invokeBackendJsonRpc<GetSignalChainOrderResult>(
    page,
    'getSignalChainOrder',
    {}
  );
  return result.slots.map((slot) => slot.id);
}

async function setBackendSignalChainOrder(page: Page, slots: SignalChainSlot[]): Promise<void> {
  await invokeBackendJsonRpc<Record<string, never>>(page, 'setSignalChainOrder', { slots });
}

async function getRenderedSignalChainOrder(page: Page): Promise<string[]> {
  return page
    .locator('[data-slot-id]')
    .evaluateAll((elements) =>
      elements
        .map((element) =>
          element instanceof HTMLElement ? (element.dataset.slotId ?? null) : null
        )
        .filter((slotId): slotId is string => slotId !== null)
    );
}

async function dragSignalChainItem(page: Page, sourceId: string, targetId: string): Promise<void> {
  const sourceHandle = page.getByTestId(`signal-chain-handle-${sourceId}`);
  const targetItem = page.getByTestId(`signal-chain-item-${targetId}`);

  await expect(sourceHandle).toBeVisible();
  await expect(targetItem).toBeVisible();

  await sourceHandle.dragTo(targetItem, {
    targetPosition: { x: 48, y: 24 },
  });
}

async function invokeBackendJsonRpc<TResult>(
  page: Page,
  method: string,
  params: Record<string, unknown>
): Promise<TResult> {
  return page.evaluate(
    ({ method: requestMethod, params: requestParams }) =>
      new Promise<TResult>((resolve, reject) => {
        const requestId = Math.floor(Math.random() * 1_000_000);
        const ws = new WebSocket('ws://127.0.0.1:9000');
        const timeoutId = globalThis.setTimeout(() => {
          ws.close();
          reject(new Error(`Timed out waiting for ${requestMethod} response`));
        }, 5000);

        ws.onerror = () => {
          globalThis.clearTimeout(timeoutId);
          ws.close();
          reject(new Error(`WebSocket error while invoking ${requestMethod}`));
        };

        ws.onmessage = (event) => {
          const message = JSON.parse(event.data) as {
            id?: number;
            result?: TResult;
            error?: { message?: string };
          };

          if (message.id !== requestId) {
            return;
          }

          globalThis.clearTimeout(timeoutId);
          ws.close();

          if (message.error) {
            reject(new Error(message.error.message ?? `JSON-RPC error calling ${requestMethod}`));
            return;
          }

          if (message.result === undefined) {
            reject(new Error(`Missing JSON-RPC result for ${requestMethod}`));
            return;
          }

          resolve(message.result);
        };

        ws.onopen = () => {
          ws.send(
            JSON.stringify({
              jsonrpc: '2.0',
              id: requestId,
              method: requestMethod,
              params: requestParams,
            })
          );
        };
      }),
    { method, params }
  );
}
