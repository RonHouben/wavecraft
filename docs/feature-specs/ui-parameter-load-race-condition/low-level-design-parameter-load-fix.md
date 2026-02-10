# Low-Level Design: UI Parameter Load Race Condition Fix

## Metadata

| Field | Value |
|-------|-------|
| **Feature** | Fix UI Race Condition on Parameter Load |
| **Status** | Draft |
| **Author** | Architect Agent |
| **Date** | 2026-02-10 |
| **Scope** | `@wavecraft/core` package (UI-only) |
| **User Stories** | [user-stories.md](./user-stories.md) |

---

## 1. Problem Analysis

### 1.1 Root Cause

The `useAllParameters()` hook has two effects that run on mount:

```
Effect 1: reload()                           ← fires unconditionally on mount
Effect 2: if (connected) { reload() }        ← fires only when connected changes
```

When the component mounts before the WebSocket connects:

1. **Effect 1** calls `reload()` → `ParameterClient.getAllParameters()` → `IpcBridge.invoke()` → throws `'IpcBridge: Transport not connected'`
2. Error is caught: `setError(err)`, `setIsLoading(false)`
3. **Effect 2** checks `connected === false` → no-op
4. Hook settles into: `{ params: [], isLoading: false, error: Error }`

The recovery path depends on `useConnectionStatus()`, which **polls at 1-second intervals**. When the WebSocket eventually connects and the poll fires, `connected` becomes `true`, re-running Effect 2. However, the user already sees a flash of error state for up to 1 second, and the polling approach has additional defects.

### 1.2 Identified Defects

| # | Defect | Impact | Severity |
|---|--------|--------|----------|
| **D1** | `isLoading` becomes `false` on "not connected yet" failure | UI shows error flash for ~1s before auto-retry succeeds | Medium |
| **D2** | Polling-based connection detection has up to 1s latency | Slow recovery; can miss rapid connect/disconnect transitions | Medium |
| **D3** | No cleanup of async operations on unmount | Potential `setState` on unmounted component; memory leaks | Medium |
| **D4** | No deduplication of concurrent `reload()` calls | React 18 Strict Mode double-mount triggers duplicate fetches | Low |
| **D5** | No timeout for waiting-for-connection phase | If server never starts, error appears immediately from Effect 1 instead of after a reasonable wait | Low |
| **D6** | `reload()` doesn't check connection state before calling | Manual `reload()` while disconnected immediately throws | Low |

### 1.3 Comparison: `useParameter()` vs `useAllParameters()`

`useParameter()` already has an `isMounted` cleanup flag (see `useParameter.ts` lines 24–59). `useAllParameters()` lacks this pattern entirely — a consistency gap that this fix addresses.

---

## 2. Design Decisions

### 2.1 Event-Based Connection Notification

**Decision:** Replace polling in `useConnectionStatus()` with event-based notification from the transport layer.

| Approach | Latency | CPU | Reliability | Complexity |
|----------|---------|-----|-------------|------------|
| Polling (current, 1s) | 0–1000ms | Continuous timer | Can miss transient states | Low |
| Polling (faster, 200ms) | 0–200ms | 5× more timers | Still misses sub-200ms transitions | Low |
| **Event-based (chosen)** | **~0ms** | **Zero when idle** | **Guaranteed delivery** | **Medium** |

Event-based notification is superior on every axis except implementation complexity, which is minimal given the transport already knows its own state transitions (`ws.onopen`, `ws.onclose`).

### 2.2 Hook State Machine

**Decision:** `useAllParameters()` internally tracks a state machine that distinguishes "waiting for connection" from "fetching" and "error."

The hook must **not** surface `isLoading: false` with an error when the failure is simply "transport not connected yet." That's a transient condition, not an application error. From the consumer's perspective:

- `isLoading: true` until parameters are fetched **or** a real timeout is reached
- `error` is only set for unrecoverable failures (timeout exceeded, application-level error after connection)

### 2.3 Modify `useAllParameters()` Directly

**Decision:** Modify the existing hook rather than creating a wrapper.

