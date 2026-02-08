# Implementation Plan: Macro API Simplification

## Overview

Simplify the `wavecraft_plugin!` macro API from 5 required properties to 2 (`name` and `signal`), reducing plugin definitions from ~9 lines to ~4 lines. Plugin metadata will be automatically derived from Cargo environment variables, and signal chains will use consistent `SignalChain!` syntax.

## Requirements

- Minimize `wavecraft_plugin!` macro properties (remove `vendor`, `url`, `email`; make `crate` optional)
- Derive metadata from Cargo environment variables (`CARGO_PKG_*`)
- Rename `Chain!` → `SignalChain!` with deprecation path
- Update VST3/CLAP ID generation to use package name
- Update CLI templates to use new API
- Provide clear compile-time error messages
- Create migration guide for existing users

## Architecture Changes

- `engine/crates/wavecraft-macros/src/plugin.rs` — Simplify `PluginDef` struct, metadata derivation
- `engine/crates/wavecraft-macros/src/lib.rs` — Update docstring examples
- `engine/crates/wavecraft-dsp/src/combinators/mod.rs` — Add `SignalChain!`, deprecate `Chain!`
- `engine/crates/wavecraft-dsp/src/lib.rs` — Export `SignalChain!`
- `engine/crates/wavecraft-core/src/prelude.rs` — Re-export `SignalChain!`
- `engine/crates/wavecraft-nih_plug/src/prelude.rs` — Re-export `SignalChain!`
- `cli/sdk-templates/new-project/react/engine/src/lib.rs` — Use new macro API
- `cli/sdk-templates/new-project/react/engine/Cargo.toml.template` — Add metadata fields
- `docs/MIGRATION-0.8.md` — New migration guide
- `docs/architecture/high-level-design.md` — Update DSL examples
- `docs/architecture/coding-standards.md` — Update macro guidelines
- `README.md` — Update quick start examples

---

## Implementation Steps

### Phase 1: Core Macro Changes (High Priority)

#### 1. **Simplify `PluginDef` struct** (File: `engine/crates/wavecraft-macros/src/plugin.rs`)
   - **Action**: Remove `vendor`, `url`, `email` fields; make `krate` optional
   - **Why**: Core breaking change that enables all other simplifications
   - **Dependencies**: None
   - **Risk**: High — Breaks existing plugins
   - **Details**:
     ```rust
     struct PluginDef {
         name: LitStr,
         signal: Expr,
         krate: Option<Path>,  // Optional, defaults to ::wavecraft
         // Removed: vendor, url, email
     }
     ```

#### 2. **Update `Parse` implementation** (File: `engine/crates/wavecraft-macros/src/plugin.rs`)
   - **Action**: Remove vendor/url/email parsing logic; make `crate` optional with default
   - **Why**: Match new struct definition
   - **Dependencies**: Step 1
   - **Risk**: Medium — Must handle backward compatibility gracefully
   - **Details**:
     - Remove `vendor:`, `url:`, `email:` property parsing
     - Make `krate` default to `Some(syn::parse_quote!(::wavecraft))`
     - Update error messages to reflect simplified API

#### 3. **Implement metadata derivation** (File: `engine/crates/wavecraft-macros/src/plugin.rs`)
   - **Action**: Derive vendor/URL/email from Cargo environment variables in `wavecraft_plugin_impl()`
   - **Why**: Automatically pull metadata from `Cargo.toml` without user input
   - **Dependencies**: Steps 1-2
   - **Risk**: Medium — Cargo env vars may be empty
   - **Details**:
     ```rust
     // In wavecraft_plugin_impl():
     let vendor = env!("CARGO_PKG_AUTHORS")
         .split(',')
         .next()
         .unwrap_or("Unknown")
         .trim();

     let url = {
         let homepage = env!("CARGO_PKG_HOMEPAGE");
         if homepage.is_empty() {
             env!("CARGO_PKG_REPOSITORY")
         } else {
             homepage
         }
     };

     let email = env!("CARGO_PKG_AUTHORS")
         .split(',')
         .next()
         .and_then(|author| {
             author.split('<')
                 .nth(1)
                 .and_then(|s| s.split('>').next())
         })
         .unwrap_or("");
     ```
   - **Edge Cases**:
     - Empty `CARGO_PKG_AUTHORS` → Use "Unknown"
     - No `<email>` in authors field → Use empty string
     - Both homepage and repository empty → Use empty string

