# Low-Level Design: Macro API Simplification

**Status:** Draft  
**Created:** 2026-02-08  
**Author:** Architect Agent  
**Target Version:** 0.8.0

---

## Executive Summary

This design simplifies the `wavecraft_plugin!` macro API from 5 required properties to just 2 (`name` and `signal`), reducing plugin definitions from ~9 lines to ~4 lines while maintaining full VST3/CLAP functionality. Plugin metadata (vendor, URL, email) will be automatically derived from Cargo environment variables, and the signal processing chain will use a consistent `SignalChain!` syntax.

**Before:**
```rust
wavecraft_plugin! {
    name: "My Plugin",
    vendor: "Wavecraft",
    url: "https://example.com",
    email: "info@example.com",
    signal: InputGain,
    crate: wavecraft,
}
```

**After:**
```rust
wavecraft_plugin! {
    name: "My Plugin",
    signal: SignalChain![MyProcessor],
}
```

---

## Problem Analysis

### Current Pain Points

1. **Repetitive metadata duplication** — `vendor`, `url`, `email` duplicate information already in `Cargo.toml`
2. **Inconsistent signal syntax** — Users can pass bare processors OR `Chain![...]`, creating confusion
3. **Exposed implementation details** — The `crate` property was never meant to be user-facing
4. **Unnecessary boilerplate** — Forces users to think about metadata when they just want to build DSP

### Why This Change Now

The macro was initially designed conservatively to avoid "magic" behavior. After real-world usage and feedback, it's clear that:

- Cargo.toml is the canonical source for project metadata
- Developers expect minimal boilerplate
- The dual syntax (bare vs Chain) causes confusion
- The `crate` property leaks implementation details

---

## Design Decisions

### 1. Metadata Derivation from Cargo Environment Variables

**Decision:** Use Cargo's compile-time environment variables instead of reading `Cargo.toml`.

**Rationale:**
- Proc-macros execute during compilation and cannot perform I/O (including reading files)
- Cargo automatically exposes package metadata as environment variables at compile time
- This is the same mechanism used for version (`env!("CARGO_PKG_VERSION")`)
- Zero runtime overhead — all values are compile-time constants

**Available Cargo Environment Variables:**

| Variable | Source | Default Fallback | Usage |
|----------|--------|------------------|-------|
| `CARGO_PKG_AUTHORS` | `Cargo.toml` `authors` field | `"Unknown"` | Plugin vendor |
| `CARGO_PKG_HOMEPAGE` | `Cargo.toml` `homepage` field | `""` | Plugin URL |
| `CARGO_PKG_REPOSITORY` | `Cargo.toml` `repository` field | Fallback if `homepage` empty | Plugin URL |
| `CARGO_PKG_NAME` | `Cargo.toml` `package.name` | _(always present)_ | VST3 ID generation |

**Implementation:**

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

// Email is not exposed by Cargo, but can be parsed from CARGO_PKG_AUTHORS
// Format: "Name <email@example.com>"
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

**Why not build.rs?**
- Adds unnecessary complexity (requires separate build script file)
- Proc-macros already run at compile time — no need for additional build step
- Cargo env vars are the idiomatic Rust solution (used by clap, serde, and other major crates)

---

### 2. VST3 Class ID Generation Strategy

**Current Implementation:**
```rust
fn generate_vst3_id(name: &str, vendor: &str) -> [u8; 16] {
    let mut hasher = DefaultHasher::new();
    format!("{}{}", vendor, name).hash(&mut hasher);
    // ... convert to bytes
}
```

**Problem:** If we remove `vendor` from the macro API, the hash input changes.

**Solution:** Use `CARGO_PKG_NAME` instead of vendor for deterministic ID generation.

**Rationale:**
1. **Stability** — Package name is the canonical, stable identifier (more stable than vendor)
2. **Determinism** — Same package name + plugin name = same ID across builds
3. **Uniqueness** — Cargo package names are globally unique (enforced by crates.io convention)
4. **Collision resistance** — Hash still provides 128-bit space
5. **Better semantics** — "my-plugin-cool-effect" is more descriptive than "Wavecraft Cool Effect"

**New Implementation:**

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