The current behavior is broken. A wrapper would leave the current hook broken for anyone who doesn't use the wrapper. The fix must be transparent and backward-compatible.

### 2.4 Backward-Compatible API

**Decision:** The `UseAllParametersResult` interface remains unchanged:

```ts
interface UseAllParametersResult {
  params: ParameterInfo[];
  isLoading: boolean;
  error: Error | null;
  reload: () => Promise<void>;
}
```

The behavioral semantics change (`isLoading` stays `true` longer when disconnected), but the type contract is identical. This is a bug fix, not a feature addition.

### 2.5 Optional `onConnectionChange` in Transport Interface

**Decision:** Add `onConnectionChange` as an **optional** method on the `Transport` interface. `IpcBridge` falls back to 1-second polling if the transport doesn't implement it.

This avoids a breaking change for any consumer who has implemented a custom `Transport` (the interface is exported as "advanced use").

---

## 3. Architecture

### 3.1 Component Interaction Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         React Component Tree                            │
│                                                                         │
│  ┌──────────────────────┐      ┌────────────────────────────────────┐   │
│  │ useConnectionStatus  │      │ useAllParameters                   │   │
│  │                      │      │                                    │   │
│  │ Subscribes to        │      │ 1. Reads connection status         │   │
│  │ IpcBridge            │      │ 2. Waits if disconnected           │   │
│  │ .onConnectionChange  │      │ 3. Fetches when connected          │   │
│  │                      │      │ 4. Refetches on reconnect          │   │
│  └──────────┬───────────┘      └──────────────┬─────────────────────┘   │
│             │                                 │                         │
│             │ connection events               │ getAllParameters()       │
│             ▼                                 ▼                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     IpcBridge (singleton)                        │   │
│  │                                                                  │   │
│  │  • invoke(method, params) → Promise<T>                           │   │
│  │  • isConnected() → boolean                                       │   │
│  │  • onConnectionChange(cb) → unsubscribe              ← NEW      │   │
│  │                                                                  │   │
│  └─────────────────────────────┬────────────────────────────────────┘   │
│                                │                                        │
│                                │ delegates to                           │
│                                ▼                                        │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    Transport (interface)                          │   │
│  │                                                                  │   │
│  │  + send(request) → Promise<string>                               │   │
│  │  + isConnected() → boolean                                       │   │
│  │  + onConnectionChange?(cb) → unsubscribe             ← NEW opt  │   │
│  │  + onNotification(cb) → unsubscribe                              │   │
│  │  + dispose()                                                     │   │
│  │                                                                  │   │
│  ├───────────────────────┬──────────────────────────────────────────┤   │
│  │  NativeTransport      │  WebSocketTransport                      │   │
│  │  (always connected)   │  (async connect + reconnect)             │   │
│  │                       │                                          │   │
│  │  onConnectionChange:  │  onConnectionChange:                     │   │
│  │   → fires true once   │   → fires on ws.onopen (true)           │   │
│  │     on subscribe      │   → fires on ws.onclose (false)         │   │
│  │                       │   → fires maxAttempts (false, final)     │   │
│  └───────────────────────┴──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Hook State Machine

```
                               ┌─────────────────────┐
               mount +         │                     │
             disconnected      │  WAITING_FOR_       │
            ──────────────────►│  CONNECTION          │
                               │                     │
                               │  isLoading: true    │
                               │  error: null        │
                               │  params: []         │
                               └──────┬──────────────┘
                                      │           │
                        connected     │           │ timeout (15s)
                        event         │           │
                        ▼             │           ▼
       mount +     ┌──────────────┐   │     ┌──────────┐
     connected     │              │   │     │          │
     ─────────────►│  FETCHING    │   │     │  ERROR   │
                   │              │   │     │          │
                   │  isLoading:  │   │     │ isLoading│
                   │    true      │   │     │  : false │
                   │  error: null │   │     │ error:   │
                   │  params: prev│   │     │  set     │
                   └──┬─────┬────┘   │     └────┬─────┘
                      │     │        │          │
            success   │     │ fail + │  reload()│ + connected
                      │     │ connected         │
                      │     │        │          │
                      │     └────────┘          │
                      │   (retry w/ backoff,    │
                      │    max 3 attempts)      │
                      ▼                         │
              ┌───────────────┐                 │
              │               │                 │
              │   LOADED      │◄────────────────┘
              │               │   (refetch succeeds)
              │  isLoading:   │
              │    false      │
              │  error: null  │
              │  params: [..] │
              └───────┬───────┘
                      │
                      │ disconnect + reconnect event
                      │
                      └──────────► FETCHING (auto-refetch)
```

