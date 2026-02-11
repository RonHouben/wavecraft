# User Stories: Fix UI Race Condition on Parameter Load

## Overview

When using browser dev mode (`wavecraft start`), the `useAllParameters()` hook may fire before the WebSocket connection to the dev server is established, resulting in a failed parameter fetch that never recovers. This creates a poor developer experience where the UI appears broken until a manual refresh.

**Problem:** The React component lifecycle doesn't synchronize with the asynchronous WebSocket connection lifecycle, leading to silent failures during the initial render.

**Impact:** Plugin developers testing UI changes in browser dev mode see empty parameter lists or stale data, requiring manual page refreshes to recover.

**Scope:** UI-only change in `@wavecraft/core` package. No Rust engine changes required.

---

## User Story 1: Reliable Parameter Loading in Browser Dev Mode

**As a** plugin developer using `wavecraft start` in browser dev mode  
**I want** parameters to load automatically even if I mount components before the WebSocket connects  
**So that** I don't have to manually refresh the page or time my component mounts

### Acceptance Criteria

- [ ] **AC1.1:** When `useAllParameters()` is called before WebSocket connection is established, it waits and retries automatically when connection becomes ready
- [ ] **AC1.2:** The hook's `isLoading` state remains `true` until either parameters are successfully fetched or an unrecoverable error occurs
- [ ] **AC1.3:** When WebSocket reconnects after a disconnect, parameters are automatically reloaded without manual intervention
- [ ] **AC1.4:** The retry mechanism has a reasonable timeout (e.g., 10-15 seconds) before giving up with a clear error message
- [ ] **AC1.5:** Errors are surfaced through the hook's `error` state with actionable messages (e.g., "Dev server not reachable at ws://127.0.0.1:9000")

### Test Scenarios

1. **Happy path:** Mount component → wait for connection → parameters load
2. **Slow connection:** Mount component → connection takes 3 seconds → parameters load when ready
3. **No server:** Mount component → server never starts → timeout error after 15 seconds
4. **Reconnection:** Load parameters → disconnect → reconnect → parameters reload automatically

---

## User Story 2: Connection-Aware Hook Behavior

**As a** React developer using Wavecraft hooks  
**I want** `useAllParameters()` to be aware of the underlying transport connection state  
**So that** the hook behaves predictably and I can build reliable UIs

### Acceptance Criteria

- [ ] **AC2.1:** Hook internally monitors connection status via `useConnectionStatus()` or equivalent mechanism
- [ ] **AC2.2:** When connection transitions from disconnected → connected, parameters are fetched automatically
- [ ] **AC2.3:** Multiple rapid connection state changes don't trigger duplicate fetches (debounced/deduplicated)
- [ ] **AC2.4:** The `reload()` function exposed by the hook respects connection state (no-op or queued if disconnected)
- [ ] **AC2.5:** Hook cleanup properly handles in-flight requests when component unmounts during loading

### Implementation Notes

- Consider using existing `useConnectionStatus()` hook to monitor transport state
- May need to add event-based notification when WebSocket connects (instead of polling)
- Ensure proper cleanup of event listeners and pending requests

---

## User Story 3: Backward Compatibility with Native Plugin Mode

**As a** plugin developer or end user  
**I want** the fix to work seamlessly in both browser dev mode (WebSocket) and native plugin mode (postMessage)  
**So that** the behavior is consistent across development and production environments

### Acceptance Criteria

- [ ] **AC3.1:** NativeTransport (WKWebView postMessage) is always immediately "connected" → parameters load on first call as before
- [ ] **AC3.2:** No performance regression in native plugin mode (no unnecessary polling or delays)
- [ ] **AC3.3:** The hook's API surface remains unchanged (no breaking changes to `UseAllParametersResult` interface)
- [ ] **AC3.4:** Existing plugin UIs using `useAllParameters()` work without modification

---

## Edge Cases & Error Handling

### Edge Case 1: Component Unmounts Before Connection Established

**Scenario:** User navigates away from parameter view before WebSocket connects

**Expected Behavior:**
- Pending fetch is cancelled
- No memory leaks from dangling promises or timers
- No console errors about setState on unmounted component

