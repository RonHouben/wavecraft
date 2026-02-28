---

# UI Design Specification: SignalChain Component

## Research Summary

- **Design tokens** live in `ui/tailwind.config.js`. Semantic color roles: `plugin-canvas`, `plugin-surface-1`, `plugin-surface-2`, `plugin-border`, `plugin-border-strong`, `plugin-text-primary`, `plugin-text-secondary`, `plugin-text-muted`, `plugin-focus`. Typography scale: `text-type-2xs` through `text-type-lg`. Shadows: `shadow-control`, `shadow-panel`, `shadow-focus-ring`.
- **ProcessorCard** (`ui/packages/components/src/processors/ProcessorCard.tsx`) wraps `Card` with a bypass `Switch` in the header — it already handles the bypassed visual state (opacity/saturation dim).
- **No drag-and-drop library exists** in the current dependency set (React 18, lodash, clsx only).
- **IPC pattern**: `IpcBridge.invoke()` for request/response; `IpcBridge.on<event>()` for notifications. `ParameterClient` is the established singleton pattern to follow.
- **Hook pattern**: hooks use `useSyncExternalStore` (registry hooks) or polling + notification subscription (`useParameter`).
- **`ProcessorId`** is a branded string type derived from the generated `WavecraftProcessorIdMap`. All processors are runtime-registered.
- **Existing utility classes**: `focusRingClass`, `interactionStateClass`, `mergeClassNames` in `classNames.ts`.

---

## 1. Layout Direction: Vertical

**Recommendation: Vertical stack (top-to-bottom).**

**Rationale:**

- The conventional mental model for an audio signal chain is top-to-bottom flow (input → processing → output). A vertical list makes this immediately legible.
- Each processor card is _expanded_ with all parameters visible inline. Horizontal layout would require either very wide plugin windows or horizontal scrolling — both poor UX in a fixed-size plugin window.
- Processors vary in height. Vertical stacking accommodates variable heights naturally.

---

## 2. Component Tree

```
App (sdk-template/ui/src/App.tsx)
└── WavecraftProvider
    └── SignalChain                            (@wavecraft/components)
        ├── [loading state]  → SignalChainSkeleton (N placeholder cards)
        ├── [error state]    → ErrorMessage component (existing)
        └── DndContext                         (@dnd-kit/core)
            ├── SortableContext                (@dnd-kit/sortable)
            │   └── SignalChainItem × N        (@wavecraft/components)
            │       ├── DragHandle             (button, inline SVG grip icon)
            │       └── {ProcessorComponent}   (resolved via processors component lookup map)
            └── DragOverlay                    (ghost clone during drag)
```

---

## 3. Drag-and-Drop Library: `@dnd-kit`

**Recommendation: `@dnd-kit/core` + `@dnd-kit/sortable` + `@dnd-kit/utilities`**

### Why not the native HTML Drag and Drop API?

| Concern                        | Native HTML DnD                                                      | `@dnd-kit`                                     |
| ------------------------------ | -------------------------------------------------------------------- | ---------------------------------------------- |
| Keyboard reorder support       | ❌ None — must build from scratch                                    | ✅ Built in (Space/Enter, arrows, Escape)      |
| WKWebView compatibility        | ⚠️ Known quirks with `draggable`, ghost images, silent drop failures | ✅ Uses `PointerEvent` — reliable in WKWebView |
| Sort animation (items sliding) | ❌ No coordinate data during drag                                    | ✅ Live transform coordinates                  |
| Drag ghost customisation       | ⚠️ Requires `setDragImage()` with canvas                             | ✅ Full React `DragOverlay`                    |
| Bundle cost                    | 0KB                                                                  | ~8KB gzipped                                   |

The native API is technically viable for mouse-only desktop use, but requires significant custom code for keyboard accessibility and sort animation, and has reliability issues in WKWebView (the production render target for Wavecraft).

**Packages to add** to `ui/packages/components/package.json`:

```
@dnd-kit/core    @dnd-kit/sortable    @dnd-kit/utilities
```

---

## 4. `SignalChain` Component API

### Types

```typescript
export interface SignalChainProcessorEntry {
  /** The processor's stable ID, matching the engine-registered ProcessorId. */
  readonly id: ProcessorId;
  /** The rendered React component for this processor. */
  readonly component: React.ReactNode;
}
```

### Props

