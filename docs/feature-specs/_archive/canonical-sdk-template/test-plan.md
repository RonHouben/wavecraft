# Test Plan: Canonical SDK Template Refactor

## Overview
- **Date**: 2026-02-13
- **Branch**: `refactor/canonical-sdk-template`

## Test Summary
| Status | Count |
|---|---:|
| ✅ PASS | 13 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Results
1. ✅ `git checkout refactor/canonical-sdk-template` (already on branch)
2. ✅ `cargo xtask ci-check` (lint + tests passed)
3. ✅ `cargo run --manifest-path cli/Cargo.toml -- create TestPlugin --output target/tmp/test-plugin --no-git`
4. ✅ `cd target/tmp/test-plugin/engine && cargo clippy --all-targets -- -D warnings`
5. ✅ `cargo check --manifest-path /Users/ronhouben/code/private/wavecraft/engine/Cargo.toml --workspace`
6. ✅ `./scripts/setup-dev-template.sh`
7. ✅ `cargo xtask dev` started WS + Vite (`ws://127.0.0.1:9000`, `http://localhost:5173`)
8. ✅ Browser loaded, WebSocket connected, parameters visible
9. ✅ HMR verified by editing `sdk-template/ui/src/App.tsx` (marker `[HMR_TEST]` rendered)
10. ✅ Engine hot-reload verified by editing `sdk-template/engine/src/lib.rs` (`Hot-reload complete — 2 parameters`)
11. ✅ Package alias verified by editing `ui/packages/core/src/hooks/useConnectionStatus.ts` and seeing live UI behavior change
12. ✅ `cd ui && npm test` (`9 passed`, `58 passed`)
13. ✅ Stale grep sweep found zero live matches for `sdk-templates` and `wavecraft-example`

## Verification Checklist
- [x] `./scripts/setup-dev-template.sh` completes without errors
- [x] `cargo xtask dev` starts WS + Vite servers
- [x] UI loads at `http://localhost:5173`
- [x] WebSocket connects and parameters appear
- [x] HMR works for `sdk-template/ui/src/App.tsx` edits
- [x] Engine hot-reload works for `sdk-template/engine/src/lib.rs` edits
- [x] Package alias works (`ui/packages/core/src/*.ts` edit reflected)
- [x] Clean shutdown verified via SIGINT (Ctrl+C-equivalent)
- [x] `wavecraft create` still produces compilable projects

## Issues Found
### Low: React duplicate-key warning in dev UI console
- Observed repeatedly during smoke test: `Warning: Encountered two children with the same key...`
- Recommendation: ensure parameter list keys are globally unique.

## Notes
- Temporary smoke-test edits were reverted.
- Generated artifact `target/tmp/test-plugin` was removed.