**Migration Note:** This is a **breaking change** — existing plugins will get new VST3 IDs when they upgrade to 0.8.0. DAWs will see them as new plugins, requiring preset migration. This is acceptable for a major/minor version bump and aligns with semantic versioning.

**CLAP ID:** Also derived from package name for consistency:
```rust
const CLAP_ID: &'static str = concat!("com.", env!("CARGO_PKG_NAME"));
```

---

### 3. Signal Chain Type Enforcement

**Current Behavior:**
- `signal` property accepts any expression
- Can be a bare processor: `signal: MyGain`
- Can be `Chain![]`: `signal: Chain![MyGain, Filter]`
- No compile-time enforcement of `SignalChain!` vs bare processor

**Proposed Change:**

1. **Rename `Chain!` → `SignalChain!`** for semantic clarity
2. **Require `SignalChain!` wrapper** for all signal definitions (even single processors)
3. **Compile-time trait bounds** ensure the resulting type implements `Processor`
4. **Clear error messages** guide users toward correct usage

**Why Rename?**
- **Clarity:** `SignalChain![...]` is more descriptive than `Chain![...]`
- **Consistency:** Aligns with `wavecraft_plugin!` naming convention
- **Discoverability:** `SignalChain` is easier to search/autocomplete
- **Domain-specific:** "Chain" is too generic; "SignalChain" is clearly audio DSP

**Implementation:**

```rust
// wavecraft-dsp/src/combinators/mod.rs

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

/// Combines processors into a serial signal chain.
///
/// Single processor optimization:
/// ```rust,no_run
/// use wavecraft_dsp::SignalChain;
///
/// type Single = SignalChain![GainDsp]; // Zero overhead
/// ```
///
/// Multiple processors:
/// ```rust,no_run
/// type Multiple = SignalChain![GainDsp, PassthroughDsp];
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

**Error Handling:**

If user passes a bare processor without `SignalChain!`, the existing trait bounds will catch it:

```rust
// Current trait validation in wavecraft_plugin! macro:
const _: () = {
    fn assert_processor_traits<T>()
    where
        T: Processor + Default + Send + 'static,
        T::Params: ProcessorParams + Default + Send + Sync + 'static,
    {
    }

    fn validate() {
        assert_processor_traits::<__ProcessorType>();
    }
};
```

If the user forgets `SignalChain!`, they'll get a clear error:

```
error[E0277]: the trait bound `ident: Processor` is not satisfied
  --> src/lib.rs:10:12
   |
10 |     signal: MyGain,
   |             ^^^^^^ the trait `Processor` is not implemented for `ident`
   |
   = help: did you forget to wrap this in `SignalChain![...]`?
   = note: use `SignalChain![MyGain]` for single processors
```

We should enhance the error message with a custom compile-time check:

```rust
// Add to wavecraft_plugin_impl():
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

---

### 4. The `crate` Property: Purpose & Removal

**Original Purpose:**

The `crate` property allows users to specify the path to the `wavecraft-nih_plug` integration crate:

```rust
wavecraft_plugin! {
    // ...
    crate: wavecraft,  // When using Cargo rename
}
```

This was necessary because:
1. `wavecraft-nih_plug` cannot be published to crates.io (unpublished nih_plug dependency)
2. Users depend on it via git with Cargo rename: `wavecraft = { package = "wavecraft-nih_plug", git = "..." }`
3. The macro generates code referencing types from this crate

**Why It's Exposed:**

The macro needs to reference:
- `wavecraft::Processor` trait
- `wavecraft::ProcessorParams` trait
- `wavecraft::__nih::Plugin` trait
- `wavecraft::__nih::nih_export_vst3!()` macro
- etc.

The `crate` property allows users to tell the macro where to find these types when using a custom crate path.

**Current Template Usage:**

```rust
// cli/sdk-templates/new-project/react/engine/Cargo.toml
[dependencies]
wavecraft = { package = "wavecraft-nih_plug", git = "..." }