```typescript
export interface SignalChainProps {
  /**
   * Array of processor entries. Each entry pairs a ProcessorId with its React component.
   * The array order is used as the fallback initial order before the engine responds.
   * The engine's persisted order takes precedence once loaded.
   */
  readonly processors: SignalChainProcessorEntry[];

  readonly className?: string;
}
```

### Usage in `sdk-template/ui/src/App.tsx`

```tsx
<SignalChain
  processors={[
    { id: 'test_tone', component: <TestToneProcessor /> },
    { id: 'tone_filter', component: <ToneFilterProcessor /> },
    { id: 'soft_clip', component: <SoftClipProcessor /> },
    { id: 'output_gain', component: <OutputGainProcessor /> }
  ]}
  className="px-4 py-3"
/>
```

**Key design decisions:**

- `processors` is a single prop that collocates each processor's ID with its React component — no separate render function needed.
- Internally, `SignalChain` builds a `Map<ProcessorId, React.ReactNode>` from the array on mount for O(1) lookup during render.
- The array order is the **fallback initial order** used before the engine responds. Once the engine returns a persisted order via `getProcessorOrder`, that order is used exclusively.
- `renderProcessor` (previously proposed as a separate prop) is removed — replaced by the component lookup map derived from `processors`.
- The `processors` array should be defined outside the render function or wrapped in `useMemo` at the call site. Passing an inline array literal on every render causes `SignalChain`'s internal `useMemo` to rebuild the component map on every render cycle, defeating its purpose.

---

## 5. New Artifacts Needed

### 5.1 `@wavecraft/core` additions

**`ui/packages/core/src/ipc/constants.ts`** — extend `IpcMethods` and `IpcEvents`:

```typescript
GET_PROCESSOR_ORDER: 'getProcessorOrder',
SET_PROCESSOR_ORDER: 'setProcessorOrder',
PROCESSOR_ORDER_CHANGED: 'processorOrderChanged',
```

**`ui/packages/core/src/types/processorOrder.ts`** — new file:

```typescript
export interface GetProcessorOrderResult {
  order: ProcessorId[];
}

export interface SetProcessorOrderParams {
  order: ProcessorId[];
}

export interface ProcessorOrderChangedNotification {
  order: ProcessorId[];
}
```

**`ui/packages/core/src/ipc/ProcessorOrderClient.ts`** — new singleton, separate from `ParameterClient`:

```typescript
export class ProcessorOrderClient {
  private static instance: ProcessorOrderClient | null = null;
  private readonly bridge: IpcBridge;

  private constructor() { ... }
  public static getInstance(): ProcessorOrderClient { ... }

  public async getProcessorOrder(): Promise<ProcessorId[]> { ... }
  public async setProcessorOrder(order: ProcessorId[]): Promise<void> { ... }
  public onOrderChanged(callback: (order: ProcessorId[]) => void): () => void { ... }
}
```

**`ui/packages/core/src/hooks/useProcessorOrder.ts`** — new hook:

```typescript
export interface UseProcessorOrderOptions {
  /** Fallback order (ProcessorIds) used before the engine responds. Derived from the processors prop. */
  fallbackOrder: ProcessorId[];
  /** Ref owned by SignalChain, set true during active drags to defer incoming notifications. */
  isDraggingRef: React.RefObject<boolean>;
}

export interface UseProcessorOrderResult {
  order: ProcessorId[];
  isLoading: boolean;
  error: Error | null;
  reorder: (fromIndex: number, toIndex: number) => Promise<void>;
}
```

### 5.2 `@wavecraft/components` additions

| File                                                             | Description                                                                                                              |
| ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `ui/packages/components/src/signalChain/SignalChain.tsx`         | Container component, DnD context, loading/error states                                                                   |
| `ui/packages/components/src/signalChain/SignalChainItem.tsx`     | Single sortable chain item                                                                                               |
| `ui/packages/components/src/signalChain/SignalChainSkeleton.tsx` | Loading placeholder                                                                                                      |
| `ui/packages/components/src/signalChain/DragHandleIcon.tsx`      | Inline SVG grip icon (scoped to SignalChain)                                                                             |
| `ui/packages/components/src/signalChain/index.ts`                | Barrel export: `SignalChain`, `SignalChainItem`, `SignalChainProcessorEntry`, `SignalChainProps`, `SignalChainItemProps` |
| Add re-exports to `ui/packages/components/src/index.ts`          | `SignalChain`, `SignalChainItem`                                                                                         |
| Add exports to `ui/packages/core/src/index.ts`                   | hook, client, types                                                                                                      |