### 3.3 State Transitions Table

| Current State | Event | Next State | Action |
|---------------|-------|------------|--------|
| — | mount + disconnected | WAITING_FOR_CONNECTION | Subscribe to connection events; start 15s timeout |
| — | mount + connected | FETCHING | Call `getAllParameters()` |
| WAITING_FOR_CONNECTION | connected event | FETCHING | Call `getAllParameters()` |
| WAITING_FOR_CONNECTION | timeout (15s) | ERROR | Set error: "Dev server not reachable…" |
| WAITING_FOR_CONNECTION | unmount | — | Cancel timeout, unsubscribe events |
| FETCHING | success | LOADED | Set params, `isLoading: false` |
| FETCHING | fail + still connected | FETCHING | Retry with backoff (max 3 retries) |
| FETCHING | fail + max retries | ERROR | Set error with failure detail |
| FETCHING | transport disconnect mid-fetch | WAITING_FOR_CONNECTION | Stay `isLoading: true`, wait for reconnect |
| FETCHING | unmount | — | Set cancelled flag; ignore pending response |
| LOADED | reconnect event (disconnect→connect) | FETCHING | Refetch parameters |
| LOADED | unmount | — | Unsubscribe events |
| ERROR | `reload()` + connected | FETCHING | Retry |
| ERROR | `reload()` + disconnected | WAITING_FOR_CONNECTION | Wait for connection |
| ERROR | unmount | — | Unsubscribe events |

---

## 4. Interface Changes

### 4.1 Transport Interface (Optional Addition)

File: `ui/packages/core/src/transports/Transport.ts`

```ts
export interface Transport {
  send(request: string): Promise<string>;
  onNotification(callback: NotificationCallback): () => void;
  isConnected(): boolean;
  dispose(): void;

  /**
   * Subscribe to connection state changes.
   *
   * Callback fires immediately with current state on subscribe,
   * then on every subsequent state transition.
   *
   * Optional: if not implemented, IpcBridge falls back to polling.
   *
   * @param callback - Receives true when connected, false when disconnected
   * @returns Cleanup function to remove the listener
   */
  onConnectionChange?(callback: (connected: boolean) => void): () => void;
}
```

**Fire-on-subscribe rationale:** Eliminates race conditions where the transport connects between the `isConnected()` check and the event subscription. The subscriber always gets the authoritative initial state.

### 4.2 WebSocketTransport Changes

File: `ui/packages/core/src/transports/WebSocketTransport.ts`

Add a callback set and emit on state transitions:

```ts
export class WebSocketTransport implements Transport {
  // NEW: connection change listeners
  private readonly connectionChangeCallbacks = new Set<(connected: boolean) => void>();

  // NEW: event subscription with fire-on-subscribe
  onConnectionChange(callback: (connected: boolean) => void): () => void {
    this.connectionChangeCallbacks.add(callback);
    // Fire immediately with current state
    callback(this.isConnected());
    return () => {
      this.connectionChangeCallbacks.delete(callback);
    };
  }

  // MODIFY: connect() — add emit in onopen and onclose handlers
  private connect(): void {
    // ... existing code ...
    this.ws.onopen = (): void => {
      this.isConnecting = false;
      this.reconnectAttempts = 0;
      logger.info('WebSocketTransport connected', { url: this.url });
      this.emitConnectionChange(true);   // ← ADD
    };

    this.ws.onclose = (): void => {
      this.isConnecting = false;
      this.ws = null;
      this.emitConnectionChange(false);  // ← ADD
      if (!this.isDisposed && !this.maxAttemptsReached) {
        this.scheduleReconnect();
      }
    };
    // ... rest unchanged ...
  }

  // NEW: emit helper
  private emitConnectionChange(connected: boolean): void {
    for (const callback of this.connectionChangeCallbacks) {
      try {
        callback(connected);
      } catch (error) {
        logger.error('WebSocketTransport connection change callback error', { error });
      }
    }
  }

  // MODIFY: dispose() — clear connection change callbacks
  dispose(): void {
    // ... existing cleanup ...
    this.connectionChangeCallbacks.clear();  // ← ADD
  }
}
```

