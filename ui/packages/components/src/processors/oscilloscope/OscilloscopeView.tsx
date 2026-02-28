import React, { useEffect, useMemo, useRef, useState } from 'react';
import type {
  OscilloscopeChannelView,
  OscilloscopeFrame,
  OscilloscopeTriggerMode,
} from '../../types';
import { Select } from '../../Select';
import { Row } from '../../Row';

const PADDING = 10;

// Canvas drawing cannot consume Tailwind classes directly, so these values mirror
// existing design tokens (meter.safe, accent, plugin.border, gray-500, plugin.dark).
const LEFT_COLOR = '#4caf50';
const RIGHT_COLOR = '#4a9eff';
const GRID_COLOR = 'rgba(68, 68, 68, 0.65)';
const AXIS_COLOR = '#6b7280';
const BACKGROUND_COLOR = '#1a1a1a';

interface OscilloscopeProcessorProps {
  readonly connected: boolean;
  readonly frame: OscilloscopeFrame | null;
}

interface CanvasViewport {
  readonly width: number;
  readonly height: number;
  readonly dpr: number;
}

function getDevicePixelRatio(): number {
  if (typeof window === 'undefined') {
    return 1;
  }

  return Number.isFinite(window.devicePixelRatio) && window.devicePixelRatio > 0
    ? window.devicePixelRatio
    : 1;
}

export function OscilloscopeView({
  connected,
  frame,
}: Readonly<OscilloscopeProcessorProps>): React.JSX.Element | null {
  const [channelView, setChannelView] = useState<OscilloscopeChannelView>('overlay');
  const [triggerMode, setTriggerMode] = useState<OscilloscopeTriggerMode>('risingZeroCrossing');

  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const frameRef = useRef<OscilloscopeFrame | null>(null);
  const rafRef = useRef<number | null>(null);
  const viewportRef = useRef<CanvasViewport>({ width: 1, height: 1, dpr: 1 });
  const needsResizeRef = useRef<boolean>(true);

  useEffect(() => {
    frameRef.current = frame;
  }, [frame]);

  const noSignal = useMemo(() => {
    if (!connected) {
      return true;
    }

    const latest = frame;
    if (!latest) {
      return true;
    }

    return latest.no_signal;
  }, [connected, frame]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const context = canvas.getContext('2d');
    if (!context) {
      return;
    }

    const syncCanvasViewport = (): void => {
      const rect = canvas.getBoundingClientRect();
      const width = Math.max(1, Math.round(rect.width));
      const height = Math.max(1, Math.round(rect.height));
      const dpr = getDevicePixelRatio();

      viewportRef.current = { width, height, dpr };
      needsResizeRef.current = false;

      const backingWidth = Math.max(1, Math.round(width * dpr));
      const backingHeight = Math.max(1, Math.round(height * dpr));

      if (canvas.width !== backingWidth || canvas.height !== backingHeight) {
        canvas.width = backingWidth;
        canvas.height = backingHeight;
      }

      context.setTransform(dpr, 0, 0, dpr, 0, 0);
    };

    const drawGrid = (): void => {
      const { width, height } = viewportRef.current;

      context.clearRect(0, 0, width, height);
      context.fillStyle = BACKGROUND_COLOR;
      context.fillRect(0, 0, width, height);

      context.strokeStyle = GRID_COLOR;
      context.lineWidth = 1;

      const horizontalSteps = 4;
      const verticalSteps = 8;

      for (let i = 1; i < horizontalSteps; i += 1) {
        const y = (height * i) / horizontalSteps;
        context.beginPath();
        context.moveTo(0, y);
        context.lineTo(width, y);
        context.stroke();
      }

      for (let i = 1; i < verticalSteps; i += 1) {
        const x = (width * i) / verticalSteps;
        context.beginPath();
        context.moveTo(x, 0);
        context.lineTo(x, height);
        context.stroke();
      }

      context.strokeStyle = AXIS_COLOR;
      context.beginPath();
      context.moveTo(0, height / 2);
      context.lineTo(width, height / 2);
      context.stroke();
    };

    const drawWave = (points: number[], color: string): void => {
      if (points.length < 2) {
        return;
      }

      context.strokeStyle = color;
      context.lineWidth = 1.5;
      context.beginPath();

      const { width, height } = viewportRef.current;
      const drawableWidth = Math.max(1, width - PADDING * 2);
      const drawableHeight = Math.max(1, height - PADDING * 2);

      for (let index = 0; index < points.length; index += 1) {
        const x = PADDING + (index / (points.length - 1)) * drawableWidth;
        const sample = points[index] ?? 0;
        const y = PADDING + (1 - (sample + 1) * 0.5) * drawableHeight;

        if (index === 0) {
          context.moveTo(x, y);
        } else {
          context.lineTo(x, y);
        }
      }

      context.stroke();
    };

    const render = (): void => {
      if (needsResizeRef.current || getDevicePixelRatio() !== viewportRef.current.dpr) {
        syncCanvasViewport();
      }

      drawGrid();

      const { width, height } = viewportRef.current;

      const latest = frameRef.current;
      if (!connected || !latest || latest.no_signal) {
        context.strokeStyle = AXIS_COLOR;
        context.lineWidth = 2;
        context.beginPath();
        context.moveTo(PADDING, height / 2);
        context.lineTo(width - PADDING, height / 2);
        context.stroke();
      } else {
        if (channelView === 'overlay' || channelView === 'left') {
          drawWave(latest.points_l, LEFT_COLOR);
        }

        if (channelView === 'overlay' || channelView === 'right') {
          drawWave(latest.points_r, RIGHT_COLOR);
        }
      }

      rafRef.current = requestAnimationFrame(render);
    };

    syncCanvasViewport();

    const resizeObserver =
      typeof ResizeObserver !== 'undefined'
        ? new ResizeObserver(() => {
            needsResizeRef.current = true;
          })
        : null;
    resizeObserver?.observe(canvas);

    const handleWindowResize = (): void => {
      needsResizeRef.current = true;
    };
    window.addEventListener('resize', handleWindowResize);

    rafRef.current = requestAnimationFrame(render);

    return (): void => {
      resizeObserver?.disconnect();
      window.removeEventListener('resize', handleWindowResize);

      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current);
      }
    };
  }, [channelView, connected]);

  return (
    <>
      <Row className="flex flex-wrap gap-2">
        <Select
          label="Channel view"
          data-testid="osc-channel-view"
          value={channelView}
          size="sm"
          options={[
            { label: 'Overlay (L/R)', value: 'overlay' },
            { label: 'Left', value: 'left' },
            { label: 'Right', value: 'right' },
          ]}
          onChange={setChannelView}
        />
        <Select
          label="Trigger mode"
          data-testid="osc-trigger-mode"
          value={triggerMode}
          size="sm"
          options={[{ label: 'Rising zero-crossing', value: 'risingZeroCrossing' }]}
          onChange={setTriggerMode}
        />
      </Row>
      <div className="relative h-full w-full">
        <canvas
          ref={canvasRef}
          data-testid="oscilloscope-canvas"
          className="h-full w-full rounded"
        />
        {noSignal ? (
          <div
            data-testid="osc-no-signal"
            className="pointer-events-none absolute inset-0 flex items-center justify-center text-sm font-medium text-gray-400"
          >
            No signal
          </div>
        ) : null}
      </div>
    </>
  );
}
