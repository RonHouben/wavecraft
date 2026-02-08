## Summary

**Milestone 18: Audio Pipeline Fixes & Mocking Cleanup (v0.10.0)**

Fixes three critical audio architecture gaps in dev mode (`wavecraft start`) and removes unused synthetic metering infrastructure. This milestone ensures beta testers get a working audio development experience before M19 (User Testing).

**Key changes:**
1. **Full-duplex audio** — `AudioServer` now opens separate cpal input/output streams connected by an `rtrb` SPSC ring buffer. Audio flows: mic → deinterleave → FfiProcessor::process() → interleave → speakers.
2. **Lock-free parameter sync** — New `AtomicParameterBridge` maps parameter IDs to `Arc<AtomicF32>`. WebSocket thread writes via `store(Relaxed)`, audio thread reads via `load(Relaxed)`. Zero allocations, zero locks on the audio thread.
3. **Mocking cleanup** — `MeterGenerator` (synthetic animated meter data) deleted entirely. Fallback when no vtable = silent zeros (honest representation).
4. **UI reconnection fix** — `useAllParameters` now watches `useConnectionStatus` and re-fetches parameters when WebSocket connects (event-driven, no polling).

## Changes

- **Engine/DSP**:
  - `wavecraft-dev-server/src/audio_server.rs` — Rewritten with full-duplex cpal streams and rtrb ring buffers
  - `wavecraft-dev-server/src/atomic_params.rs` — New lock-free parameter bridge
  - `wavecraft-dev-server/src/ffi_processor.rs` — Vec allocations replaced with stack arrays (RT-safe)
  - `wavecraft-metering/src/dev.rs` — Deleted (MeterGenerator removed)
  - `wavecraft-dev-server/Cargo.toml` — Added rtrb dependency
- **UI**:
  - `ui/packages/core/src/hooks/useAllParameters.ts` — Connection-aware parameter retry
- **Build/Config**:
  - `engine/Cargo.toml` — Version bump to 0.10.0
  - All crate Cargo.toml versions updated
- **Documentation**:
  - `docs/architecture/high-level-design.md` — Full-duplex diagram, AtomicParameterBridge docs, updated crate descriptions
  - `docs/architecture/coding-standards.md` — Two new patterns: Lock-Free Parameter Bridge, SPSC Ring Buffer
  - `docs/roadmap.md` — M18 marked complete (18/21, 86%)
  - Feature spec archived to `_archive/audio-pipeline-fixes/`

## Commits

- `b850840` docs: archive M18 feature spec and update roadmap/architecture docs
- `7543f9c` feat(audio): refactor audio server to use lock-free ring buffer for meter updates
- `15be063` feat(docs): add QA report for audio pipeline fixes and mocking cleanup
- `5eb57f0` Bump version to 0.10.0 and update dependencies

## Related Documentation

- [User Stories](./user-stories.md)
- [Low-Level Design](./low-level-design-audio-pipeline-fixes.md)
- [Implementation Plan](./implementation-plan.md)
- [Implementation Progress](./implementation-progress.md)
- [QA Report](./QA-report.md)
- [Architectural Review](./architectural-review.md)

## Testing

- [x] CI passes: `cargo xtask ci-check` (146 engine + 28 UI + 57 CLI tests)
- [x] Linting passes: cargo fmt, clippy, ESLint, Prettier
- [x] Manual audio verification: gain slider audibly changes output
- [x] QA review: PASS (3 Medium findings fixed, 0 Critical/High)
- [x] Architecture review: APPROVED

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed (18 new audio-feature tests)
- [x] Documentation updated (high-level-design.md, coding-standards.md)
- [x] No linting errors
- [x] Feature spec archived
- [x] Roadmap updated