### 4.3 NativeTransport Changes

File: `ui/packages/core/src/transports/NativeTransport.ts`

Trivial implementation — native is always connected:

```ts
export class NativeTransport implements Transport {
  onConnectionChange(callback: (connected: boolean) => void): () => void {
    // Always connected. Fire true immediately, never again.
    callback(true);
    return () => { /* no-op: native never transitions */ };
  }
}
```

### 4.4 IpcBridge Changes

File: `ui/packages/core/src/ipc/IpcBridge.ts`

Add `onConnectionChange()` with polling fallback for transports that don't implement it:

```ts
export class IpcBridge {
  /**
   * Subscribe to transport connection state changes.
   *
   * Uses transport's event-based notification if available,
   * falls back to 1-second polling otherwise.
   *
   * @param callback - Receives true (connected) or false (disconnected)
   * @returns Cleanup function
   */
  public onConnectionChange(callback: (connected: boolean) => void): () => void {
    this.initialize();

    if (!this.transport) {
      callback(false);
      return () => {};
    }

    // Prefer event-based if transport supports it
    if (this.transport.onConnectionChange) {
      return this.transport.onConnectionChange(callback);
    }

    // Fallback: poll every second (backward compat with custom transports)
    let lastState = this.transport.isConnected();
    callback(lastState); // Fire-on-subscribe

    const intervalId = setInterval(() => {
      const currentState = this.transport?.isConnected() ?? false;
      if (currentState !== lastState) {
        lastState = currentState;
        callback(currentState);
      }
    }, 1000);

    return () => clearInterval(intervalId);
  }
}
```

### 4.5 useConnectionStatus Refactor

File: `ui/packages/core/src/hooks/useConnectionStatus.ts`

Replace polling with event subscription:

```ts
export function useConnectionStatus(): ConnectionStatus {
  const [status, setStatus] = useState<ConnectionStatus>({
    connected: false,
    transport: 'none',
  });

  useEffect(() => {
    const bridge = IpcBridge.getInstance();
    const isNative = isWebViewEnvironment();

    const unsubscribe = bridge.onConnectionChange((connected) => {
      const transport: TransportType = isNative
        ? 'native'
        : connected
          ? 'websocket'
          : 'none';

      setStatus((prev) => {
        if (prev.connected !== connected || prev.transport !== transport) {
          return { connected, transport };
        }
        return prev;
      });
    });

    return unsubscribe;
  }, []);

  return status;
}
```

**Migration impact:** Zero. Same return type, same semantics, same hook name. Consumers see no difference except faster updates.

### 4.6 useAllParameters Rewrite

File: `ui/packages/core/src/hooks/useAllParameters.ts`

