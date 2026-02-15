# Implementation Progress — Processor Presence Hook

## Scope implemented

- Added codegen-first processor presence capability (no runtime IPC endpoint).
- Added processor metadata export from macro FFI (`wavecraft_get_processors_json`).
- Added loader support for processor metadata parsing.
- Added processor TypeScript generation (`ui/src/generated/processors.ts`) on `wavecraft start` and hot reload.
- Added `@wavecraft/core` processor typing + hooks:
  - `useHasProcessor(processorId: string): boolean`
  - `useAvailableProcessors(): readonly string[]`
- Added template runtime side-effect import for generated processors module.

## Status checklist

- [x] Macro processor metadata export
- [x] Loader parsing for processor metadata
- [x] CLI/start generation path (`processors.ts`)
- [x] Hot reload generation path (`processors.ts`)
- [x] Core types/hooks/exports
- [x] Template runtime import for generated processors
- [x] Unit tests for codegen and hooks
- [ ] Full repo verification (`cargo xtask ci-check`) pending
