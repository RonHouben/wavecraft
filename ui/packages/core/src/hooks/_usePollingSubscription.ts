import { useEffect } from 'react';

// private — do not export from index.ts
function usePollingSubscriptionInternal(
  subscribe: () => (() => void) | undefined,
  deps: readonly unknown[]
): void {
  useEffect(() => {
    const unsubscribe = subscribe();
    return () => {
      unsubscribe?.();
    };
    // deps are intentionally passed by caller to preserve hook-specific semantics
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);
}

// private — do not export from index.ts
export const _usePollingSubscription = usePollingSubscriptionInternal;