```ts
/**
 * useAllParameters - Hook for loading all parameters
 *
 * Connection-aware: waits for transport connection before fetching.
 * Auto-refetches on reconnection. Deduplicates concurrent requests.
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { ParameterClient } from '../ipc/ParameterClient';
import { IpcBridge } from '../ipc/IpcBridge';
import { useConnectionStatus } from './useConnectionStatus';
import { logger } from '../logger/Logger';
import type { ParameterInfo } from '../types/parameters';

/** Maximum time (ms) to wait for connection before giving up */
const CONNECTION_TIMEOUT_MS = 15_000;

/** Maximum fetch retry attempts after connection is established */
const MAX_FETCH_RETRIES = 3;

/** Base delay (ms) for fetch retry backoff */
const FETCH_RETRY_BASE_MS = 500;

export interface UseAllParametersResult {
  params: ParameterInfo[];
  isLoading: boolean;
  error: Error | null;
  reload: () => Promise<void>;
}

export function useAllParameters(): UseAllParametersResult {
  const [params, setParams] = useState<ParameterInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const { connected } = useConnectionStatus();

  // Cleanup guard: prevents setState on unmounted component
  const mountedRef = useRef(true);
  // Deduplication: prevents concurrent fetches
  const fetchingRef = useRef(false);
  // Track previous connected state to detect transitions
  const prevConnectedRef = useRef<boolean | null>(null);

  // Mount/unmount lifecycle
  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  /**
   * Fetch all parameters with retry logic.
   * Only retries on application-level failures while transport is connected.
   * Bails out silently if transport disconnects mid-fetch (reconnect handler
   * will re-trigger).
   */
  const fetchParameters = useCallback(async (): Promise<void> => {
    if (fetchingRef.current) {
      logger.debug('useAllParameters: fetch already in-flight, skipping');
      return;
    }

    fetchingRef.current = true;
    const client = ParameterClient.getInstance();

    for (let attempt = 0; attempt <= MAX_FETCH_RETRIES; attempt++) {
      if (!mountedRef.current) {
        fetchingRef.current = false;
        return;
      }

      try {
        if (attempt > 0) {
          logger.debug('useAllParameters: retry attempt', {
            attempt,
            maxRetries: MAX_FETCH_RETRIES,
          });
        }

        const allParams = await client.getAllParameters();

        if (mountedRef.current) {
          setParams(allParams);
          setError(null);
          setIsLoading(false);
        }

        fetchingRef.current = false;
        return; // Success — exit retry loop
      } catch (err) {
        // If transport disconnected mid-fetch, don't retry here.
        // The connection change effect will re-trigger when reconnected.
        const bridge = IpcBridge.getInstance();
        if (!bridge.isConnected()) {
          logger.debug('useAllParameters: transport disconnected during fetch, awaiting reconnect');
          fetchingRef.current = false;
          return; // Stay in loading state — no error surfaced
        }

        // Last retry exhausted → surface error
        if (attempt === MAX_FETCH_RETRIES) {
          if (mountedRef.current) {
            const message = err instanceof Error ? err.message : String(err);
            setError(
              new Error(
                `Parameter fetch failed after ${MAX_FETCH_RETRIES + 1} attempts: ${message}`
              )
            );
            setIsLoading(false);
          }
          fetchingRef.current = false;
          return;
        }

        // Exponential backoff before next retry
        const delay = FETCH_RETRY_BASE_MS * Math.pow(2, attempt);
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }

    fetchingRef.current = false;
  }, []);

  /**
   * Public reload function.
   * Connection-aware: if connected → fetch immediately.
   * If disconnected → resets to loading state (connection effect handles retry).
   */
  const reload = useCallback(async (): Promise<void> => {
    if (!mountedRef.current) return;

    setIsLoading(true);
    setError(null);

    const bridge = IpcBridge.getInstance();
    if (bridge.isConnected()) {
      await fetchParameters();
    }
    // If not connected: isLoading stays true.
    // The connection change effect will trigger fetchParameters on connect.
  }, [fetchParameters]);

  // ─── Effect: React to connection state transitions ──────────────────
  useEffect(() => {
    const wasConnected = prevConnectedRef.current;
    prevConnectedRef.current = connected;

    if (connected && wasConnected !== true) {
      // Transition: disconnected/initial → connected
      logger.debug('useAllParameters: connection established, fetching parameters');
      fetchParameters();
    }

    if (!connected && wasConnected === true) {
      // Transition: connected → disconnected
      // Keep stale params (better than empty). Show loading for incoming refetch.
      logger.debug('useAllParameters: connection lost, awaiting reconnect');
      setIsLoading(true);
    }
  }, [connected, fetchParameters]);

  // ─── Effect: Connection timeout ─────────────────────────────────────
  useEffect(() => {
    if (connected) return; // Already connected — no timeout needed

    const timeoutId = setTimeout(() => {
      if (!mountedRef.current) return;

      const bridge = IpcBridge.getInstance();
      if (!bridge.isConnected()) {
        setError(
          new Error(
            'Could not connect to dev server within 15 seconds. ' +
              'Is `wavecraft start` running?'
          )
        );
        setIsLoading(false);
      }
    }, CONNECTION_TIMEOUT_MS);

    return () => clearTimeout(timeoutId);
  }, [connected]);

  // ─── Effect: Subscribe to parameter change notifications ────────────
  useEffect(() => {
    const client = ParameterClient.getInstance();
    const handleParamChange = (changedId: string, value: number): void => {
      setParams((prev) =>
        prev.map((p) => (p.id === changedId ? { ...p, value } : p))
      );
    };
    return client.onParameterChanged(handleParamChange);
  }, []);

  return { params, isLoading, error, reload };
}
```

