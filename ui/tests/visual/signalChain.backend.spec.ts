import { expect, test, type Page } from '@playwright/test';

type SlotType = 'processor' | 'tap';

interface SignalChainSlot {
  id: string;
  type: SlotType;
}

interface GetSignalChainOrderResult {
  slots: SignalChainSlot[];
}

test('reordering a processor from the UI updates the backend signal chain order', async ({
  page,
}) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'My Cool Plugin' })).toBeVisible();
  await expect(page.getByRole('list', { name: 'Signal chain processor order' })).toBeVisible();

  const defaultSignalChainOrder = await getBackendSignalChainOrderSlots(page);
  const reorderedSignalChainOrder = moveSlotAfter(
    defaultSignalChainOrder,
    'InputTrim',
    'Passthrough'
  );
  const defaultSignalChainIds = defaultSignalChainOrder.map((slot) => slot.id);
  const reorderedSignalChainIds = reorderedSignalChainOrder.map((slot) => slot.id);
  const renderedDefaultSignalChainIds = toRenderedSignalChainIds(defaultSignalChainIds);
  const renderedReorderedSignalChainIds = toRenderedSignalChainIds(reorderedSignalChainIds);

  try {
    await setBackendSignalChainOrder(page, defaultSignalChainOrder);

    await expect.poll(() => getBackendSignalChainOrder(page)).toEqual(defaultSignalChainIds);
    await expect
      .poll(() => getRenderedSignalChainOrder(page))
      .toEqual(renderedDefaultSignalChainIds);

    await dragSignalChainItem(page, 'InputTrim', 'Passthrough');

    await expect.poll(() => getBackendSignalChainOrder(page)).toEqual(reorderedSignalChainIds);
    await expect
      .poll(() => getRenderedSignalChainOrder(page))
      .toEqual(renderedReorderedSignalChainIds);
  } finally {
    await setBackendSignalChainOrder(page, defaultSignalChainOrder);
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

async function getBackendSignalChainOrderSlots(page: Page): Promise<SignalChainSlot[]> {
  const result = await invokeBackendJsonRpc<GetSignalChainOrderResult>(
    page,
    'getSignalChainOrder',
    {}
  );
  return result.slots;
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

function moveSlotAfter(
  slots: SignalChainSlot[],
  slotId: string,
  targetId: string
): SignalChainSlot[] {
  const sourceIndex = slots.findIndex((slot) => slot.id === slotId);
  const targetIndex = slots.findIndex((slot) => slot.id === targetId);

  expect(sourceIndex).toBeGreaterThanOrEqual(0);
  expect(targetIndex).toBeGreaterThanOrEqual(0);

  const reordered = [...slots];
  const [slot] = reordered.splice(sourceIndex, 1);
  if (!slot) {
    throw new Error(`Unable to move missing slot ${slotId}`);
  }
  const insertionIndex = reordered.findIndex((candidate) => candidate.id === targetId);
  reordered.splice(insertionIndex + 1, 0, slot);
  return reordered;
}

function toRenderedSignalChainIds(ids: string[]): string[] {
  return ids.filter((id) => id !== 'ExampleProcessor');
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