**`SignalChainItemProps`** — defined in `SignalChainItem.tsx`:

```typescript
export interface SignalChainItemProps {
  /** The processor's stable ID — used as the dnd-kit sort key. */
  readonly processorId: ProcessorId;
  /** 1-based position in the chain, used for ARIA announcements. */
  readonly index: number;
  /** Total number of processors in the chain, used for ARIA announcements. */
  readonly totalCount: number;
  /** The rendered processor component. */
  readonly children: React.ReactNode;
  /** When true, disables drag interaction for this item. Reserved for future use. */
  readonly isDragDisabled?: boolean;
}
```

---

## 6. Handling `processorOrderChanged` During Active Drags

`SignalChain` owns an `isDraggingRef` (`useRef<boolean>(false)`). It passes this ref to `useProcessorOrder`. The hook defers incoming `processorOrderChanged` notifications while `isDraggingRef.current === true`, buffering them in a `pendingOrderRef`. On drag end, the buffered order is applied.

```
SignalChain
  isDraggingRef = useRef(false)    ← SignalChain owns the ref
        │
        └─► passed to useProcessorOrder({ isDraggingRef, fallbackOrder })
                  │
                  └─► reads isDraggingRef.current in the onOrderChanged subscription
```

`onDragStart` sets `isDraggingRef.current = true`. `onDragEnd` and `onDragCancel` set it to `false` and flush any buffered order.

---

## 7. Skeleton Code Outline

### `SignalChain.tsx`

```tsx
export function SignalChain({
  processors,
  className
}: SignalChainProps): JSX.Element {
  // Build stable component lookup map from the processors prop
  const componentMap = useMemo(
    () => new Map(processors.map(({ id, component }) => [id, component])),
    [processors]
  );
  const fallbackOrder = useMemo(
    () => processors.map(({ id }) => id),
    [processors]
  );

  const isDraggingRef = useRef(false);
  const { order, isLoading, error, reorder } = useProcessorOrder({
    fallbackOrder,
    isDraggingRef
  });
  const [activeId, setActiveId] = useState<ProcessorId | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  );

  if (isLoading) return <SignalChainSkeleton count={processors.length} />;
  if (error)
    return (
      <ErrorMessage
        title="Failed to load signal chain"
        message={error.message}
      />
    );

  function handleDragStart(event: DragStartEvent) {
    isDraggingRef.current = true;
    setActiveId(event.active.id as ProcessorId);
  }

  function handleDragEnd(event: DragEndEvent) {
    isDraggingRef.current = false;
    setActiveId(null);
    const { active, over } = event;
    if (over && active.id !== over.id) {
      const fromIndex = order.indexOf(active.id as ProcessorId);
      const toIndex = order.indexOf(over.id as ProcessorId);
      void reorder(fromIndex, toIndex);
    }
  }

  function handleDragCancel() {
    isDraggingRef.current = false;
    setActiveId(null);
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={handleDragCancel}
      accessibility={{ announcements: buildAnnouncements(order) }}
    >
      <p id="signal-chain-dnd-instructions" className="sr-only">
        To reorder: press Space or Enter to pick up, use arrow keys to move,
        Space or Enter to drop, Escape to cancel.
      </p>
      <SortableContext items={order} strategy={verticalListSortingStrategy}>
        <ol
          aria-label="Signal chain"
          className={mergeClassNames('flex flex-col gap-3 pl-8', className)}
        >
          {order.map((processorId, index) => (
            <SignalChainItem
              key={processorId}
              processorId={processorId}
              index={index + 1}
              totalCount={order.length}
            >
              {componentMap.get(processorId)}
            </SignalChainItem>
          ))}
        </ol>
      </SortableContext>
      <DragOverlay>
        {activeId ? (
          <div className="opacity-90 shadow-panel">
            {componentMap.get(activeId)}
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}
```

### `SignalChainItem.tsx`