---

## 5. Behavioral Walkthroughs

### 5.1 Happy Path (WebSocket already connected at mount)

```
T0: Component mounts
    useConnectionStatus() → connected: true (from onConnectionChange immediate fire)
    Connection effect: wasConnected=null → connected=true → fetchParameters()
    fetchParameters() → getAllParameters() → success
    Result: { params: [...], isLoading: false, error: null }
```

Backward compatible. Same behavior as current code but without the wasted initial failure.

### 5.2 Slow Connection (mount before WebSocket connects)

```
T0: Component mounts
    useConnectionStatus() → connected: false
    Connection effect: wasConnected=null, connected=false → no action
    Timeout effect: starts 15s timer
    Result: { params: [], isLoading: true, error: null }

T0+2s: WebSocket connects
    onConnectionChange fires → useConnectionStatus → connected: true
    Connection effect: wasConnected=false → connected=true → fetchParameters()
    Timeout effect: cleanup (clears timer)
    fetchParameters() → success
    Result: { params: [...], isLoading: false, error: null }
```

**UX improvement:** No error flash. User sees loading indicator for ~2s, then parameters appear.

### 5.3 Server Never Starts

```
T0: Component mounts, connected: false
    Result: { params: [], isLoading: true, error: null }

T0+15s: Timeout fires
    Result: { params: [], isLoading: false,
              error: "Could not connect to dev server within 15 seconds.
                      Is `wavecraft start` running?" }
```

### 5.4 Reconnection After Disconnect

```
T0: Parameters loaded, connected: true
    Result: { params: [...], isLoading: false, error: null }

T1: WebSocket disconnects
    Connection effect: wasConnected=true → connected=false
    → setIsLoading(true); keep params
    Result: { params: [stale], isLoading: true, error: null }

T2: WebSocket reconnects
    Connection effect: wasConnected=false → connected=true → fetchParameters()
    Result: { params: [fresh], isLoading: false, error: null }
```

### 5.5 Native Plugin Mode (NativeTransport)

```
T0: Component mounts
    NativeTransport.onConnectionChange fires immediately: connected=true
    useConnectionStatus(): connected: true
    Connection effect: wasConnected=null → true → fetchParameters() → success
    Result: { params: [...], isLoading: false, error: null }
```

**No regression.** NativeTransport fires `true` synchronously on subscribe. No timeout, no waiting.

### 5.6 React 18 Strict Mode (Double Mount)

```
Mount #1:
    mountedRef.current = true
    Connection effect subscribes. If connected → fetchParameters() (fetchingRef = true)

Unmount #1 (strict mode):
    mountedRef.current = false
    Effect cleanups run

Mount #2:
    mountedRef.current = true (new instance)
    Connection effect subscribes. If connected → fetchParameters()
      If previous fetch still in-flight (fetchingRef = true) → skipped (dedup)
      If previous fetch completed → new fetch starts
```

`fetchingRef` prevents duplicate in-flight requests. `mountedRef` prevents `setState` on unmounted components.

**Note on refs in Strict Mode:** `useRef` values persist across strict-mode remounts in the same component instance. The mount/unmount effect resets `mountedRef` correctly for each lifecycle.

---

