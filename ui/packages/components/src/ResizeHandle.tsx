/**
 * ResizeHandle component - Bottom-right corner resize grip
 *
 * Provides a visual affordance for freeform window resizing.
 * Captures mouse drag events and communicates size changes to the host.
 */

import React, { useCallback, useRef, useState } from 'react';
import { focusRingClass, interactionStateClass } from './utils/classNames';

const MIN_WIDTH = 400;
const MIN_HEIGHT = 300;
const KEYBOARD_RESIZE_STEP = 24;
const KEYBOARD_RESIZE_STEP_LARGE = 64;

export interface ResizeHandleProps {
  readonly onRequestResize: (width: number, height: number) => Promise<boolean>;
}

export function ResizeHandle({ onRequestResize }: Readonly<ResizeHandleProps>): React.JSX.Element {
  const [isDragging, setIsDragging] = useState(false);
  const dragStartRef = useRef({ x: 0, y: 0, width: 0, height: 0 });

  const requestResizeWithDelta = useCallback(
    (deltaWidth: number, deltaHeight: number): void => {
      const newWidth = Math.max(MIN_WIDTH, window.innerWidth + deltaWidth);
      const newHeight = Math.max(MIN_HEIGHT, window.innerHeight + deltaHeight);
      void onRequestResize(newWidth, newHeight);
    },
    [onRequestResize]
  );

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      setIsDragging(true);

      // Capture initial state
      dragStartRef.current = {
        x: e.clientX,
        y: e.clientY,
        width: window.innerWidth,
        height: window.innerHeight,
      };

      const handleMouseMove = (moveEvent: MouseEvent): void => {
        const deltaX = moveEvent.clientX - dragStartRef.current.x;
        const deltaY = moveEvent.clientY - dragStartRef.current.y;

        const newWidth = Math.max(MIN_WIDTH, dragStartRef.current.width + deltaX);
        const newHeight = Math.max(MIN_HEIGHT, dragStartRef.current.height + deltaY);

        // Request resize from host
        void onRequestResize(newWidth, newHeight);
      };

      const handleMouseUp = (): void => {
        setIsDragging(false);
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
      };

      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    },
    [onRequestResize]
  );

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLButtonElement>): void => {
      const step = event.shiftKey ? KEYBOARD_RESIZE_STEP_LARGE : KEYBOARD_RESIZE_STEP;

      switch (event.key) {
        case 'ArrowRight':
          event.preventDefault();
          requestResizeWithDelta(step, 0);
          break;
        case 'ArrowLeft':
          event.preventDefault();
          requestResizeWithDelta(-step, 0);
          break;
        case 'ArrowDown':
          event.preventDefault();
          requestResizeWithDelta(0, step);
          break;
        case 'ArrowUp':
          event.preventDefault();
          requestResizeWithDelta(0, -step);
          break;
        default:
          break;
      }
    },
    [requestResizeWithDelta]
  );

  return (
    <button
      data-testid="resize-handle"
      className={`group fixed bottom-2 right-2 z-[9999] flex h-11 w-11 cursor-nwse-resize select-none items-center justify-center rounded-md border border-plugin-border bg-plugin-surface p-0 shadow-sm ${focusRingClass} ${interactionStateClass} ${
        isDragging ? 'border-accent/60 bg-accent/25' : 'hover:border-white/35 hover:bg-white/15'
      }`}
      onMouseDown={handleMouseDown}
      onKeyDown={handleKeyDown}
      aria-label="Resize window"
      type="button"
    >
      <svg
        width="22"
        height="22"
        viewBox="0 0 16 16"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className={`transition-colors duration-150 ${
          isDragging ? 'text-accent-light' : 'text-white/75 group-hover:text-white'
        }`}
      >
        {/* Diagonal grip lines */}
        <line
          x1="14"
          y1="2"
          x2="2"
          y2="14"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
        <line
          x1="14"
          y1="6"
          x2="6"
          y2="14"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
        <line
          x1="14"
          y1="10"
          x2="10"
          y2="14"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
    </button>
  );
}
