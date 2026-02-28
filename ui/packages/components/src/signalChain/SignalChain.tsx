/**
 * SignalChain - Drag-and-drop ordered list of processor cards
 *
 * Renders processor cards in the server-authoritative order and allows the
 * user to reorder them via drag-and-drop or keyboard navigation.
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
import { useProcessorOrder } from '@wavecraft/core';
import React, { useCallback, useMemo, useRef, useState } from 'react';

import { mergeClassNames } from '../utils/classNames';
import { SignalChainItem } from './SignalChainItem';
import type { SignalChainProcessorEntry } from './types';
import { useSortedProcessors } from './useSignalChainPresentation';

export interface SignalChainProps {
  /** Ordered list of processor entries; each entry has an `id` and a `component` to render */
  processors: SignalChainProcessorEntry[];
  className?: string;
}

export function SignalChain({
  processors,
  className,
}: Readonly<SignalChainProps>): React.JSX.Element {
  // Track drag state — used to suppress incoming IPC notifications during drag
  const isDraggingRef = useRef(false);
  const [activeId, setActiveId] = useState<string | null>(null);

  const { order, setOrder } = useProcessorOrder(isDraggingRef);

  // Compute the display-sorted processors from the IPC order
  const sortedProcessors = useSortedProcessors(processors, order);

  // Build a stable id→slotIndex map for translating DnD result back to IPC order
  const idToSlotIndex = useMemo(
    () => new Map<string, string>(processors.map((p, i) => [p.id, String(i)])),
    [processors]
  );

  // DnD item IDs in current display order (processor `id` strings)
  const items = useMemo(() => sortedProcessors.map((p) => p.id), [sortedProcessors]);

  // Build a map for O(1) child lookup
  const processorMap = useMemo(
    () => new Map<string, React.ReactNode>(processors.map((p) => [p.id, p.component])),
    [processors]
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

      // Translate processor IDs back to slot indices for the IPC call
      const newOrder = reorderedIds
        .map((id) => idToSlotIndex.get(id))
        .filter((slot): slot is string => slot !== undefined);

      void setOrder(newOrder);
    },
    [items, idToSlotIndex, setOrder]
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
          {sortedProcessors.map((entry) => (
            <div key={entry.id} role="listitem">
              <SignalChainItem id={entry.id} isDragging={activeId === entry.id}>
                {processorMap.get(entry.id)}
              </SignalChainItem>
            </div>
          ))}
        </div>
      </SortableContext>

      {/* Drag overlay — renders a ghost of the dragged item */}
      <DragOverlay>
        {activeId ? (
          <div className="opacity-90 shadow-panel">{processorMap.get(activeId)}</div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}