## 6. Files Changed Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `transports/Transport.ts` | **Interface addition** | Add optional `onConnectionChange` method |
| `transports/WebSocketTransport.ts` | **Minor addition** | Connection change callbacks + emit on open/close |
| `transports/NativeTransport.ts` | **Minor addition** | Trivial `onConnectionChange` (fire true, no-op) |
| `ipc/IpcBridge.ts` | **Minor addition** | `onConnectionChange()` forwarding with polling fallback |
| `hooks/useConnectionStatus.ts` | **Refactor** | Replace 1s polling with event subscription |
| `hooks/useAllParameters.ts` | **Rewrite** | State machine with connection-awareness, dedup, cleanup, timeout |
| `hooks/useParameter.ts` | **No change** | Already has `isMounted` guard; unaffected |

### New Public API

| Export | Type | Notes |
|--------|------|-------|
| `IpcBridge.onConnectionChange()` | Method | New on singleton; used by hooks internally |
| `Transport.onConnectionChange?()` | Interface method (optional) | For custom transport implementors |

No changes to `UseAllParametersResult` or `ConnectionStatus` types.

---

## 7. Test Scenarios

### 7.1 Unit Tests (`useAllParameters`) — Vitest + React Testing Library

| # | Scenario | AC | Setup | Expected |
|---|----------|-----|-------|----------|
| T1 | Mount when already connected | AC1.1 | Mock transport, `connected=true` | `isLoading` → `false`, params populated |
| T2 | Mount disconnected → connect after 500ms | AC1.1, AC1.2 | Fire connected after 500ms | `isLoading: true` throughout wait, params populated after connect |
| T3 | Mount disconnected → never connects | AC1.4 | Never fires connected | `isLoading: true` for 15s → `false`, error contains "wavecraft start" |
| T4 | Reconnection auto-refetch | AC1.3 | Connect → load → disconnect → reconnect | Params refetched automatically |
| T5 | Duplicate fetch prevention | AC2.3 | Fire connected twice rapidly (<50ms) | Only one `getAllParameters()` call made |
| T6 | Cleanup on unmount during WAITING | AC4.1 | Mount → unmount before connect | No setState warnings, timeout cleared |
| T7 | Cleanup on unmount during FETCH | AC4.1 | Mount → connect → unmount during fetch | No setState after unmount |
| T8 | React 18 Strict Mode double-mount | AC4.2 | Wrap in `<StrictMode>` | Single fetch, no duplicate requests |
| T9 | Fetch fails (connected, server error) | AC4.4 | Transport connected, `getAllParameters()` rejects | Retries 3x with backoff, then error |
| T10 | Transport disconnects mid-fetch | AC4.4 | Disconnect transport during `getAllParameters()` | Stays loading, retries on reconnect (not surfaced as error) |
| T11 | Native transport (always connected) | AC3.1, AC3.2 | NativeTransport mock | Immediate fetch, no timeout delay |
| T12 | `reload()` while disconnected | AC2.4 | Call `reload()` while disconnected | Sets `isLoading: true`, waits for connection |
| T13 | Error message content (timeout) | AC1.5 | Timeout fires | Error text includes "wavecraft start" |
| T14 | Error message content (fetch failure) | AC4.4 | Fetch fails all retries | Error text includes attempt count |
| T15 | Parameter change notification | — | Param notification arrives post-load | Params updated in place |
| T16 | `reload()` clears error state | — | Error state → `reload()` + connected | `isLoading: true`, `error: null`, then loads |

### 7.2 Unit Tests (`useConnectionStatus`)

| # | Scenario | Expected |
|---|----------|----------|
| T17 | Event-based status update (connected) | `{ connected: true, transport: 'websocket' }` |
| T18 | Disconnect event | Transitions to `{ connected: false, transport: 'none' }` |
| T19 | Native environment | `{ connected: true, transport: 'native' }` always |
| T20 | Cleanup on unmount | Unsubscribe function called |

### 7.3 Unit Tests (`WebSocketTransport.onConnectionChange`)

