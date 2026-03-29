import { expect, test, type Page } from '@playwright/test';

type SlotType = 'processor' | 'tap';
type InputSourceKind = 'hardwareInput' | 'testTone';

interface SignalChainSlot {
  id: string;
  type: SlotType;
}

interface JsonRpcSuccess<T> {
  result: T;
}

interface OscilloscopeFrame {
  points_l: number[];
  points_r: number[];
  sample_rate: number;
  timestamp: number;
  no_signal: boolean;
  trigger_mode: 'risingZeroCrossing';
}

interface GetOscilloscopeFrameResult {
  frame: OscilloscopeFrame | null;
}

interface GetSignalChainOrderResult {
  slots: SignalChainSlot[];
}

const SOFT_CLIP_DRIVE_PARAM_ID = 'soft_clip_drive_db';

test('moving Passthrough across TestTone updates the Passthrough eye based on chain position', async ({
  page,
}) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'My Cool Plugin' })).toBeVisible();

  const defaultOrder = await getSignalChainOrderSlots(page);
  const passthroughBeforeTestToneOrder = moveSlotBefore(defaultOrder, 'Passthrough', 'TestTone');
  const testToneBeforePassthroughOrder = moveSlotBefore(defaultOrder, 'TestTone', 'Passthrough');
  const passthroughBeforeTestToneIds = passthroughBeforeTestToneOrder.map((slot) => slot.id);
  const testToneBeforePassthroughIds = testToneBeforePassthroughOrder.map((slot) => slot.id);

  try {
    await setInputSource(page, 'testTone');
    await setParameter(page, 'test_tone_bypass', 0);
    await setParameter(page, 'test_tone_enabled', 1);
    await setParameter(page, 'test_tone_level', 0.5);

    await setSignalChainOrder(page, passthroughBeforeTestToneOrder);
    await expect.poll(() => getSignalChainOrder(page)).toEqual(passthroughBeforeTestToneIds);
    await expect.poll(() => getPassthroughEyeSignalActive(page)).toBe(false);

    await setSignalChainOrder(page, testToneBeforePassthroughOrder);
    await expect.poll(() => getSignalChainOrder(page)).toEqual(testToneBeforePassthroughIds);
    await expect.poll(() => getPassthroughEyeSignalActive(page)).toBe(true);
  } finally {
    await setInputSource(page, 'hardwareInput');
    await setParameter(page, 'test_tone_enabled', 0);
    await setParameter(page, 'test_tone_bypass', 0);
    await setParameter(page, 'test_tone_level', 0.5);
    await setSignalChainOrder(page, defaultOrder);
  }
});

test('moving oscilloscope before soft clip removes soft clip influence from runtime oscilloscope frames', async ({
  page,
}) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'My Cool Plugin' })).toBeVisible();

  const defaultOrder = await getSignalChainOrderSlots(page);
  const oscilloscopeAfterSoftClipOrder = moveSlotBefore(
    defaultOrder,
    'SoftClip',
    'OscilloscopeTap'
  );
  const oscilloscopeBeforeSoftClipOrder = moveSlotBefore(
    defaultOrder,
    'OscilloscopeTap',
    'SoftClip'
  );
  const oscilloscopeAfterSoftClipIds = oscilloscopeAfterSoftClipOrder.map((slot) => slot.id);
  const oscilloscopeBeforeSoftClipIds = oscilloscopeBeforeSoftClipOrder.map((slot) => slot.id);

  try {
    await setInputSource(page, 'testTone');
    await setParameter(page, 'tone_filter_bypass', 1);
    await setParameter(page, 'soft_clip_mix', 1);
    await setParameter(page, 'soft_clip_output_db', 0);
    await setParameter(page, SOFT_CLIP_DRIVE_PARAM_ID, 0);

    await setSignalChainOrder(page, oscilloscopeAfterSoftClipOrder);
    await expect.poll(() => getSignalChainOrder(page)).toEqual(oscilloscopeAfterSoftClipIds);
    await sleep(250);

    const downstreamClean = await waitForFreshOscilloscopeFrame(page);
    await setParameter(page, SOFT_CLIP_DRIVE_PARAM_ID, 24);
    await sleep(150);
    const downstreamDriven = await waitForFreshOscilloscopeFrame(page, downstreamClean.timestamp);
    const downstreamDelta = frameDifference(downstreamClean, downstreamDriven);

    await setSignalChainOrder(page, oscilloscopeBeforeSoftClipOrder);
    await expect.poll(() => getSignalChainOrder(page)).toEqual(oscilloscopeBeforeSoftClipIds);
    await sleep(350);

    await setParameter(page, SOFT_CLIP_DRIVE_PARAM_ID, 0);
    await sleep(150);
    const upstreamClean = await waitForFreshOscilloscopeFrame(page, downstreamDriven.timestamp);
    await setParameter(page, SOFT_CLIP_DRIVE_PARAM_ID, 24);
    await sleep(150);
    const upstreamDriven = await waitForFreshOscilloscopeFrame(page, upstreamClean.timestamp);
    const upstreamDelta = frameDifference(upstreamClean, upstreamDriven);

    expect(downstreamDelta).toBeGreaterThan(0.02);
    expect(upstreamDelta).toBeLessThan(0.02);
    expect(downstreamDelta).toBeGreaterThan(upstreamDelta * 10);
  } finally {
    await setInputSource(page, 'hardwareInput');
    await setParameter(page, SOFT_CLIP_DRIVE_PARAM_ID, 0);
    await setParameter(page, 'soft_clip_mix', 1);
    await setParameter(page, 'soft_clip_output_db', 0);
    await setParameter(page, 'tone_filter_bypass', 1);
    await setSignalChainOrder(page, defaultOrder);
  }
});

