/**
 * SignalChainItem - Draggable wrapper for a single processor card
 */

import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import React from 'react';
import { mergeClassNames } from '../utils/classNames';

export interface SignalChainItemProps {
  /** Processor ID — used as the DnD sort key */
  id: string;
  /** Whether this item is currently being dragged */
  isDragging: boolean;
  children: React.ReactNode;
}

export function SignalChainItem({
  id,
  isDragging,
  children,
}: Readonly<SignalChainItemProps>): React.JSX.Element {
  const { attributes, listeners, setNodeRef, transform, transition } = useSortable({ id });

  const style: React.CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div
      ref={setNodeRef}
      data-slot-id={id}
      data-testid={`signal-chain-item-${id}`}
      style={style}
      className={mergeClassNames(
        'flex items-stretch gap-2 transition-opacity duration-150',
        isDragging ? 'opacity-50' : 'opacity-100'
      )}
    >
      {/* Drag handle */}
      <button
        type="button"
        data-testid={`signal-chain-handle-${id}`}
        className={mergeClassNames(
          'flex shrink-0 cursor-grab items-center justify-center rounded-lg px-1.5',
          'text-plugin-text-muted hover:text-plugin-text-secondary',
          'hover:bg-plugin-surface-2 focus-visible:outline-none focus-visible:ring-2',
          'focus-visible:ring-plugin-focus focus-visible:ring-offset-1',
          'focus-visible:ring-offset-plugin-canvas active:cursor-grabbing',
          isDragging ? 'cursor-grabbing' : 'cursor-grab'
        )}
        aria-label="Drag to reorder"
        {...attributes}
        {...listeners}
      >
        {/* 6-dot grip icon */}
        <svg
          aria-hidden="true"
          focusable="false"
          width="10"
          height="16"
          viewBox="0 0 10 16"
          fill="currentColor"
          xmlns="http://www.w3.org/2000/svg"
        >
          <circle cx="2.5" cy="2.5" r="1.5" />
          <circle cx="7.5" cy="2.5" r="1.5" />
          <circle cx="2.5" cy="8" r="1.5" />
          <circle cx="7.5" cy="8" r="1.5" />
          <circle cx="2.5" cy="13.5" r="1.5" />
          <circle cx="7.5" cy="13.5" r="1.5" />
        </svg>
      </button>

      {/* Processor content */}
      <div className="min-w-0 flex-1">{children}</div>
    </div>
  );
}