#### 4. **Update VST3 ID generation** (File: `engine/crates/wavecraft-macros/src/plugin.rs`)
   - **Action**: Change `generate_vst3_id()` to use package name instead of vendor
   - **Why**: Package name is more stable and unique than vendor
   - **Dependencies**: Steps 1-3
   - **Risk**: High — Changes VST3 class ID (breaking change for existing plugins)
   - **Details**:
     ```rust
     fn generate_vst3_id(name: &str) -> [u8; 16] {
         let package_name = env!("CARGO_PKG_NAME");
         
         let mut hasher = DefaultHasher::new();
         format!("{}{}", package_name, name).hash(&mut hasher);
         let hash = hasher.finish();

         // Convert to 16 bytes (same format as before)
         [
             (hash >> 56) as u8,
             (hash >> 48) as u8,
             (hash >> 40) as u8,
             (hash >> 32) as u8,
             (hash >> 24) as u8,
             (hash >> 16) as u8,
             (hash >> 8) as u8,
             hash as u8,
             0, 0, 0, 0, 0, 0, 0, 0, // Padding
         ]
     }
     ```

#### 5. **Update CLAP ID generation** (File: `engine/crates/wavecraft-macros/src/plugin.rs`)
   - **Action**: Use package name for CLAP ID consistency
   - **Why**: Matches VST3 ID strategy
   - **Dependencies**: Step 4
   - **Risk**: Low — CLAP IDs are less critical than VST3
   - **Details**:
     ```rust
     const CLAP_ID: &'static str = concat!("com.", env!("CARGO_PKG_NAME"));
     ```

#### 6. **Add signal validation** (File: `engine/crates/wavecraft-macros/src/plugin.rs`)
   - **Action**: Detect bare processors (identifiers) and emit helpful compile error
   - **Why**: Guide users to use `SignalChain![]` wrapper
   - **Dependencies**: Steps 1-5
   - **Risk**: Low — Improves error messages
   - **Details**:
     ```rust
     // Add early validation in wavecraft_plugin_impl():
     if let Expr::Path(_) = signal_type {
         return syn::Error::new(
             signal_type.span(),
             "signal property must use `SignalChain!` wrapper.\n\
              \n\
              Did you mean:\n\
              signal: SignalChain![YourProcessor]\n\
              \n\
              Or for multiple processors:\n\
              signal: SignalChain![A, B, C]"
         ).to_compile_error().into();
     }
     ```