```tsx
export function SignalChainItem({
  processorId,
  index,
  totalCount,
  children,
  isDragDisabled = false
}: SignalChainItemProps): JSX.Element {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging
  } = useSortable({ id: processorId, disabled: isDragDisabled });

  return (
    <li
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      aria-roledescription="sortable"
      aria-label={`Processor ${index} of ${totalCount}`}
      className={mergeClassNames('relative', isDragging && 'opacity-40')}
    >
      <button
        type="button"
        aria-label="Drag to reorder"
        aria-describedby="signal-chain-dnd-instructions"
        className={mergeClassNames(
          'absolute -left-6 top-1/2 -translate-y-1/2',
          'flex h-6 w-6 items-center justify-center rounded',
          'text-plugin-text-muted hover:text-plugin-text-secondary',
          'cursor-grab active:cursor-grabbing',
          focusRingClass,
          isDragDisabled && 'cursor-not-allowed opacity-40'
        )}
        {...attributes}
        {...listeners}
      >
        <DragHandleIcon aria-hidden="true" />
      </button>
      {children}
    </li>
  );
}
```

### `DragHandleIcon.tsx`

```tsx
export function DragHandleIcon({
  className
}: {
  className?: string;
}): JSX.Element {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="currentColor"
      aria-hidden="true"
      focusable="false"
      className={mergeClassNames('h-4 w-4 shrink-0', className)}
    >
      <circle cx="5" cy="4" r="1.2" />
      <circle cx="5" cy="8" r="1.2" />
      <circle cx="5" cy="12" r="1.2" />
      <circle cx="11" cy="4" r="1.2" />
      <circle cx="11" cy="8" r="1.2" />
      <circle cx="11" cy="12" r="1.2" />
    </svg>
  );
}
```

### `SignalChainSkeleton.tsx`

```tsx
export function SignalChainSkeleton({
  count = 3
}: {
  count?: number;
}): JSX.Element {
  return (
    <ol aria-label="Loading signal chain" className="flex flex-col gap-3">
      {Array.from({ length: count }, (_, i) => (
        <li key={i}>
          <div
            aria-hidden="true"
            className="h-24 w-full animate-pulse rounded-xl border border-plugin-border bg-plugin-surface-1"
          />
        </li>
      ))}
    </ol>
  );
}
```

### `useProcessorOrder.ts` (outline)

```typescript
export function useProcessorOrder({
  fallbackOrder,
  isDraggingRef
}: UseProcessorOrderOptions): UseProcessorOrderResult {
  const [order, setOrder] = useState<ProcessorId[]>(fallbackOrder);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const pendingOrderRef = useRef<ProcessorId[] | null>(null);

  useEffect(() => {
    const client = ProcessorOrderClient.getInstance();

    client
      .getProcessorOrder()
      .then((newOrder) => {
        setOrder(newOrder);
        setIsLoading(false);
      })
      .catch((err) => {
        setError(err instanceof Error ? err : new Error(String(err)));
        setIsLoading(false);
      });

    const unsubscribe = client.onOrderChanged((newOrder) => {
      if (isDraggingRef.current) {
        pendingOrderRef.current = newOrder; // defer until drag ends
      } else {
        setOrder(newOrder);
      }
    });

    return unsubscribe;
  }, [isDraggingRef]);

  const reorder = useCallback(async (fromIndex: number, toIndex: number) => {
    // 1. Optimistic local splice
    // 2. Flush any buffered order from pendingOrderRef
    // 3. IPC setProcessorOrder with new order
  }, []);

  return { order, isLoading, error, reorder };
}
```

> **Note for Coder:** The `reorder` callback must capture the post-splice array before calling `setProcessorOrder`. Use a local variable or a ref to avoid stale closure issues.

---

## 8. Design Token Usage

All visual values use existing tokens. No ad-hoc values introduced.

| UI element                 | Token usage                                                                                   |
| -------------------------- | --------------------------------------------------------------------------------------------- |
| Chain item card            | Existing `ProcessorCard` → `bg-plugin-surface-1 border-plugin-border rounded-xl shadow-panel` |
| Drag handle icon           | `text-plugin-text-muted hover:text-plugin-text-secondary`                                     |
| Drag handle focus ring     | `focusRingClass` constant                                                                     |
| Item ghost (being dragged) | `opacity-40` on original list slot                                                            |
| DragOverlay clone          | `opacity-90 shadow-panel`                                                                     |
| Skeleton pulse             | `animate-pulse bg-plugin-surface-1 border-plugin-border rounded-xl`                           |
| Gap between chain items    | `gap-3`                                                                                       |
| Drag handle gutter         | `pl-8` on `<ol>`, `-left-6` on handle button                                                  |