**Acceptance Criterion:**
- [ ] **AC4.1:** Hook properly cleans up all async operations in its cleanup function

---

### Edge Case 2: Rapid Mount/Unmount Cycles

**Scenario:** React strict mode or fast navigation causes multiple mount/unmount cycles

**Expected Behavior:**
- Each mount gets fresh data once connection is ready
- No duplicate requests from previous mounts
- Connection state monitoring doesn't accumulate listeners

**Acceptance Criterion:**
- [ ] **AC4.2:** Hook handles React 18 Strict Mode double-mounting without duplicate fetches

---

### Edge Case 3: Maximum Reconnection Attempts Reached

**Scenario:** WebSocket fails to connect after max retry attempts (e.g., dev server crashed)

**Expected Behavior:**
- Hook stops retrying after reasonable attempts
- Clear error message: "Could not connect to dev server. Is `wavecraft start` running?"
- `isLoading` becomes `false`, `error` is set with helpful guidance

**Acceptance Criterion:**
- [ ] **AC4.3:** After WebSocket exhausts reconnection attempts, hook fails gracefully with actionable error

---

### Edge Case 4: Connection Established But getAllParameters() Fails

**Scenario:** WebSocket connects successfully, but the engine doesn't respond to `getAllParameters` request (timeout, error response, etc.)

**Expected Behavior:**
- Distinguish between transport failure (no connection) and application failure (bad response)
- Error message indicates the specific failure: "Parameter fetch timed out" vs "Dev server not reachable"
- `isLoading` → `false`, `error` state populated

**Acceptance Criterion:**
- [ ] **AC4.4:** Hook differentiates between connection failures and request failures in error messages

---

## Non-Functional Requirements

### Performance

- [ ] **NFR1:** Retry logic adds minimal overhead (< 100ms) when connection is already established
- [ ] **NFR2:** No more than one pending `getAllParameters()` request at a time per hook instance
- [ ] **NFR3:** Connection status polling (if used) has reasonable frequency (≤ 1 Hz)

### Developer Experience

- [ ] **NFR4:** Console logs provide clear visibility into connection state and retry attempts (at debug level)
- [ ] **NFR5:** Error messages include next steps (e.g., "Run `wavecraft start` to start the dev server")

### Testing

- [ ] **NFR6:** Unit tests cover all edge cases above
- [ ] **NFR7:** Integration tests validate behavior against real WebSocketTransport with delayed connection
- [ ] **NFR8:** Manual test plan includes browser dev mode startup sequences

---

## Out of Scope

The following are explicitly **not** included in this feature:

- ❌ Changes to nih-plug integration or native plugin parameter handling
- ❌ Optimizations to parameter change notification system
- ❌ UI components for displaying connection status (separate feature)
- ❌ WebSocket transport improvements beyond connection state awareness
- ❌ Parameter caching or persistence across page reloads

---

## Technical Constraints

1. **Single Package:** All changes must be in `@wavecraft/core` (no engine crate changes)
2. **No Breaking Changes:** Hook API (`UseAllParametersResult`) must remain compatible
3. **Transport Agnostic:** Solution must work with both `WebSocketTransport` and `NativeTransport`
4. **React Best Practices:** Follow React 18 concurrent rendering guidelines, proper cleanup patterns

---

## Definition of Done

- ✅ All acceptance criteria (AC1.1–AC4.4) pass
- ✅ All non-functional requirements (NFR1–NFR8) verified
- ✅ Unit tests added for hook logic
- ✅ Integration tests validate against delayed WebSocket connection
- ✅ Manual testing in browser dev mode confirms fix
- ✅ No regressions in native plugin mode (tested in Ableton/Logic)
- ✅ Documentation updated (JSDoc comments, changelog entry)
- ✅ Code review completed by architect
- ✅ Feature spec archived after PR merge

---

## Next Steps

**Handoff to Orchestrator** for routing to the Architect agent to create a low-level design.

The Architect should:
1. Review current `useAllParameters()` and `useConnectionStatus()` implementations
2. Determine if event-based or polling-based approach is better
3. Design the retry/queue mechanism
4. Specify error handling strategy
5. Define test scenarios in detail