#### 7. **Update macro docstring** (File: `engine/crates/wavecraft-macros/src/lib.rs`)
   - **Action**: Update `wavecraft_plugin!` documentation and examples
   - **Why**: Examples should show minimal API
   - **Dependencies**: Steps 1-6
   - **Risk**: Low
   - **Details**:
     ```rust
     /// # Example
     ///
     /// ```rust,no_run
     /// use wavecraft::prelude::*;
     ///
     /// wavecraft_processor!(MyGain => Gain);
     ///
     /// wavecraft_plugin! {
     ///     name: "My Plugin",
     ///     signal: SignalChain![MyGain],
     /// }
     /// ```
     ```

---

### Phase 2: SignalChain Macro Rename (Medium Priority)

#### 8. **Create `SignalChain!` macro** (File: `engine/crates/wavecraft-dsp/src/combinators/mod.rs`)
   - **Action**: Add new `SignalChain!` macro with same implementation as `Chain!`
   - **Why**: More descriptive name, consistent with `wavecraft_plugin!` DSL
   - **Dependencies**: None (independent of Phase 1)
   - **Risk**: Low — Additive change
   - **Details**:
     ```rust
     /// Combines processors into a serial signal chain.
     ///
     /// # Single Processor (Zero Overhead)
     /// ```rust,no_run
     /// use wavecraft_dsp::{SignalChain, builtins::Gain};
     ///
     /// type Single = SignalChain![Gain];
     /// ```
     ///
     /// # Multiple Processors
     /// ```rust,no_run
     /// type Chain = SignalChain![Gain, Passthrough];
     /// ```
     #[macro_export]
     macro_rules! SignalChain {
         // Single processor: no wrapping, zero overhead
         ($single:ty) => {
             $single
         };
         // Multiple: nest into Chain<A, Chain<B, ...>>
         ($first:ty, $($rest:ty),+ $(,)?) => {
             $crate::combinators::Chain<$first, $crate::SignalChain![$($rest),+]>
         };
     }
     ```

#### 9. **Deprecate `Chain!` macro** (File: `engine/crates/wavecraft-dsp/src/combinators/mod.rs`)
   - **Action**: Add deprecation warning, delegate to `SignalChain!`
   - **Why**: Graceful migration path without breaking existing code
   - **Dependencies**: Step 8
   - **Risk**: Low — Backward compatible
   - **Details**:
     ```rust
     /// DEPRECATED: Use `SignalChain!` instead.
     ///
     /// This macro is deprecated and will be removed in 0.9.0.
     /// Please use `SignalChain!` for consistency with the Wavecraft DSL.
     #[deprecated(since = "0.8.0", note = "use `SignalChain!` instead")]
     #[macro_export]
     macro_rules! Chain {
         ($($tt:tt)*) => {
             $crate::SignalChain![$($tt)*]
         };
     }
     ```

#### 10. **Export `SignalChain!`** (File: `engine/crates/wavecraft-dsp/src/lib.rs`)
   - **Action**: Re-export both `Chain` and `SignalChain` at crate root
   - **Why**: Make macros available via `use wavecraft_dsp::{Chain, SignalChain};`
   - **Dependencies**: Steps 8-9
   - **Risk**: Low
   - **Details**:
     ```rust
     pub use combinators::{Chain, SignalChain};
     ```

#### 11. **Update core prelude** (File: `engine/crates/wavecraft-core/src/prelude.rs`)
   - **Action**: Re-export `SignalChain!` in core prelude
   - **Why**: Available via `use wavecraft_core::prelude::*;`
   - **Dependencies**: Step 10
   - **Risk**: Low
   - **Details**:
     ```rust
     pub use wavecraft_dsp::{
         Processor, ProcessorParams, Transport,
         builtins, Chain, SignalChain,  // Add SignalChain
     };
     ```

#### 12. **Update nih_plug prelude** (File: `engine/crates/wavecraft-nih_plug/src/prelude.rs`)
   - **Action**: Re-export `SignalChain!` in main SDK prelude
   - **Why**: Available via `use wavecraft::prelude::*;` in templates
   - **Dependencies**: Step 11
   - **Risk**: Low
   - **Details**:
     ```rust
     pub use wavecraft_core::prelude::*;  // Already includes SignalChain
     ```

---

### Phase 3: CLI Template Updates (Medium Priority)

#### 13. **Update template plugin code** (File: `cli/sdk-templates/new-project/react/engine/src/lib.rs`)
   - **Action**: Use simplified `wavecraft_plugin!` API with `SignalChain![]`
   - **Why**: Template demonstrates best practices
   - **Dependencies**: Steps 1-12
   - **Risk**: Low
   - **Details**:
     ```rust
     // Import everything from Wavecraft SDK
     use wavecraft::prelude::*;

     // Define the processor chain
     wavecraft_processor!({{plugin_name_pascal}}Gain => Gain);

     // Generate the complete plugin from DSL
     wavecraft_plugin! {
         name: "{{plugin_name_title}}",
         signal: SignalChain![{{plugin_name_pascal}}Gain],
     }
     ```

#### 14. **Update template Cargo.toml** (File: `cli/sdk-templates/new-project/react/engine/Cargo.toml.template`)
   - **Action**: Add `authors` and `homepage` fields to package metadata
   - **Why**: Provide source for derived plugin metadata
   - **Dependencies**: Step 13
   - **Risk**: Low
   - **Details**:
     ```toml
     [package]
     name = "{{plugin_name}}"
     version = "0.1.0"
     edition = "2021"
     authors = ["{{author_name}} <{{author_email}}>"]
     homepage = "{{homepage_url}}"

     [dependencies]
     wavecraft = { package = "wavecraft-nih_plug", git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
     ```

#### 15. **Update template variables** (File: `cli/src/template/variables.rs`)
   - **Action**: Add `author_name`, `author_email`, `homepage_url` template variables
   - **Why**: Populate Cargo.toml metadata fields
   - **Dependencies**: Step 14
   - **Risk**: Low
   - **Details**:
     - Add prompts for author name, email, homepage URL
     - Provide sensible defaults (git config, empty strings)
     - Substitute into Cargo.toml.template

---

### Phase 4: Documentation Updates (Low Priority)

#### 16. **Update High-Level Design** (File: `docs/architecture/high-level-design.md`)
   - **Action**: Replace `Chain![]` with `SignalChain![]` in all examples; show minimal macro API
   - **Why**: Documentation should reflect current best practices
   - **Dependencies**: Steps 1-15
   - **Risk**: Low
   - **Details**:
     - Update "Declarative Plugin DSL" section
     - Update "Public API Surface" section
     - Update code examples throughout

#### 17. **Update Coding Standards** (File: `docs/architecture/coding-standards.md`)
   - **Action**: Document new macro API, Cargo.toml metadata derivation, `SignalChain!` usage
   - **Why**: Standards guide should explain the "how" and "why"
   - **Dependencies**: Step 16
   - **Risk**: Low
   - **Details**:
     - Add section: "Declarative Plugin DSL / Minimal API"
     - Explain metadata derivation from Cargo env vars
     - Show `SignalChain!` examples (single vs multiple processors)

#### 18. **Update README** (File: `README.md`)
   - **Action**: Update quick start examples to use new minimal API
   - **Why**: First impression matters — show simplest possible plugin
   - **Dependencies**: Step 17
   - **Risk**: Low
   - **Details**:
     ```rust
     // In README quick start section:
     use wavecraft::prelude::*;

     wavecraft_processor!(MyGain => Gain);

     wavecraft_plugin! {
         name: "My First Plugin",
         signal: SignalChain![MyGain],
     }
     ```

#### 19. **Create migration guide** (File: `docs/MIGRATION-0.8.md`)
   - **Action**: Document breaking changes, migration steps, VST3 ID impact
   - **Why**: Users need clear upgrade path from 0.7.x → 0.8.0
   - **Dependencies**: Steps 1-18
   - **Risk**: Low
   - **Details**:
     - Section 1: Simplified `wavecraft_plugin!` macro (remove properties, add to Cargo.toml)
     - Section 2: `Chain!` → `SignalChain!` rename
     - Section 3: VST3 Class ID changes (breaking change warning)
     - Section 4: Checklist of migration steps
     - Section 5: Rollback instructions (stay on 0.7.x if needed)

---

### Phase 5: Testing (Highest Priority — Run Throughout)

#### 20. **Create macro expansion tests** (File: `engine/crates/wavecraft-macros/tests/plugin_macro.rs`)
   - **Action**: Add unit tests for macro expansion and error cases
   - **Why**: Catch regressions in macro behavior
   - **Dependencies**: Steps 1-7
   - **Risk**: Low
   - **Details**:
     ```rust
     #[test]
     fn minimal_plugin_compiles() {
         let input = quote! {
             wavecraft_plugin! {
                 name: "Test Plugin",
                 signal: SignalChain![MyProcessor],
             }
         };
         // Verify macro expansion succeeds
     }

     #[test]
     fn metadata_derived_from_cargo_env() {
         // Verify vendor/url/email use env! macros in output
         // Parse expanded TokenStream and check for env! calls
     }

     #[test]
     fn bare_processor_without_signal_chain_errors() {
         let input = quote! {
             wavecraft_plugin! {
                 name: "Test",
                 signal: MyProcessor,  // Missing SignalChain!
             }
         };
         let result = wavecraft_plugin(input);
         assert!(result.to_string().contains("signal property must use"));
     }

     #[test]
     fn vst3_id_uses_package_name() {
         // Verify generate_vst3_id() uses CARGO_PKG_NAME, not vendor
     }

     #[test]
     fn crate_property_defaults_to_wavecraft() {
         let input = quote! {
             wavecraft_plugin! {
                 name: "Test",
                 signal: SignalChain![MyProcessor],
                 // No crate: property
             }
         };
         // Verify expanded code uses ::wavecraft
     }
     ```

#### 21. **Update CLI template tests** (File: `cli/tests/template_generation.rs`)
   - **Action**: Verify generated projects use new macro API
   - **Why**: Ensure `wavecraft create` produces correct templates
   - **Dependencies**: Steps 13-15
   - **Risk**: Low
   - **Details**:
     ```rust
     #[test]
     fn generated_plugin_uses_signal_chain() {
         let temp_dir = tempfile::tempdir().unwrap();
         // Run wavecraft create
         // Read generated lib.rs
         // Assert contains "SignalChain!["
         // Assert does NOT contain "vendor:", "url:", "email:"
     }

     #[test]
     fn generated_cargo_toml_includes_metadata() {
         // Verify authors, homepage present in generated Cargo.toml
     }
     ```

#### 22. **Run manual integration tests** (Manual)
   - **Action**: Generate plugin, build, load in DAW
   - **Why**: Verify end-to-end functionality
   - **Dependencies**: All previous steps
   - **Risk**: Medium — Manual testing required
   - **Checklist**:
     - [ ] Generate new plugin: `wavecraft create test-macro-api-simplification --output target/tmp/test-plugin`
     - [ ] Build plugin: `cd target/tmp/test-plugin && cargo xtask bundle`
     - [ ] Verify no compile errors
     - [ ] Install plugin: `cargo xtask install`
     - [ ] Open Ableton Live
     - [ ] Load plugin on a track
     - [ ] Verify plugin name appears correctly
     - [ ] Check plugin info in DAW (vendor should be from Cargo.toml authors)
     - [ ] Test with multiple processors: `SignalChain![A, B]`
     - [ ] Verify `Chain!` macro emits deprecation warning
     - [ ] Verify bare processor without `SignalChain!` shows helpful error

#### 23. **Run automated test suite** (CI validation)
   - **Action**: Run `cargo xtask ci-check` to validate all checks pass
   - **Why**: Catch regressions before merge
   - **Dependencies**: Steps 20-22
   - **Risk**: Low
   - **Details**:
     ```bash
     cargo xtask ci-check
     # Verify:
     # - All Rust tests pass
     # - All UI tests pass
     # - Linting passes (clippy, fmt, eslint)
     # - No deprecation warnings except for Chain! (expected)
     ```

---

## Testing Strategy

### Unit Tests
- **File**: `engine/crates/wavecraft-macros/tests/plugin_macro.rs`
- **Coverage**:
  - Minimal plugin definition compiles
  - Metadata correctly derived from `CARGO_PKG_*` env vars
  - Bare processor (without `SignalChain!`) emits clear error
  - VST3 ID generation uses package name
  - CLAP ID generation uses package name
  - `crate` property defaults to `::wavecraft`

### Integration Tests
- **File**: `cli/tests/template_generation.rs`
- **Coverage**:
  - Generated plugin uses `SignalChain![]` syntax
  - Generated `Cargo.toml` includes `authors` and `homepage`
  - No `vendor`, `url`, `email` in generated macro invocation

### Manual Testing (Checklist)
- [ ] Generate plugin with `wavecraft create`
- [ ] Build with `cargo xtask bundle --debug`
- [ ] Install with `cargo xtask install`
- [ ] Load in Ableton Live
- [ ] Verify metadata displays correctly (vendor from Cargo.toml)
- [ ] Test multiple processors: `SignalChain![A, B, C]`
- [ ] Verify `Chain!` emits deprecation warning
- [ ] Verify bare processor error message guides to `SignalChain!`
- [ ] Verify plugin receives new VST3 class ID (expected breaking change)

### CI Validation
- `cargo xtask ci-check` must pass:
  - Linting (clippy, fmt, eslint, prettier)
  - All Rust tests (engine + macros)
  - All UI tests (Vitest)

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **VST3 ID changes break existing user plugins** | High | High | (1) Document clearly in migration guide; (2) Recommend staying on 0.7.x until major release cycle; (3) Provide rollback instructions |
| **Cargo metadata fields empty/missing** | Low | Low | Use sensible defaults ("Unknown", empty strings); VST3/CLAP don't strictly require these |
| **Users forget `SignalChain!` wrapper** | Medium | Low | Add compile-time validation with helpful error message pointing to correct syntax |
| **Email parsing from `CARGO_PKG_AUTHORS` fails** | Low | Low | Fallback to empty string; email is optional in VST3/CLAP specs |
| **`crate` property removal breaks advanced users** | Low | Medium | Make `crate` optional (not removed), defaulting to `::wavecraft`; power users can still override |
| **Breaking changes disrupt SDK adoption** | Medium | Medium | (1) Clear migration guide; (2) Version bump to 0.8.0 signals breaking change; (3) Deprecation warning for `Chain!` gives one-version grace period |

---

## Success Criteria

- [ ] `wavecraft_plugin!` only requires `name` and `signal` properties
- [ ] Plugin metadata correctly derived from Cargo environment variables
- [ ] `SignalChain!` macro works for single and multiple processors
- [ ] `Chain!` macro emits deprecation warning (still works)
- [ ] VST3 class ID generated deterministically using package name
- [ ] CLAP ID generated deterministically using package name
- [ ] CLI template uses new simplified API
- [ ] All unit tests pass (macro expansion, error messages)
- [ ] All integration tests pass (template generation)
- [ ] Manual testing checklist complete
- [ ] Documentation updated (HLD, coding standards, README)
- [ ] Migration guide created (`MIGRATION-0.8.md`)
- [ ] Clear compile-time error for bare processors without `SignalChain!`
- [ ] CI pipeline passes (`cargo xtask ci-check`)

---

## Implementation Order Summary

**Critical Path:**
1. Phase 1 (Steps 1-7) → Phase 2 (Steps 8-12) → Phase 3 (Steps 13-15) → Phase 5 (Steps 20-23)

**Parallel Workstreams:**
- Phase 2 (Steps 8-12) can start immediately (independent of Phase 1)
- Phase 4 (Steps 16-19) can run in parallel with Phase 3

**Recommended Sequence:**
1. Start with Phase 1 (core macro changes) — highest risk, enables everything else
2. Run Phase 2 (SignalChain rename) in parallel — low risk, independent
3. Phase 3 (template updates) requires Phase 1 & 2 complete
4. Phase 4 (docs) can happen anytime after Phase 1-3
5. Phase 5 (testing) runs continuously throughout all phases

**Checkpoints:**
- After Phase 1: Verify macro compiles and expands correctly
- After Phase 2: Verify `SignalChain!` and deprecated `Chain!` both work
- After Phase 3: Verify CLI generates working plugins
- After Phase 5: Full integration testing in Ableton Live

---

## Related Documents

- [User Stories](./user-stories.md) — Feature requirements
- [Low-Level Design](./low-level-design.md) — Technical specification
- [High-Level Design](../../architecture/high-level-design.md) — Overall architecture
- [Coding Standards](../../architecture/coding-standards.md) — Implementation conventions