// cli/sdk-templates/new-project/react/engine/src/lib.rs
wavecraft_plugin! {
    // ...
    crate: wavecraft,
}
```

**Proposed Change:**

**Make `crate` optional with sensible default:**

```rust
impl Parse for PluginDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // ...
        Ok(PluginDef {
            // ...
            krate: krate.or_else(|| Some(syn::parse_quote!(::wavecraft))),
        })
    }
}
```

**Rationale:**
1. **Convention over configuration** — The CLI template standardizes on `wavecraft` rename
2. **Leaky abstraction** — Users shouldn't need to think about crate paths
3. **Advanced override** — Power users can still specify `crate:` if needed (not removed, just optional)
4. **Default works 99% of the time** — The template generates `use wavecraft::prelude::*;`

**Migration:**

The `crate` property becomes optional, so existing code continues to work:

```rust
// Still valid (for advanced use cases):
wavecraft_plugin! {
    name: "My Plugin",
    signal: SignalChain![MyGain],
    crate: some_custom_path,
}

// But no longer required:
wavecraft_plugin! {
    name: "My Plugin",
    signal: SignalChain![MyGain],
}
```

---

## Implementation Plan

### Phase 1: Macro Simplification (Core Changes)

**File: `engine/crates/wavecraft-macros/src/plugin.rs`**

1. **Update `PluginDef` struct:**
   ```rust
   struct PluginDef {
       name: LitStr,
       signal: Expr,
       krate: Option<Path>,  // Optional, defaults to ::wavecraft
       // Removed: vendor, url, email
   }
   ```

2. **Update `Parse` implementation:**
   - Remove vendor/url/email parsing
   - Make `crate` optional with default
   - Update error messages

3. **Update `wavecraft_plugin_impl()`:**
   - Derive vendor from `env!("CARGO_PKG_AUTHORS")`
   - Derive URL from `env!("CARGO_PKG_HOMEPAGE")` or `env!("CARGO_PKG_REPOSITORY")`
   - Parse email from authors field
   - Default `krate` to `::wavecraft` if not specified

4. **Update `generate_vst3_id()`:**
   ```rust
   fn generate_vst3_id(name: &str) -> [u8; 16] {
       let package_name = env!("CARGO_PKG_NAME");
       // ... hash package_name + name
   }
   ```

5. **Add signal validation:**
   ```rust
   // Reject bare identifiers, require macro syntax
   if let Expr::Path(_) = signal_type {
       return compile_error("signal must use SignalChain![...]");
   }
   ```

**File: `engine/crates/wavecraft-macros/src/lib.rs`**

6. **Update docstring examples:**
   ```rust
   /// # Example
   ///
   /// ```rust,no_run
   /// wavecraft_plugin! {
   ///     name: "My Plugin",
   ///     signal: SignalChain![MyProcessor],
   /// }
   /// ```
   ```

### Phase 2: `SignalChain!` Macro Rename

**File: `engine/crates/wavecraft-dsp/src/combinators/mod.rs`**

7. **Create `SignalChain!` macro:**
   - Copy implementation from `Chain!`
   - Update docstrings

8. **Deprecate `Chain!` macro:**
   ```rust
   #[deprecated(since = "0.8.0", note = "use `SignalChain!` instead")]
   #[macro_export]
   macro_rules! Chain {
       ($($tt:tt)*) => {
           $crate::SignalChain![$($tt)*]
       };
   }
   ```

**File: `engine/crates/wavecraft-dsp/src/lib.rs`**

9. **Export `SignalChain!`:**
   ```rust
   pub use combinators::{Chain, SignalChain};
   ```

**File: `engine/crates/wavecraft-core/src/prelude.rs`**

10. **Update prelude:**
    ```rust
    pub use wavecraft_dsp::{Chain, SignalChain, /* ... */};
    ```

**File: `engine/crates/wavecraft-nih_plug/src/prelude.rs`**

11. **Update nih_plug prelude:**
    ```rust
    pub use wavecraft_dsp::{Chain, SignalChain, /* ... */};
    ```

### Phase 3: Template Updates

**File: `cli/sdk-templates/new-project/react/engine/src/lib.rs`**

12. **Update template to use new API:**
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

**File: `cli/sdk-templates/new-project/react/engine/Cargo.toml.template`**

13. **Update Cargo.toml to include metadata:**
    ```toml
    [package]
    name = "{{plugin_name}}"
    version = "0.1.0"
    edition = "2021"
    authors = ["{{vendor}}"]
    homepage = "{{url}}"
    
    [dependencies]
    wavecraft = { package = "wavecraft-nih_plug", git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
    ```

**File: `cli/src/template/variables.rs`**

14. **Remove vendor/url/email from template substitution** (now handled by Cargo.toml)

### Phase 4: Documentation Updates

**File: `docs/architecture/high-level-design.md`**

15. **Update DSL examples:**
    - Replace `Chain!` with `SignalChain!`
    - Show minimal macro API
    - Update declarative plugin DSL section

**File: `docs/architecture/coding-standards.md`**

16. **Update macro usage guidelines:**
    - Document new minimal API
    - Explain Cargo.toml metadata derivation
    - Show `SignalChain!` usage

**File: `README.md`**

17. **Update getting started examples:**
    ```rust
    wavecraft_plugin! {
        name: "My First Plugin",
        signal: SignalChain![MyGain],
    }
    ```

### Phase 5: Migration Guide

**File: `docs/MIGRATION-0.8.md`** (new)

18. **Create migration guide:**
    ```markdown
    # Migrating to Wavecraft 0.8.0
    
    ## Breaking Changes
    
    ### 1. Simplified `wavecraft_plugin!` Macro
    
    **Before:**
    ```rust
    wavecraft_plugin! {
        name: "My Plugin",
        vendor: "Wavecraft",
        url: "https://example.com",
        email: "info@example.com",
        signal: MyGain,
        crate: wavecraft,
    }
    ```
    
    **After:**
    ```rust
    wavecraft_plugin! {
        name: "My Plugin",
        signal: SignalChain![MyGain],
    }
    ```
    
    **Required Changes:**
    1. Remove `vendor`, `url`, `email` properties
    2. Move metadata to `Cargo.toml`:
       ```toml
       [package]
       authors = ["Your Name <you@example.com>"]
       homepage = "https://example.com"
       ```
    3. Wrap signal in `SignalChain![...]`
    4. Remove `crate:` property (optional, defaults to `::wavecraft`)
    
    ### 2. `Chain!` Renamed to `SignalChain!`
    
    Update all usages:
    ```rust
    // Before
    type MyChain = Chain![A, B, C];
    
    // After
    type MyChain = SignalChain![A, B, C];
    ```
    
    **Note:** `Chain!` is deprecated but still works (emits warning).
    
    ### 3. VST3 Class ID Changes
    
    **Impact:** Your plugin will receive a new VST3 class ID.
    
    **Why:** Class IDs are now based on package name instead of vendor name
    for better stability and uniqueness.
    
    **Action Required:**
    - DAWs will see this as a new plugin
    - Users may need to migrate presets manually
    - Consider releasing as a new major version if you have existing users
    
    **Mitigation:** For commercial plugins with existing users, consider
    staying on 0.7.x until your next major release cycle.
    ```

---

## API Before & After Comparison

| Aspect | Before (0.7.x) | After (0.8.0) |
|--------|----------------|---------------|
| **Required Properties** | `name`, `vendor`, `signal` | `name`, `signal` |
| **Optional Properties** | `url`, `email`, `crate` | `crate` |
| **Signal Syntax** | Bare processor OR `Chain![]` | `SignalChain![]` only |
| **Metadata Source** | Hardcoded in macro | Derived from `Cargo.toml` |
| **VST3 ID Generation** | Hash of vendor + name | Hash of package name + name |
| **Lines of Code** | ~9 lines | ~4 lines |
| **`Chain!` Macro** | Primary API | Deprecated (use `SignalChain!`) |

---

## Compile-Time Error Improvements

### Before:
```rust
wavecraft_plugin! {
    name: "My Plugin",
    // Missing vendor → unhelpful error
}
```

```
error: missing required field: `vendor`
```

### After:
```rust
wavecraft_plugin! {
    name: "My Plugin",
    signal: MyGain,  // Forgot SignalChain!
}
```

```
error: signal property must use `SignalChain!` wrapper

Did you mean:
    signal: SignalChain![MyGain]

Or for multiple processors:
    signal: SignalChain![A, B, C]
```

---

## Testing Strategy

### Unit Tests

**File: `engine/crates/wavecraft-macros/tests/plugin_macro.rs`** (new)

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
}

#[test]
fn bare_processor_without_signal_chain_errors() {
    let input = quote! {
        wavecraft_plugin! {
            name: "Test",
            signal: MyProcessor,  // Missing SignalChain!
        }
    };
    // Verify compile error with helpful message
}

#[test]
fn vst3_id_uses_package_name() {
    // Verify generate_vst3_id() uses CARGO_PKG_NAME
}

#[test]
fn crate_property_defaults_to_wavecraft() {
    // Verify missing crate: property defaults correctly
}
```

### Integration Tests

**File: `cli/tests/template_generation.rs`** (update)

```rust
#[test]
fn generated_plugin_uses_new_macro_api() {
    // wavecraft create test-plugin
    // Verify generated lib.rs uses SignalChain!
    // Verify no vendor/url/email in macro invocation
}

#[test]
fn generated_cargo_toml_includes_metadata() {
    // Verify authors, homepage present in generated Cargo.toml
}
```

### Manual Testing Checklist

- [ ] Generate new plugin with `wavecraft create`
- [ ] Build plugin with `cargo xtask bundle`
- [ ] Load plugin in Ableton Live
- [ ] Verify metadata appears correctly in DAW (vendor, URL)
- [ ] Verify plugin receives new VST3 ID (expected behavior)
- [ ] Test with multiple processors: `SignalChain![A, B, C]`
- [ ] Verify `Chain!` macro emits deprecation warning
- [ ] Verify `SignalChain!` works correctly

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **VST3 ID change breaks existing plugins** | High | High | Document clearly, provide migration guide, consider it a breaking change |
| **Cargo metadata missing/empty** | Low | Low | Fallback to sensible defaults ("Unknown", "") |
| **Users forget `SignalChain!` wrapper** | Medium | Low | Clear compile-time error message guides them |
| **`CARGO_PKG_AUTHORS` format varies** | Low | Low | Parse robustly, handle multiple formats |
| **Email parsing from authors field fails** | Low | Low | Fallback to empty string (email is optional in VST3/CLAP) |

---

## Open Questions

### Q1: Should we support override properties for advanced users?

**Proposal:** Allow explicit overrides for metadata:

```rust
wavecraft_plugin! {
    name: "My Plugin",
    signal: SignalChain![MyProcessor],
    vendor: "Custom Vendor",  // Override Cargo.toml
}
```

**Decision:** **No.** Keep the API minimal. If users need custom metadata, they can:
1. Update `Cargo.toml` (preferred)
2. Implement `Plugin` trait manually (advanced users)

**Rationale:** Adding overrides defeats the purpose of simplification. Cargo.toml is the source of truth.

### Q2: Should `Chain!` emit a compile error instead of deprecation warning?

**Proposal:** Make `Chain!` a hard error immediately in 0.8.0.

**Decision:** **No.** Use deprecation warning in 0.8.0, remove in 0.9.0.

**Rationale:**
- Gives users one version to migrate gracefully
- Deprecation warnings are less disruptive than compile errors
- Standard Rust practice (warn → error → remove)

### Q3: Should we hash email into VST3 ID for additional uniqueness?

**Proposal:** Include parsed email in VST3 ID hash.

**Decision:** **No.** Use package name only.

**Rationale:**
- Package name is sufficient (globally unique on crates.io)
- Email can be missing or change over time
- Simpler = better

---

## Success Criteria

- [ ] `wavecraft_plugin!` only requires `name` and `signal`
- [ ] Plugin metadata correctly derived from `Cargo.toml` environment variables
- [ ] `SignalChain!` macro works for single and multiple processors
- [ ] `Chain!` macro emits deprecation warning
- [ ] VST3 class ID deterministic and uses package name
- [ ] CLAP ID deterministic and uses package name
- [ ] CLI template uses new simplified API
- [ ] All tests pass (unit + integration + manual)
- [ ] Documentation updated (HLD, coding standards, README)
- [ ] Migration guide created (`MIGRATION-0.8.md`)
- [ ] Clear compile-time errors for common mistakes

---

## Related Documents

- [User Stories](./user-stories.md) — Feature requirements
- [High-Level Design](../../architecture/high-level-design.md) — Overall architecture
- [Coding Standards](../../architecture/coding-standards.md) — Macro conventions
- [Roadmap](../../roadmap.md) — Version tracking

