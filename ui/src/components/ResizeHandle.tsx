/**
 * ResizeHandle component - Bottom-right corner resize grip
 *
 * Provides a visual affordance for freeform window resizing.
 * Captures mouse drag events and communicates size changes to the host.
 */

import React, { useCallback, useRef, useState } from 'react';
import { useRequestResize } from '../lib/vstkit-ipc';

export function ResizeHandle(): React.JSX.Element {
  const requestResize = useRequestResize();
  const [isDragging, setIsDragging] = useState(false);
  const dragStartRef = useRef({ x: 0, y: 0, width: 0, height: 0 });

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

        const newWidth = Math.max(400, dragStartRef.current.width + deltaX);
        const newHeight = Math.max(300, dragStartRef.current.height + deltaY);

        // Request resize from host
        requestResize(newWidth, newHeight).catch((err) => {
          console.error('Resize request failed:', err);
        });
      };

      const handleMouseUp = (): void => {
        setIsDragging(false);
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
      };

      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    },
    [requestResize]
  );

  return (
    <button
      className={`group fixed bottom-0 right-0 z-[9999] flex h-6 w-6 cursor-nwse-resize select-none items-center justify-center border-none bg-transparent p-0 transition-colors duration-150 ${
        isDragging ? 'bg-white/10' : 'hover:bg-white/5'
      }`}
      onMouseDown={handleMouseDown}
      aria-label="Resize window"
      type="button"
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className={`transition-colors duration-150 ${
          isDragging ? 'text-white/80' : 'text-white/30 group-hover:text-white/60'
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