async function sleep(ms: number): Promise<void> {
  await new Promise((resolve) => {
    globalThis.setTimeout(resolve, ms);
  });
}

function moveSlotBefore(
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
  reordered.splice(insertionIndex, 0, slot);
  return reordered;
}

async function waitForFreshOscilloscopeFrame(
  page: Page,
  previousTimestamp = -1
): Promise<OscilloscopeFrame> {
  await expect
    .poll(async () => {
      const frame = await getOscilloscopeFrame(page);
      if (!frame || frame.no_signal || frame.timestamp <= previousTimestamp) {
        return null;
      }

      return frame;
    })
    .not.toBeNull();

  const firstFresh = await getOscilloscopeFrame(page);
  if (!firstFresh || firstFresh.no_signal) {
    throw new Error('Expected a fresh oscilloscope frame after polling');
  }

  await expect
    .poll(async () => {
      const frame = await getOscilloscopeFrame(page);
      if (!frame || frame.no_signal || frame.timestamp <= firstFresh.timestamp) {
        return null;
      }

      return frame;
    })
    .not.toBeNull();

  const settled = await getOscilloscopeFrame(page);
  if (!settled || settled.no_signal) {
    throw new Error('Expected a settled oscilloscope frame after polling');
  }

  return settled;
}

async function getPassthroughEyeSignalActive(page: Page): Promise<boolean> {
  const state = await page.getByTestId('passthrough-eye').getAttribute('data-signal-active');
  return state === 'true';
}

function frameDifference(a: OscilloscopeFrame, b: OscilloscopeFrame): number {
  const sampleCount = Math.min(a.points_l.length, b.points_l.length, 256);
  if (sampleCount === 0) {
    return 0;
  }

  let total = 0;
  for (let index = 0; index < sampleCount; index += 1) {
    const leftA = a.points_l[index] ?? 0;
    const leftB = b.points_l[index] ?? 0;
    total += Math.abs(leftA - leftB);
  }

  return total / sampleCount;
}

async function getOscilloscopeFrame(page: Page): Promise<OscilloscopeFrame | null> {
  const result = await invokeBackendJsonRpc<GetOscilloscopeFrameResult>(
    page,
    'getOscilloscopeFrame',
    {}
  );
  return result.frame;
}

async function getSignalChainOrder(page: Page): Promise<string[]> {
  const result = await invokeBackendJsonRpc<GetSignalChainOrderResult>(
    page,
    'getSignalChainOrder',
    {}
  );
  return result.slots.map((slot) => slot.id);
}

async function getSignalChainOrderSlots(page: Page): Promise<SignalChainSlot[]> {
  const result = await invokeBackendJsonRpc<GetSignalChainOrderResult>(
    page,
    'getSignalChainOrder',
    {}
  );
  return result.slots;
}

async function setSignalChainOrder(page: Page, slots: SignalChainSlot[]): Promise<void> {
  await invokeBackendJsonRpc<Record<string, never>>(page, 'setSignalChainOrder', { slots });
}

async function setInputSource(page: Page, selected: InputSourceKind): Promise<void> {
  await invokeBackendJsonRpc<Record<string, never>>(page, 'setInputSource', { selected });
}

async function setParameter(page: Page, id: string, value: number): Promise<void> {
  await invokeBackendJsonRpc<Record<string, never>>(page, 'setParameter', { id, value });
}

async function invokeBackendJsonRpc<T>(
  page: Page,
  method: string,
  params: Record<string, unknown>
): Promise<T> {
  return page.evaluate(
    ({ method, params }) =>
      new Promise<T>((resolve, reject) => {
        const ws = new WebSocket('ws://127.0.0.1:9000');
        const requestId = Math.floor(Math.random() * 1_000_000);
        const timeoutId = globalThis.setTimeout(() => {
          ws.close();
          reject(new Error(`Timed out waiting for ${method}`));
        }, 5000);

        ws.onerror = () => {
          globalThis.clearTimeout(timeoutId);
          ws.close();
          reject(new Error(`WebSocket error while invoking ${method}`));
        };

        ws.onopen = () => {
          ws.send(
            JSON.stringify({
              jsonrpc: '2.0',
              id: requestId,
              method,
              params,
            })
          );
        };

        ws.onmessage = (event) => {
          const message = JSON.parse(event.data) as
            | JsonRpcSuccess<T>
            | {
                id?: number;
                error?: { message?: string };
              };

          if (!('id' in message) || message.id !== requestId) {
            return;
          }

          globalThis.clearTimeout(timeoutId);
          ws.close();

          if ('error' in message && message.error) {
            reject(new Error(message.error.message ?? `JSON-RPC ${method} failed`));
            return;
          }

          resolve((message as JsonRpcSuccess<T>).result);
        };
      }),
    { method, params }
  );
}