---

## 9. Accessibility Design

### Keyboard interaction (via `@dnd-kit`)

| Key                       | Action                   |
| ------------------------- | ------------------------ |
| `Tab`                     | Focus drag handle        |
| `Space` / `Enter`         | Pick up item             |
| `↑` / `↓`                 | Move item up/down        |
| `Escape`                  | Cancel drag              |
| `Space` / `Enter` (again) | Drop at current position |

### ARIA structure

```html
<p id="signal-chain-dnd-instructions" class="sr-only">
  To reorder: press Space or Enter to pick up, use arrow keys to move, Space or
  Enter to drop, Escape to cancel.
</p>
<ol aria-label="Signal chain">
  <li aria-roledescription="sortable" aria-label="Processor 1 of 4">
    <button
      aria-label="Drag to reorder"
      aria-describedby="signal-chain-dnd-instructions"
    >
      ...
    </button>
    <!-- ProcessorCard -->
  </li>
</ol>
```

### dnd-kit announcements

```typescript
function buildAnnouncements(order: ProcessorId[]): Announcements {
  return {
    onDragStart: ({ active }) =>
      `Picked up processor ${order.indexOf(active.id as ProcessorId) + 1} of ${order.length}.`,
    onDragOver: ({ active, over }) =>
      over
        ? `Moving to position ${order.indexOf(over.id as ProcessorId) + 1}.`
        : 'Not over a droppable area.',
    onDragEnd: ({ active, over }) =>
      over
        ? `Dropped at position ${order.indexOf(over.id as ProcessorId) + 1}.`
        : 'Dropped. Order unchanged.',
    onDragCancel: () => 'Drag cancelled. Order unchanged.'
  };
}
```

### Reduced motion

```tsx
className = 'motion-safe:transition-transform motion-safe:duration-150';
```

---

## 10. Connection/Loading States

| State                              | UI behaviour                                                                                             |
| ---------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `isLoading = true`                 | `SignalChainSkeleton` with `count={processors.length}`                                                   |
| `error != null`                    | Existing `<ErrorMessage>` component                                                                      |
| Drag active + notification arrives | Buffer notification, apply on drag end                                                                   |
| Empty order                        | `<p className="text-type-sm text-plugin-text-muted text-center py-8">No processors in signal chain.</p>` |
| Engine disconnect                  | Compose with `useConnectionStatus()`                                                                     |

---

## 11. File Placement Summary

| New file                                                         | Package                 |
| ---------------------------------------------------------------- | ----------------------- |
| `ui/packages/core/src/ipc/ProcessorOrderClient.ts`               | `@wavecraft/core`       |
| `ui/packages/core/src/types/processorOrder.ts`                   | `@wavecraft/core`       |
| `ui/packages/core/src/hooks/useProcessorOrder.ts`                | `@wavecraft/core`       |
| `ui/packages/components/src/signalChain/SignalChain.tsx`         | `@wavecraft/components` |
| `ui/packages/components/src/signalChain/SignalChainItem.tsx`     | `@wavecraft/components` |
| `ui/packages/components/src/signalChain/SignalChainSkeleton.tsx` | `@wavecraft/components` |
| `ui/packages/components/src/signalChain/DragHandleIcon.tsx`      | `@wavecraft/components` |
| `ui/packages/components/src/signalChain/index.ts`                | `@wavecraft/components` |
| `ui/packages/components/src/index.ts`                            | add re-exports          |
| `ui/packages/core/src/index.ts`                                  | add exports             |
| `ui/packages/core/src/ipc/constants.ts`                          | add IPC constants       |

---

## 12. Non-Goals (deferred)

- Signal flow arrows/lines between processor cards
- Add/remove processors from the chain (reorder only)
- Horizontal layout variant
- Per-preset component expand/collapse persistence

---

## References

- [Low-Level Design: Runtime-Reorderable SignalChain](./low-level-design-ui-signal-chain-reorder.md)
- [Coding Standards — TypeScript & React](../../architecture/coding-standards-typescript.md)
- [Coding Standards — CSS & Styling](../../architecture/coding-standards-css.md)
- [@dnd-kit documentation](https://docs.dndkit.com)
- [HTML Drag and Drop API (MDN)](https://developer.mozilla.org/en-US/docs/Web/API/HTML_Drag_and_Drop_API)
