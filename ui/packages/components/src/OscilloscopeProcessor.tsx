import React, { useEffect, useMemo, useRef, useState } from 'react';
import {
  useConnectionStatus,
  useOscilloscopeFrame,
  useHasProcessorInSignalChain,
  type OscilloscopeChannelView,
  type OscilloscopeFrame,
  type OscilloscopeTriggerMode,
} from '@wavecraft/core';

const WIDTH = 640;
const HEIGHT = 220;
const PADDING = 10;

const LEFT_COLOR = '#22c55e';
const RIGHT_COLOR = '#3b82f6';
const GRID_COLOR = '#334155';
const AXIS_COLOR = '#64748b';

interface OscilloscopeProcessorProps {
  hideWhenNotInSignalChain?: boolean;
}

export function OscilloscopeProcessor({
  hideWhenNotInSignalChain,
}: Readonly<OscilloscopeProcessorProps>): React.JSX.Element | null {
  const { connected } = useConnectionStatus();
  const frame = useOscilloscopeFrame();
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('oscilloscope_tap');

  const [channelView, setChannelView] = useState<OscilloscopeChannelView>('overlay');
  const [triggerMode, setTriggerMode] = useState<OscilloscopeTriggerMode>('risingZeroCrossing');

  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const frameRef = useRef<OscilloscopeFrame | null>(null);
  const rafRef = useRef<number | null>(null);

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

    const drawGrid = (): void => {
      context.clearRect(0, 0, WIDTH, HEIGHT);
      context.fillStyle = '#111827';
      context.fillRect(0, 0, WIDTH, HEIGHT);

      context.strokeStyle = GRID_COLOR;
      context.lineWidth = 1;

      const horizontalSteps = 4;
      const verticalSteps = 8;

      for (let i = 1; i < horizontalSteps; i += 1) {
        const y = (HEIGHT * i) / horizontalSteps;
        context.beginPath();
        context.moveTo(0, y);
        context.lineTo(WIDTH, y);
        context.stroke();
      }

      for (let i = 1; i < verticalSteps; i += 1) {
        const x = (WIDTH * i) / verticalSteps;
        context.beginPath();
        context.moveTo(x, 0);
        context.lineTo(x, HEIGHT);
        context.stroke();
      }

      context.strokeStyle = AXIS_COLOR;
      context.beginPath();
      context.moveTo(0, HEIGHT / 2);
      context.lineTo(WIDTH, HEIGHT / 2);
      context.stroke();
    };

    const drawWave = (points: number[], color: string): void => {
      if (points.length < 2) {
        return;
      }

      context.strokeStyle = color;
      context.lineWidth = 1.5;
      context.beginPath();

      const drawableWidth = WIDTH - PADDING * 2;
      const drawableHeight = HEIGHT - PADDING * 2;

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
      drawGrid();

      const latest = frameRef.current;
      if (!connected || !latest || latest.no_signal) {
        context.strokeStyle = AXIS_COLOR;
        context.lineWidth = 2;
        context.beginPath();
        context.moveTo(PADDING, HEIGHT / 2);
        context.lineTo(WIDTH - PADDING, HEIGHT / 2);
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

    rafRef.current = requestAnimationFrame(render);

    return (): void => {
      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current);
      }
    };
  }, [channelView, connected]);

  if (hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  return (
    <div
      data-testid="oscilloscope"
      className="rounded-lg border border-plugin-border bg-plugin-surface p-4"
    >
      <div className="mb-3 flex flex-wrap items-center gap-3">
        <div className="text-xs font-semibold uppercase tracking-wide text-gray-400">
          Oscilloscope
        </div>

        <label className="flex items-center gap-2 text-xs text-gray-300" htmlFor="osc-view-select">
          Channel view
          <select
            id="osc-view-select"
            data-testid="osc-channel-view"
            className="rounded border border-plugin-border bg-plugin-dark px-2 py-1 text-xs text-gray-200"
            value={channelView}
            onChange={(event) => {
              setChannelView(event.target.value as OscilloscopeChannelView);
            }}
          >
            <option value="overlay">Overlay (L/R)</option>
            <option value="left">Left</option>
            <option value="right">Right</option>
          </select>
        </label>

        <label
          className="flex items-center gap-2 text-xs text-gray-300"
          htmlFor="osc-trigger-mode-select"
        >
          Trigger mode
          <select
            id="osc-trigger-mode-select"
            data-testid="osc-trigger-mode"
            className="rounded border border-plugin-border bg-plugin-dark px-2 py-1 text-xs text-gray-200"
            value={triggerMode}
            onChange={(event) => {
              setTriggerMode(event.target.value as OscilloscopeTriggerMode);
            }}
          >
            <option value="risingZeroCrossing">Rising zero-crossing</option>
          </select>
        </label>
      </div>

      <div className="relative rounded border border-plugin-border bg-plugin-dark p-2">
        <canvas
          ref={canvasRef}
          data-testid="oscilloscope-canvas"
          width={WIDTH}
          height={HEIGHT}
          className="h-auto w-full rounded"
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
    </div>
  );
}
