import { expect, test, type Page } from '@playwright/test';

type SlotType = 'processor' | 'tap';

interface SignalChainSlot {
  id: string;
  type: SlotType;
}

interface SignalChainNotification {
  slots: SignalChainSlot[];
}

interface GetSignalChainOrderResult {
  slots: SignalChainSlot[];
}

const DEFAULT_SIGNAL_CHAIN_ORDER: SignalChainSlot[] = [
  { id: 'test_tone', type: 'processor' },
  { id: 'input_trim', type: 'processor' },
  { id: 'passthrough', type: 'processor' },
  { id: 'tone_filter', type: 'processor' },
  { id: 'soft_clip', type: 'processor' },
  { id: 'output_gain', type: 'processor' },
  { id: 'oscilloscope_tap', type: 'tap' },
];

const REORDERED_SIGNAL_CHAIN_ORDER: SignalChainSlot[] = [
  { id: 'input_trim', type: 'processor' },
  { id: 'test_tone', type: 'processor' },
  { id: 'passthrough', type: 'processor' },
  { id: 'tone_filter', type: 'processor' },
  { id: 'soft_clip', type: 'processor' },
  { id: 'output_gain', type: 'processor' },
  { id: 'oscilloscope_tap', type: 'tap' },
];

const DEFAULT_SIGNAL_CHAIN_IDS = DEFAULT_SIGNAL_CHAIN_ORDER.map((slot) => slot.id);
const REORDERED_SIGNAL_CHAIN_IDS = REORDERED_SIGNAL_CHAIN_ORDER.map((slot) => slot.id);

test.afterEach(async ({ page }) => {
  await setBackendSignalChainOrder(page, DEFAULT_SIGNAL_CHAIN_ORDER);
});

test('dragging a processor updates the backend signal chain order', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'My Plugin' })).toBeVisible();
  await expect(page.getByRole('list', { name: 'Signal chain processor order' })).toBeVisible();

  await setBackendSignalChainOrder(page, DEFAULT_SIGNAL_CHAIN_ORDER);

  await expect.poll(() => getBackendSignalChainOrder(page)).toEqual(DEFAULT_SIGNAL_CHAIN_IDS);
  await expect.poll(() => getRenderedSignalChainOrder(page)).toEqual(DEFAULT_SIGNAL_CHAIN_IDS);

  const orderChanged = waitForSignalChainOrderChanged(page);

  await dragSignalChainItem(page, 'test_tone', 'input_trim');

  await expect(orderChanged).resolves.toEqual(REORDERED_SIGNAL_CHAIN_IDS);
  await expect.poll(() => getBackendSignalChainOrder(page)).toEqual(REORDERED_SIGNAL_CHAIN_IDS);
  await expect.poll(() => getRenderedSignalChainOrder(page)).toEqual(REORDERED_SIGNAL_CHAIN_IDS);
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

async function waitForSignalChainOrderChanged(page: Page): Promise<string[]> {
  const notification = await page.evaluate(
    () =>
      new Promise<SignalChainNotification>((resolve, reject) => {
        const ws = new WebSocket('ws://127.0.0.1:9000');
        const timeoutId = globalThis.setTimeout(() => {
          ws.close();
          reject(new Error('Timed out waiting for signalChainOrderChanged notification'));
        }, 5000);

        ws.onerror = () => {
          globalThis.clearTimeout(timeoutId);
          ws.close();
          reject(new Error('WebSocket error while waiting for signalChainOrderChanged'));
        };

        ws.onmessage = (event) => {
          const message = JSON.parse(event.data) as {
            method?: string;
            params?: SignalChainNotification;
          };
          if (message.method !== 'signalChainOrderChanged' || !message.params) {
            return;
          }

          globalThis.clearTimeout(timeoutId);
          ws.close();
          resolve(message.params);
        };
      })
  );

  return notification.slots.map((slot) => slot.id);
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

  const sourceBox = await sourceHandle.boundingBox();
  const targetBox = await targetItem.boundingBox();

  if (!sourceBox || !targetBox) {
    throw new Error(`Missing drag bounds for ${sourceId} -> ${targetId}`);
  }

  await page.mouse.move(sourceBox.x + sourceBox.width / 2, sourceBox.y + sourceBox.height / 2);
  await page.mouse.down();
  await page.mouse.move(
    sourceBox.x + sourceBox.width / 2,
    sourceBox.y + sourceBox.height / 2 + 24,
    { steps: 8 }
  );
  await page.mouse.move(
    targetBox.x + Math.min(targetBox.width / 2, 48),
    targetBox.y + targetBox.height / 2,
    { steps: 20 }
  );
  await page.mouse.up();
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
