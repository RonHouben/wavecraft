# Implementation Plan: UI Parameter Load Race Condition Fix

## 1. Overview

This plan details the implementation for fixing a race condition in the `@wavecraft/core` package where the `useAllParameters` hook can fail if it's used before the WebSocket connection is established in the browser development environment.

The fix involves creating a more robust, event-driven connection notification system in the transport layer and refactoring the `useAllParameters` hook to be connection-aware, with a proper state machine to handle transient connection states.

**Estimated Effort:**
*   **Development:** 4-6 hours
*   **Testing:** 2-3 hours

## 2. Prerequisites

*   A local Wavecraft development environment must be set up as per the project's contribution guidelines.
*   Familiarity with the concepts in the [Low-Level Design](./low-level-design-parameter-load-fix.md).
*   The Coder agent should have access to the `@wavecraft/core` package source code at `ui/packages/core/`.

## 3. Implementation Phases

The implementation is divided into three main phases to ensure a structured and testable rollout.

### Phase 1: Transport Layer Event System

This phase focuses on adding an event-based notification mechanism for connection status changes, moving away from the current polling system. This is the foundation for the hook refactor.

### Phase 2: Hook Refactor

With the event system in place, this phase involves rewriting the `useAllParameters` hook to use the new system. It will implement a state machine to gracefully handle the period before a connection is established and manage automatic data fetching on connection events. The `useConnectionStatus` hook will also be updated.

### Phase 3: Testing

This phase involves writing comprehensive unit tests for the new logic and performing manual integration testing to ensure the fix is effective and does not introduce regressions.

## 4. Step-by-Step Tasks

### Phase 1: Transport Layer Event System (Est: 1.5 hours)

1.  **Task 1.1: Update `Transport` Interface**
    *   **File:** `ui/packages/core/src/transports/Transport.ts`
    *   **Action:** Add the optional `onConnectionChange?(callback: (connected: boolean) => void): () => void;` method to the `Transport` interface.
    *   **Why:** To define a standard way for transports to provide connection status updates. Making it optional ensures backward compatibility.
    *   **Dependencies:** None.

2.  **Task 1.2: Implement `onConnectionChange` in `WebSocketTransport`**
    *   **File:** `ui/packages/core/src/transports/WebSocketTransport.ts`
    *   **Action:**
        *   Add a `connectionChangeCallbacks` Set.
        *   Implement the `onConnectionChange` method to add/remove callbacks and fire immediately with the current state.
        *   Call `emitConnectionChange(true)` in `onopen` and `emitConnectionChange(false)` in `onclose`.
        *   Clear callbacks in `dispose()`.
    *   **Why:** To provide real-time connection status events for the WebSocket transport.
    *   **Dependencies:** Task 1.1.

3.  **Task 1.3: Implement `onConnectionChange` in `NativeTransport`**
    *   **File:** `ui/packages/core/src/transports/NativeTransport.ts`
    *   **Action:** Implement the `onConnectionChange` method. It should fire the callback with `true` immediately upon subscription and do nothing else.
    *   **Why:** To ensure the native transport conforms to the updated interface. Behavior is simple as it's always connected.
    *   **Dependencies:** Task 1.1.

4.  **Task 1.4: Update `IpcBridge` to use `onConnectionChange`**
    *   **File:** `ui/packages/core/src/ipc/IpcBridge.ts`
    *   **Action:** Implement a public `onConnectionChange` method that prefers the transport's event-based mechanism but falls back to 1-second polling if the transport does not implement it.
    *   **Why:** To provide a single, reliable source for connection status to the rest of the application, abstracting away the transport details.
    *   **Dependencies:** Tasks 1.2, 1.3.

### Phase 2: Hook Refactor (Est: 2.5 hours)

1.  **Task 2.1: Refactor `useConnectionStatus` Hook**
    *   **File:** `ui/packages/core/src/hooks/useConnectionStatus.ts`
    *   **Action:** Replace the `setInterval` polling logic with a `useEffect` that subscribes to `IpcBridge.getInstance().onConnectionChange`.
    *   **Why:** To make the hook more efficient and responsive by using the new event-based system.
    *   **Dependencies:** Task 1.4.

