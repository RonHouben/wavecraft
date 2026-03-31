## Summary

Implements runtime-reorderable signal chain processor ordering — a full-stack feature spanning engine macros, IPC protocol, lock-free handoff in generated plugin code, and a drag-and-drop React UI component.

The macro DSL receives a **breaking change**: `signal: SignalChain![...]` is replaced by `processors: [...]`. The macro now generates a `ProcessorOrder` struct with lock-free (`AtomicBool` + `AtomicU8`) pending-order handoff and a 256-sample crossfade on reorder. Processor order is persisted per-preset via `serialize_fields`/`deserialize_fields`. New IPC methods (`getProcessorOrder`, `setProcessorOrder`, `processorOrderChanged`) coordinate UI ↔ audio. A new `ProcessorOrderAccess` trait in `wavecraft-bridge` allows the bridge to query and update order without knowing plugin internals. On the UI side, `@wavecraft/core` gains `ProcessorOrderClient` and `useProcessorOrder`, while `@wavecraft/components` ships `<SignalChain>` and `<SignalChainItem>` powered by `@dnd-kit`. All 218/218 UI tests pass; all engine tests pass; lint and typecheck clean.

## Changes

- **Engine — Macros (`wavecraft-macros`)**: Replaced `signal: SignalChain![...]` DSL with `processors: [...]`. Codegen (`codegen.rs`) now emits `ProcessorOrder` struct, `ProcessorOrderAccess` impl, `serialize_fields`/`deserialize_fields` for preset persistence, and 256-sample crossfade on reorder. Parser (`parse.rs`) updated accordingly.
- **Engine — Protocol (`wavecraft-protocol`)**: Added `getProcessorOrder`, `setProcessorOrder` request/response types and `processorOrderChanged` notification in `ipc/methods.rs` and `ipc.rs`. Exported new types from `lib.rs`.
- **Engine — Bridge (`wavecraft-bridge`)**: Added `ProcessorOrderAccess` trait (`host.rs`), `InMemoryHost` implementation (`in_memory_host.rs`), and handler dispatch for the new IPC methods (`handler.rs`). Updated `lib.rs` exports.
- **Engine — Plugin integration (`wavecraft-nih_plug`)**: Editor bridge and macOS/webview integration updated to wire `ProcessorOrderAccess` through the editor (`editor/bridge.rs`, `editor/macos.rs`, `editor/mod.rs`, `editor/webview.rs`). `lib.rs` updated.
- **SDK template (`sdk-template/engine/src/lib.rs`, `sdk-template/ui/src/App.tsx`)**: Updated generated plugin to use new `processors: [...]` DSL and wired `<SignalChain>` into `App.tsx`.
- **UI — `@wavecraft/core`**: Added `ProcessorOrderClient` (`ipc/ProcessorOrderClient.ts`), `useProcessorOrder` hook (`hooks/useProcessorOrder.ts`), `IPC_CONSTANTS` (`ipc/constants.ts`), and exported all from `index.ts`.
- **UI — `@wavecraft/components`**: Added `<SignalChain>` and `<SignalChainItem>` components (`signalChain/`), `useSignalChainPresentation` hook, and exported from `index.ts`. Added `@dnd-kit` dependencies.
- **Build/Config**: `.gitignore` updated; `ui/package-lock.json` and `ui/packages/components/package-lock.json` updated for new `@dnd-kit` dependencies.
- **Documentation**: Updated `declarative-plugin-dsl.md`, `high-level-design.md`, `sdk-architecture.md`, `guides/sdk-getting-started.md`, and `roadmap.md` to reflect DSL change and new IPC contract. Feature spec fully archived. TDD skill and documentation added.

## Commits

```
36c08fa fix: resolve CI blockers (lockfile, SignalChain export, serialize_fields in Params impl, tracing in generated code)
a336412 feat: add processor order management to the IPC protocol
69a9a2e feat: add TDD documentation including principles, anti-patterns, and testing guidelines
5fec35a feat: add implementation plan for runtime-reorderable SignalChain
```

## Related Documentation

- [Implementation Plan](./_archive/ui-signal-chain-reorder/implementation-plan.md)
- [Implementation Progress](./_archive/ui-signal-chain-reorder/implementation-progress.md)
- [Low-Level Design](./_archive/ui-signal-chain-reorder/low-level-design-ui-signal-chain-reorder.md)
- [Test Plan](./_archive/ui-signal-chain-reorder/test-plan.md)
- [QA Report](./_archive/ui-signal-chain-reorder/QA-report.md)
- [User Stories](./_archive/ui-signal-chain-reorder/user-stories.md)

## Testing

- [x] All 218/218 UI tests pass (`vitest`)
- [x] All engine tests pass (`cargo test --all`)
- [x] Lint passes (`cargo clippy`, `eslint`)
- [x] TypeScript typecheck passes (`tsc --noEmit`)
- [x] `cargo xtask ci-check` passes (0 failures)
- [x] Drag-and-drop UI manually verified via VS Code Simple Browser (screenshot evidence in test-plan.md)
- [x] Processor order persists across preset save/load
- [x] 256-sample crossfade verified (no clicks/pops on reorder)
- [x] QA approved

## Checklist

- [x] Code follows project coding standards (Rust + TypeScript)
- [x] No allocations or locks on audio thread (lock-free `AtomicBool`/`AtomicU8` handoff)
- [x] All public APIs documented with `///` doc comments
- [x] Tests added/updated (218 UI tests, engine unit tests)
- [x] No `unwrap()` in production paths
- [x] `cargo fmt` applied
- [x] `cargo clippy` passes with no warnings
- [x] Feature spec archived to `docs/feature-specs/_archive/ui-signal-chain-reorder/`
- [x] Roadmap updated (`docs/roadmap.md`)

## Breaking Changes

> ⚠️ **Breaking DSL change**: The `signal: SignalChain![ProcessorA, ProcessorB]` syntax in the `wavecraft_plugin!` macro is **replaced** by `processors: [ProcessorA, ProcessorB]`. Existing plugins using the old syntax must update their `wavecraft_plugin!` invocation.
