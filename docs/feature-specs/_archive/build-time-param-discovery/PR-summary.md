# PR Summary: Build-Time Parameter Discovery Fix

## Summary

Fixed `wavecraft start` hanging at "Loading plugin parameters..." on macOS. Root cause: Loading the plugin dylib triggered nih-plug's VST3/CLAP static initializers, which block on `AudioComponentRegistrar`. Solution: Feature-gated `nih_export_clap!` / `nih_export_vst3!` behind `#[cfg(not(feature = "_param-discovery"))]` in the `wavecraft_plugin!` macro output.

## Changes

### Engine/DSP
- **`engine/crates/wavecraft-macros/src/plugin.rs`**: Wrapped nih-plug export macros with `#[cfg]` guards
- **`engine/crates/wavecraft-bridge/src/plugin_loader.rs`**: Added `load_params_from_file()` method for sidecar JSON cache

### CLI
- **`cli/src/commands/start.rs`**: Implemented two-phase build strategy:
  - Phase 1: Fast build with `_param-discovery` to extract params
  - Phase 2: Sidecar JSON cache with mtime-based invalidation
  - Graceful fallback for older plugins

### Template
- **`cli/sdk-templates/new-project/react/engine/Cargo.toml.template`**: Added `_param-discovery = []` feature flag

### Documentation
- **`docs/architecture/development-workflows.md`**: Updated with parameter discovery flow
- **`docs/architecture/declarative-plugin-dsl.md`**: Documented feature-gated exports
- **`docs/roadmap.md`**: Added changelog entry

## Related Documentation

- [Low-Level Design](docs/feature-specs/_archive/build-time-param-discovery/low-level-design-build-time-param-discovery.md)
- [Implementation Plan](docs/feature-specs/_archive/build-time-param-discovery/implementation-plan.md)
- [Implementation Progress](docs/feature-specs/_archive/build-time-param-discovery/implementation-progress.md)
- [Test Plan](docs/feature-specs/_archive/build-time-param-discovery/test-plan.md)
- [QA Report](docs/feature-specs/_archive/build-time-param-discovery/QA-report.md)

## Testing

### Automated Tests
- ✅ 87 engine tests + 28 UI tests passing
- ✅ All linting checks clean (clippy, ESLint, Prettier)
- ✅ Template validation passing (clippy on generated code)

### Manual Tests
- ✅ Symbol verification with `nm -g` (nih-plug exports excluded with feature, included without)
- ✅ `wavecraft start` loads params without hanging
- ✅ Sidecar cache working correctly
- ✅ Backward compatibility confirmed (fallback for older plugins)

## Checklist

- [x] Code follows project coding standards
- [x] All tests passing (`cargo xtask ci-check`)
- [x] Documentation updated
- [x] Feature spec archived to `_archive/`
- [x] Roadmap updated
- [x] QA approved (zero issues)
- [x] Backward compatible

## QA Status

**✅ APPROVED** — Zero critical, high, or medium issues found.

All quality checks passed:
- Template validation clean
- Symbol verification confirms feature gate works correctly
- No regressions detected
- Architecture compliance verified
