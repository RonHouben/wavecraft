# Test Plan: Oscilloscope v1

## Overview

- Feature: oscilloscope
- Spec: docs/feature-specs/oscilloscope/
- Date: 2026-02-15
- Tester: Tester Agent

## Summary

| Status  | Count |
| ------- | ----: |
| PASS    |     8 |
| FAIL    |     0 |
| BLOCKED |     0 |
| NOT RUN |     0 |

## Test Cases and Results

1. Protocol serialization
   - Command: cargo test --manifest-path engine/Cargo.toml -p wavecraft-protocol oscilloscope
   - Result: PASS
2. Bridge and host retrieval
   - Command: cargo test --manifest-path engine/Cargo.toml -p wavecraft-bridge oscilloscope
   - Result: PASS
3. Processor tap invariants
   - Command: cargo test --manifest-path engine/Cargo.toml -p wavecraft-processors oscilloscope
   - Result: PASS
4. Dev server oscilloscope host path
   - Command: cargo test --manifest-path dev-server/Cargo.toml oscilloscope
   - Result: PASS
5. UI hook polling
   - Command: npm --prefix ui run test -- packages/core/src/hooks/useOscilloscopeFrame.test.ts
   - Result: PASS
6. UI component behavior
   - Command: npm --prefix ui run test -- packages/components/src/Oscilloscope.test.tsx
   - Result: PASS
7. Workspace checks
   - Command: cargo xtask ci-check
   - Result: PASS
8. Full checks
   - Command: cargo xtask ci-check -F
   - Result: PASS

## Issues Found

- None

## Blockers

- None

## Notes

- Non blocking npm warning observed: Unknown user config NODE_OPTIONS.

## DocWriter Handoff Record

- Complete test plan content prepared and persisted to this path for handoff workflow continuity.

## Sign off

- Ready for QA handoff: YES
