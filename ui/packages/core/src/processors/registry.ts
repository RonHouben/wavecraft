const processors = new Set<string>();
const listeners = new Set<() => void>();
let snapshot: readonly string[] = [];

function rebuildSnapshot(): void {
  snapshot = [...processors].sort();
}

function notifyListeners(): void {
  for (const listener of listeners) {
    listener();
  }
}

/**
 * Register discovered processor IDs at runtime.
 *
 * Called by generated `ui/src/generated/processors.ts` during app startup.
 */
export function registerAvailableProcessors(processorIds: readonly string[]): void {
  let changed = false;

  for (const processorId of processorIds) {
    if (!processors.has(processorId)) {
      processors.add(processorId);
      changed = true;
    }
  }

  if (changed) {
    rebuildSnapshot();
    notifyListeners();
  }
}

export function hasRegisteredProcessor(processorId: string): boolean {
  return processors.has(processorId);
}

export function getRegisteredProcessors(): readonly string[] {
  return snapshot;
}

export function subscribeToRegisteredProcessors(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getRegisteredProcessorsSnapshot(): readonly string[] {
  return snapshot;
}

export function __resetRegisteredProcessorsForTests(): void {
  processors.clear();
  listeners.clear();
  snapshot = [];
}
