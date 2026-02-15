# Implementation Progress: Oscilloscope (v1)

## Overview

- Feature: `oscilloscope`
- Date: 2026-02-15
- Branch: `feature/oscilloscope-v1`
- Scope: Single-PR end-to-end implementation

## Completed Work

### 1) Protocol + bridge contracts

- Added IPC method: `getOscilloscopeFrame`
- Added protocol types:
  - `OscilloscopeTriggerMode`
  - `OscilloscopeChannelView`
  - `OscilloscopeFrame`
  - `GetOscilloscopeFrameResult`
- Extended `ParameterHost` with `get_oscilloscope_frame()`
- Added handler dispatch branch and response path in `wavecraft-bridge`
- Updated in-memory host provider support and tests

### 2) Engine processor (`wavecraft-processors`)

- Added `OscilloscopeTap` observation-only processor
- Captures fixed-size 1024-point stereo snapshots
- Implements rising zero-crossing trigger alignment
- Implements no-signal detection
- Uses lock-free SPSC channel (`rtrb`) for frame publication
- Verified passthrough invariance (audio not modified)

### 3) Plugin editor bridge wiring (WebView path)

- Added oscilloscope consumer plumbing through:
  - `wavecraft-nih_plug` editor config
  - platform webview setup (macOS/windows)
  - `PluginEditorBridge` host implementation
- Added macro runtime wiring in `wavecraft_plugin!` generated plugin:
  - oscilloscope channel creation
  - post-process capture tap
  - editor consumer handoff

### 4) Dev-server/browser wiring

- Added latest oscilloscope frame storage to `DevServerHost`
- Added setter/getter path for `getOscilloscopeFrame`
- Added capture in dev audio callback using `OscilloscopeTap`
- Forwarded oscilloscope frame consumer in CLI `start` runtime loop

### 5) UI core API (`@wavecraft/core`)

- Added TypeScript contracts and method constant
- Added `getOscilloscopeFrame()` helper
- Added `useOscilloscopeFrame()` hook (RAF-based polling, single in-flight request)
- Exported new types/api/hook from package index
- Added hook and bridge test coverage

### 6) UI components (`@wavecraft/components`)

- Added reusable `Oscilloscope` component with:
  - L/R overlay default
  - channel view options (`overlay`, `left`, `right`)
  - trigger mode control defaulting to rising zero-crossing
  - no-signal flat-line state with `No signal` label
  - requestAnimationFrame-driven canvas rendering loop
- Added component tests
- Exported component from package index
- Integrated component in `sdk-template/ui/src/App.tsx`

## Validation

- Targeted Rust tests for protocol/bridge/processors: PASS
- Targeted UI tests for oscilloscope hook/component: PASS
- Compile checks for CLI + sdk-template engine: PASS

## Notes

- v1 intentionally excludes FFT/history/advanced controls to preserve scope.
- Transport remains pull-based (`getOscilloscopeFrame`) for both browser-dev and plugin WebView.
