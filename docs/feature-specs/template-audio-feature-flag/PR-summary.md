## Summary

Fixes `wavecraft start` failing with `error[E0432]: unresolved import wavecraft_dev_server::audio_server` on newly scaffolded plugin projects.

## Problem

The `audio_server` module in `wavecraft-dev-server` is gated behind `#[cfg(feature = "audio")]`. The project template's `Cargo.toml` depended on `wavecraft-dev-server` without enabling this feature, so the `dev-audio` binary couldn't find the import.

## Changes

### CLI / Templates
- **`cli/sdk-templates/new-project/react/engine/Cargo.toml.template`** — Added `features = ["audio"]` to the `wavecraft-dev-server` dependency

## Testing

- [x] `cargo xtask ci-check` passes
- [x] Scaffolded fresh plugin with `wavecraft create` — generated `Cargo.toml` includes `features = ["audio"]`
- [x] Confirmed `E0432: unresolved import audio_server` error no longer occurs

## Checklist

- [x] One-line fix, minimal risk
- [x] CI passes
- [x] Tested with fresh project scaffold