2.  **Task 2.2: Rewrite `useAllParameters` Hook**
    *   **File:** `ui/packages/core/src/hooks/useAllParameters.ts`
    *   **Action:** Rewrite the hook to implement the state machine described in the low-level design.
        *   Use `useConnectionStatus` to react to connection changes.
        *   Introduce `mountedRef` and `fetchingRef` to handle unmounts and deduplicate fetches.
        *   Implement the `WAITING_FOR_CONNECTION` state with a `CONNECTION_TIMEOUT_MS` of 15 seconds.
        *   Implement retry logic with exponential backoff for application-level fetch failures.
        *   Ensure `isLoading` remains `true` while waiting for a connection.
        *   Refactor the `reload` function to be connection-aware.
    *   **Why:** This is the core of the fix, making the hook robust against race conditions and connection interruptions.
    *   **Dependencies:** Task 2.1.

### Phase 3: Testing (Est: 2 hours)

1.  **Task 3.1: Write Unit Tests for Transports**
    *   **File:** A new test file, e.g., `ui/packages/core/src/transports/Transport.test.ts`.
    *   **Action:** Add unit tests for `WebSocketTransport` and `NativeTransport`'s `onConnectionChange` implementation, covering fire-on-subscribe, state transitions, and unsubscribing.
    *   **Why:** To verify the foundation of the eventing system.
    *   **Dependencies:** Tasks 1.2, 1.3.

2.  **Task 3.2: Write Unit Tests for Hooks**
    *   **File:** `ui/packages/core/src/hooks/useAllParameters.test.ts` and `ui/packages/core/src/hooks/useConnectionStatus.test.ts`.
    *   **Action:** Add comprehensive unit tests covering the scenarios outlined in the LLD (T1-T20), including slow connections, timeouts, reconnections, and React 18 Strict Mode.
    *   **Why:** To ensure the new hook logic is correct and handles all edge cases.
    *   **Dependencies:** Tasks 2.1, 2.2.

3.  **Task 3.3: Perform Manual Testing**
    *   **Action:** Follow the manual test plan (MT1-MT4) from the LLD.
        1.  Test timeout behavior by starting the UI without the dev server.
        2.  Test slow connection behavior by starting the dev server after the UI.
        3.  Test reconnection by killing and restarting the dev server.
        4.  Test for regressions in a native plugin host (e.g., Ableton).
    *   **Why:** To validate the real-world user experience in both development and production environments.
    *   **Dependencies:** All previous tasks.

## 5. Testing Strategy

*   **Unit Testing:** Use Vitest and React Testing Library to mock transports and test the hooks' state machine logic in isolation. All 26 test scenarios from the LLD must be implemented.
*   **Integration Testing:** The manual tests serve as integration tests, verifying the interaction between the React UI, the WebSocket transport, and the dev server.
*   **Regression Testing:** Manual test MT4 is critical to ensure that the changes, which primarily target the dev experience, do not negatively impact the production native plugin behavior.

## 6. Rollback Plan

If significant issues are discovered after merging, the changes can be reverted using `git revert`. Since this is a bug fix with no breaking API changes, reverting will restore the previous (buggy) behavior without affecting consumers of the library. The feature should be developed on a dedicated branch to allow for easy rollback before merging.

## 7. Success Criteria

The implementation will be considered "done" when all of the following are met:

*   [ ] All tasks in the implementation plan are completed.
*   [ ] All 26 unit test scenarios from the LLD are implemented and passing.
*   [ ] All 4 manual test scenarios pass, confirming the fix and the absence of regressions.
*   [ ] The `useAllParameters` hook no longer enters a permanent error state when mounted before the WebSocket connects.
*   [ ] The UI shows a loading state until parameters are fetched or a connection timeout occurs.
*   [ ] The hook automatically recovers and fetches data upon connection or reconnection.
*   [ ] No performance or behavioral regressions are observed in the native plugin environment.
*   [ ] The code has been reviewed and approved.
