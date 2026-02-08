# User Stories: Macro API Simplification

## Overview

The `wavecraft_plugin!` macro currently requires too many properties that add boilerplate without adding value for developers. This feature simplifies the macro API to the absolute minimum needed for a working plugin.

## Version

**Target Version:** `0.8.0` (minor bump from `0.7.1`)

**Rationale:** This is a breaking API change that simplifies the developer interface. While it reduces complexity, it changes the public macro API signature, warranting a minor version bump.

---

## User Story 1: Minimal Plugin Definition

**As a** plugin developer using Wavecraft  
**I want** to define a plugin with minimal boilerplate  
**So that** I can focus on DSP logic instead of configuration

### Acceptance Criteria

- [ ] `wavecraft_plugin!` macro only requires `name` and `signal` properties
- [ ] `vendor`, `url`, `email` properties are removed (no longer exposed)
- [ ] `crate` property is removed (internal implementation detail)
- [ ] Minimal working example:
  ```rust
  wavecraft_plugin! {
      name: "My Plugin",
      signal: SignalChain![MyProcessor],
  }
  ```

### Notes
- Plugin metadata like vendor, URL, email should be derived from `Cargo.toml` or use sensible defaults
- The `crate` property was never meant to be user-configurable

---

## User Story 2: Consistent Signal Chain API

**As a** plugin developer  
**I want** a consistent way to define my signal processing chain  
**So that** I don't have confusion about when to use `SignalChain` vs bare processors

### Acceptance Criteria

- [ ] `signal` property only accepts `SignalChain![...]` syntax
- [ ] Single processors must be wrapped: `SignalChain![MyProcessor]`
- [ ] Multiple processors use same syntax: `SignalChain![A, B, C]`
- [ ] `Chain!` macro is renamed to `SignalChain!` for consistency
- [ ] Clear error message if user tries to use a bare processor

### Notes
- This eliminates the dual API (bare processor vs chain)
- Makes the DSL more predictable and consistent

---

## User Story 3: Automatic Metadata Derivation

**As a** plugin developer  
**I want** plugin metadata automatically derived from my `Cargo.toml`  
**So that** I don't duplicate information across files

### Acceptance Criteria

- [ ] Plugin vendor derived from `Cargo.toml` `authors` field (or default to "Unknown")
- [ ] Plugin URL derived from `Cargo.toml` `homepage` or `repository` (or default to empty)
- [ ] Plugin email derived from first author's email in `Cargo.toml` (or default to empty)
- [ ] Plugin version derived from `Cargo.toml` `version` (already implemented)

### Notes
- VST3/CLAP don't strictly require these fields
- Defaults should allow compilation without warnings
- Advanced users can override via additional (optional) properties if needed in future

---

## Open Questions for Architect

1. **Metadata Derivation:** How to read `Cargo.toml` at macro expansion time? Should this be a proc-macro or build-time generation?

2. **VST3 Class ID Generation:** Currently uses hash of name+vendor. Does removing vendor property break deterministic ID generation?

3. **Signal Type Enforcement:** Should the macro reject non-`SignalChain` types at compile time? How?

4. **Migration Path:** Should we provide deprecation warnings first, or is this a clean breaking change?

5. **`crate` Property:** What was the original purpose? Can it be safely removed or made internal?

---

## Success Metrics

- ✅ Plugin definition reduced from ~9 lines to ~4 lines
- ✅ No loss of functionality (VST3/CLAP export still works)
- ✅ Clear compile-time errors for misuse
- ✅ Existing template uses simplified API

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md)
- [Declarative Plugin DSL](../../architecture/high-level-design.md#declarative-plugin-dsl)
- [Coding Standards](../../architecture/coding-standards.md)
