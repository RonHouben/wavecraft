# Developer SDK (Milestone 8)

## Summary

This PR completes **Milestone 8: Developer SDK**, transforming VstKit from an internal framework into a reusable development kit that other developers can use to build their own VST/CLAP plugins with Rust + React.

**Version**: 0.4.0

## Key Deliverables

### 5-Crate SDK Architecture

Restructured the codebase into a clean, modular SDK with clear domain boundaries:

| Crate | Purpose |
|-------|---------|
| `vstkit-protocol` | IPC contracts, JSON-RPC types, parameter specs |
| `vstkit-dsp` | Pure audio processing (`Processor` trait), no framework deps |
| `vstkit-bridge` | IPC handling (`ParameterHost` trait), message routing |
| `vstkit-metering` | Real-time meters via SPSC ring buffer (lock-free) |
| `vstkit-core` | Framework integration, `vstkit_plugin!` macro, nih-plug wrapper |

### Zero-Boilerplate Plugin Macro

```rust
vstkit_plugin! {
    ident: MyPlugin,
    name: "My Plugin",
    vendor: "My Company",
    url: "https://example.com",
    email: "contact@example.com",
    version: env!("CARGO_PKG_VERSION"),
    audio: { inputs: 2, outputs: 2 },
    params: [MyParams],
    processor: MyProcessor,
}
```

### Plugin Template

Complete working template (`vstkit-plugin-template/`) demonstrating:
- Full SDK integration
- Custom DSP processor
- React UI with meters
- xtask bundler for VST3/CLAP

### Documentation

- SDK Getting Started guide (`docs/guides/sdk-getting-started.md`)
- Updated high-level design with SDK architecture
- Added `unwrap()`/`expect()` coding standards

## Changes

### Engine/DSP
- Restructured 5 SDK crates with clear boundaries
- Implemented `vstkit_plugin!` macro for plugin generation
- Added `Processor` trait for user DSP code
- Added `ParamSet` trait and `vstkit_params!` macro
- Extracted `ParameterHost` trait for IPC abstraction

### Template
- Created complete plugin template project
- Added engine with custom gain processor
- Added React UI with TailwindCSS
- Added xtask build system integration

### Documentation
- Added SDK Getting Started guide
- Updated coding standards with error handling guidelines
- Updated high-level design with SDK architecture

### Build/Config
- Version bumped to 0.4.0
- Added template to workspace structure
- Updated nih-plug dependency pinning

## Test Results

```
Engine Tests: 111 passed, 0 failed, 4 ignored (environment-dependent)
UI Tests:     35 passed, 0 failed
Manual Tests: 22/22 passed
Linting:      All checks passed (cargo fmt, clippy, ESLint, Prettier)
```

## Commits

- `d48e93f` docs: mark Milestone 8 complete, archive feature spec
- `2e90f61` fix: update IPC response serialization to use expect()
- `c28d84a` fix: update test plan to reflect all tests completed
- `b1e55e7` feat: enhance Playwright MCP UI testing skill
- `e546ba5` fix: resolve nih-plug version mismatch in template
- `add81be` feat: implement `vstkit_plugin!` macro
- `f22636a` feat: update SDK version to 0.4.0
- ... and 30+ more commits

## Related Documentation

- [User Stories](docs/feature-specs/_archive/developer-sdk/user-stories.md)
- [Low-Level Design](docs/feature-specs/_archive/developer-sdk/low-level-design-developer-sdk.md)
- [Implementation Plan](docs/feature-specs/_archive/developer-sdk/implementation-plan.md)
- [Test Plan](docs/feature-specs/_archive/developer-sdk/test-plan.md)
- [QA Report](docs/feature-specs/_archive/developer-sdk/QA-report.md)

## Testing

- [x] All 111 engine tests pass
- [x] All 35 UI tests pass
- [x] 22/22 manual test cases pass
- [x] Template compiles and builds VST3/CLAP bundles
- [x] Visual testing verified via Playwright
- [x] Code signing verified (ad-hoc)
- [x] Linting passes (Rust + TypeScript)

## Checklist

- [x] Code follows project coding standards
- [x] All tests pass locally
- [x] Documentation updated
- [x] Feature spec archived
- [x] Roadmap updated (M8 complete)
- [x] Version bumped to 0.4.0
- [x] QA approved
- [x] Architect review complete
