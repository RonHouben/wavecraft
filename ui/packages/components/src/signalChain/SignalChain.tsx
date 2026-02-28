/**
 * SignalChain - Drag-and-drop ordered list of processor and tap cards
 *
 * Renders slot cards in the server-authoritative unified order (processors +
 * taps) and allows the user to reorder them via drag-and-drop or keyboard.
 *
 * Uses @dnd-kit for accessible DnD with full keyboard support.
 */

import {
    closestCenter,
    DndContext,
    DragOverlay,
    KeyboardSensor,
    PointerSensor,
    useSensor,
    useSensors,
    type DragEndEvent,
    type DragStartEvent,
} from '@dnd-kit/core';
import {
    arrayMove,
    SortableContext,
    sortableKeyboardCoordinates,
    verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import type { SignalChainOrder, SlotType } from '@wavecraft/core';
import { useSignalChainOrder } from '@wavecraft/core';
import React, { useCallback, useMemo, useRef, useState } from 'react';

import { mergeClassNames } from '../utils/classNames';
import { SignalChainItem } from './SignalChainItem';
import type { SignalChainEntry } from './types';
import { useSortedEntries } from './useSignalChainPresentation';

export interface SignalChainProps {
  /** Ordered list of signal chain entries; each entry has an `id`, `type`, and a `component` */
  entries: SignalChainEntry[];
  className?: string;
}

export function SignalChain({ entries, className }: Readonly<SignalChainProps>): React.JSX.Element {
  // Track drag state — used to suppress incoming IPC notifications during drag
  const isDraggingRef = useRef(false);
  const [activeId, setActiveId] = useState<string | null>(null);

  const { order, setOrder } = useSignalChainOrder(isDraggingRef);

  // Compute the display-sorted entries from the IPC order
  const sortedEntries = useSortedEntries(entries, order);

  // Build a stable id → SlotType map for translating DnD result back to IPC slots
  const idToType = useMemo(
    () => new Map<string, SlotType>(entries.map((e) => [e.id, e.type])),
    [entries]
  );

  // DnD item IDs in current display order (entry `id` strings)
  const items = useMemo(() => sortedEntries.map((e) => e.id), [sortedEntries]);

  // Build a map for O(1) child lookup
  const entryMap = useMemo(
    () => new Map<string, React.ReactNode>(entries.map((e) => [e.id, e.component])),
    [entries]
  );

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragStart = useCallback((event: DragStartEvent) => {
    isDraggingRef.current = true;
    setActiveId(String(event.active.id));
  }, []);

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      isDraggingRef.current = false;
      setActiveId(null);

      const { active, over } = event;
      if (!over || active.id === over.id) return;

      const oldIndex = items.indexOf(String(active.id));
      const newIndex = items.indexOf(String(over.id));
      if (oldIndex === -1 || newIndex === -1) return;

      const reorderedIds = arrayMove(items, oldIndex, newIndex);

      // Translate entry IDs back to SignalChainOrder slot objects for the IPC call
      const newSlots: SignalChainOrder[] = reorderedIds
        .map((id) => {
          const type = idToType.get(id);
          return type ? { id, type } : null;
        })
        .filter((s): s is SignalChainOrder => s !== null);

      void setOrder(newSlots);
    },
    [items, idToType, setOrder]
  );

  const handleDragCancel = useCallback(() => {
    isDraggingRef.current = false;
    setActiveId(null);
  }, []);

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={handleDragCancel}
    >
      <SortableContext items={items} strategy={verticalListSortingStrategy}>
        <div
          className={mergeClassNames('flex flex-col gap-2', className)}
          role="list"
          aria-label="Signal chain processor order"
        >
          {sortedEntries.map((entry) => (
            <div key={entry.id} role="listitem">
              <SignalChainItem id={entry.id} isDragging={activeId === entry.id}>
                {entryMap.get(entry.id)}
              </SignalChainItem>
            </div>
          ))}
        </div>
      </SortableContext>

      {/* Drag overlay — renders a ghost of the dragged item */}
      <DragOverlay>
        {activeId ? <div className="opacity-90 shadow-panel">{entryMap.get(activeId)}</div> : null}
      </DragOverlay>
    </DndContext>
  );
}