| # | Scenario | Expected |
|---|----------|----------|
| T21 | Fire-on-subscribe (disconnected) | Callback receives `false` synchronously |
| T22 | Fire-on-subscribe (connected) | Callback receives `true` synchronously |
| T23 | Transition connected → disconnected | Callback receives `false` |
| T24 | Unsubscribe prevents further calls | No callback after unsubscribe |
| T25 | Multiple subscribers | All receive events independently |
| T26 | dispose() clears callbacks | No callbacks after dispose |

### 7.4 Integration Tests

| # | Scenario | Setup |
|---|----------|-------|
| IT1 | Delayed WebSocket connection | Start WS server after 2s; verify hook loads params |
| IT2 | Disconnect/reconnect cycle | Drop and restore WS; verify auto-refetch |

### 7.5 Manual Test Plan

| # | Steps | Expected |
|---|-------|----------|
| MT1 | Start UI (`npm run dev`) WITHOUT `wavecraft start` → wait 15s | Loading indicator → timeout error with guidance |
| MT2 | Start UI → start `wavecraft start` within 5s | Loading indicator → parameters appear |
| MT3 | Running normally → kill dev server → restart | Brief loading → parameters refresh |
| MT4 | Load plugin in Ableton (native mode) | Parameters load instantly, no regressions |

---

## 8. Risk Assessment

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|------------|--------|------------|
| R1 | `onConnectionChange` callback fires during transport constructor (before subscription) | Low | Missed initial event | Fire-on-subscribe pattern guarantees initial state delivery |
| R2 | Rapid connect/disconnect causes excessive refetches | Low | Redundant network calls | `fetchingRef` deduplication; disconnect during fetch bails without error |
| R3 | `useRef` state lost during React fast refresh (HMR) | Low | Stale ref values | Mount/unmount effect resets `mountedRef`; fast refresh triggers full remount |
| R4 | Timer/event leak on HMR | Low | Memory leak in dev | All effect cleanup functions handle subscriptions and timers |
| R5 | `Transport` interface addition breaks custom implementations | Medium | Build error for advanced users | Made `onConnectionChange` optional; `IpcBridge` provides polling fallback |
| R6 | `fetchingRef` prevents legitimate refetch after error | Low | Stuck in error state | `fetchingRef` reset in all control flow paths (success, failure, disconnect, unmount) |
| R7 | Timeout fires after component receives params (race) | Low | False error | Timeout effect cleanup runs when `connected` changes; timeout callback checks `isLoading` |
| R8 | Double `setIsLoading(true)` on reconnect (from timeout effect + connection effect) | Low | No visible impact | Both set the same value; React batches state updates |

---

## 9. Implementation Order

Dependencies dictate this order:

```
Phase 1: Transport layer event system
  1. Transport.ts          — Add optional onConnectionChange to interface
  2a. WebSocketTransport.ts  ─┐
  2b. NativeTransport.ts      ├── Implement onConnectionChange (parallel)
  3. IpcBridge.ts             ┘  Forward with polling fallback

Phase 2: Hook refactor
  4. useConnectionStatus.ts  — Replace polling with event subscription
  5. useAllParameters.ts     — Rewrite with state machine

Phase 3: Testing
  6. Transport unit tests    — T21–T26
  7. Hook unit tests         — T1–T20
  8. Integration tests       — IT1–IT2
  9. Manual testing          — MT1–MT4
```

Steps 2a/2b are parallel. Phase 2 depends on Phase 1. Phase 3 can partially overlap with Phase 2 (test files can be structured while implementation is in progress).

---

## 10. Out of Scope

Per user stories:

- ❌ Rust engine changes
- ❌ Parameter change notification optimizations
- ❌ UI components for connection status display
- ❌ WebSocket transport improvements beyond event emission
- ❌ Parameter caching or persistence
- ❌ Changes to `useParameter()` hook (already has `isMounted` guard)

---

## Related Documents

- [User Stories](./user-stories.md) — Requirements and acceptance criteria
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards — TypeScript](../../architecture/coding-standards-typescript.md) — React hook conventions
- [Coding Standards — Testing](../../architecture/coding-standards-testing.md) — Test patterns
