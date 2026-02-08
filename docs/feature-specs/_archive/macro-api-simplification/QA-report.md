# QA Report: Macro API Simplification (v0.9.0)

**Date**: 2026-02-08  
**Reviewer**: QA Agent  
**Status**: **FAIL** (2 High-severity issues require resolution)  
**Branch**: `feature/macro-api-simplification`

---

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High | 2 |
| Medium | 4 |
| Low | 3 |

**Overall**: **FAIL** — 2 High-severity issues require fixes before merge. These involve parameter sync limitations and unsafe code safety documentation.

---

## Automated Check Results

**Note:** Automated checks (linting, type-checking, tests) were run by the Tester agent via `cargo xtask ci-check` prior to QA review. Results documented in [test-plan.md](./test-plan.md).

- **Linting**: ✅ PASSED (5.1s)
- **Tests**: ✅ PASSED (10.5s)
  - Engine: 69/69 ✅
  - UI: 28/28 ✅
  - Doctests: 10/10 ✅

---

## Findings

| ID | Severity | Category | Description | Location | Recommendation |
|----|----------|----------|-------------|----------|----------------|
| 1 | High | Architectural | Parameter sync limitation - DSL always uses default params | `plugin.rs:492-506` | Document limitation prominently + add tracking issue |
| 2 | High | Real-time Safety | Unsafe pointer write missing comprehensive safety documentation | `plugin.rs:448-452` | Add detailed safety comment + bounds check verification |
| 3 | Medium | Error Handling | Mutex unwrap could panic on lock poisoning | `plugin.rs:387` | Use `lock().expect()` with descriptive message |
| 4 | Medium | Code Quality | TODO comment in production code | `plugin.rs:477` | Implement timestamp or document deferral |
| 5 | Medium | Breaking Change | VST3 Class ID generation changed - user impact | `plugin.rs:130-165` | Ensure migration guide exists |
| 6 | Medium | Metadata Parsing | Email extraction relies on specific format | `plugin.rs:194-204` | Add format validation + fallback |
| 7 | Low | Documentation | Missing migration guide reference | `lib.rs:95` | Create docs/MIGRATION-0.9.md |
| 8 | Low | Code Quality | Magic constant without explanation | `plugin.rs:475-476` | Document RMS estimation factor (0.707) |
| 9 | Low | FFI Safety | FFI exports use unwrap_or for error handling | `plugin.rs:555-557` | Document why null return is acceptable |

---

## Detailed Analysis

### HIGH SEVERITY

#### Finding 1: Parameter Sync Limitation (Architectural)

**Location**: `engine/crates/wavecraft-macros/src/plugin.rs:492-506`

**Issue**: The `build_processor_params()` method always returns default parameter values instead of syncing from nih-plug parameter state. This means DSL-generated plugins cannot access host automation values in their DSP code.

```rust
/// Build processor parameters from current nih-plug parameter values.
///
/// # Known Limitation
///
/// Currently returns default params. Full bidirectional parameter sync
/// between nih-plug and processor params is not yet implemented.
/// The DSL-generated nih-plug params work correctly for host automation
/// and UI display, but processor-level params use defaults.
fn build_processor_params(&self) -> <__ProcessorType as #krate::Processor>::Params {
    <<__ProcessorType as #krate::Processor>::Params as ::std::default::Default>::default()
}
```

**Impact**:
- DSL-generated plugins **cannot respond to parameter automation** in their DSP processing
- Host automation works (parameters are visible) but doesn't affect audio
- UI displays correct values but DSP uses defaults
- This is a fundamental limitation that users may not discover until runtime

**Evidence of Design Trade-Off**:
The low-level design document acknowledges this:

> "For custom parameter behavior, implement the `Plugin` trait directly instead of using the `wavecraft_plugin!` macro."

**Recommendation**: 
1. **Document prominently** in macro docstring (not just in a hidden method comment)
2. Add to macro-level documentation with clear guidance:
   ```rust
   /// # Known Limitations (v0.9.0)
   ///
   /// - **Parameter automation**: Processor-level params use default values.
   ///   Host automation and UI work correctly, but DSP cannot read parameter
   ///   changes. For parameter-driven DSP, implement `Plugin` trait manually.
   /// - Tracking: https://github.com/RonHouben/wavecraft/issues/XXX
   ```
3. Create GitHub issue to track proper parameter sync implementation
4. Add runtime warning during development builds (if feasible)

**Architectural Decision Required**: Should this limitation block 0.9.0 release, or is it acceptable as documented behavior?

---

#### Finding 2: Unsafe Pointer Write Safety Documentation (Real-time Safety)

**Location**: `engine/crates/wavecraft-macros/src/plugin.rs:448-452`

**Issue**: Raw pointer write in audio thread lacks comprehensive safety documentation. Current comment says "we're within bounds" but doesn't explain **why** the bounds checks above guarantee this.

```rust
if let Some(channel) = buffer.as_slice().get(ch) {
    if sample_idx < channel.len() {
        // Safety: we're within bounds
        unsafe {
            let channel_ptr = channel.as_ptr() as *mut f32;
            *channel_ptr.add(sample_idx) = sample_buf[0];
        }
    }
}
```

**Why This Is High Severity**:
- This code runs on the **audio thread** (real-time context)
- Undefined behavior here causes crashes or audio glitches
- The safety comment is insufficient for code review
- nih-plug's buffer API is complex (interleaved vs deinterleaved, shared references)

**Missing Safety Justifications**:
1. Why is casting `*const f32 → *mut f32` safe? (nih-plug buffer has shared references)
2. Why is `ptr.add(sample_idx)` safe? (already checked, but not explained)
3. Why is writing through the pointer safe? (exclusive access assumption)
4. How does this interact with nih-plug's buffer ownership model?

**Recommendation**: Replace comment with comprehensive safety documentation:

```rust
// Safety justification:
// 1. Bounds check: `sample_idx < channel.len()` ensures we're within allocation
// 2. Exclusive access: nih-plug's Buffer API guarantees no concurrent access
//    during process() execution (single-threaded audio callback)
// 3. Pointer cast: nih-plug's as_slice() returns shared refs for convenience,
//    but Buffer owns the data exclusively during process(). Cast to *mut is
//    safe because we have unique ownership during this callback.
// 4. Write safety: The write is atomic (f32 is Copy) and buffer is properly
//    aligned (allocated by nih-plug).
//
// Alternative: Use unsafe { buffer.as_slice_mut() } if nih-plug provides it.
unsafe {
    let channel_ptr = channel.as_ptr() as *mut f32;
    *channel_ptr.add(sample_idx) = sample_buf[0];
}
```

**Long-term Fix**: Investigate if nih-plug provides a mutable buffer API to avoid the unsafe cast entirely.

---

### MEDIUM SEVERITY

#### Finding 3: Mutex Unwrap Could Panic

**Location**: `engine/crates/wavecraft-macros/src/plugin.rs:387`

```rust
let meter_consumer = self.meter_consumer.lock().unwrap().take();
```

**Issue**: If the mutex is poisoned (previous thread panicked while holding lock), this `unwrap()` will panic.

**Context**: This code runs during plugin initialization, not on the audio thread, so a panic here is not a real-time safety violation. However, it's poor error handling.

**Recommendation**: Use `expect()` with a descriptive message:

```rust
let meter_consumer = self.meter_consumer
    .lock()
    .expect("meter_consumer mutex poisoned (previous panic in editor)")
    .take();
```

Or handle the error gracefully:

```rust
let meter_consumer = match self.meter_consumer.lock() {
    Ok(mut guard) => guard.take(),
    Err(_) => {
        // Mutex poisoned - create new meter channel
        let (_, consumer) = #krate::create_meter_channel(64);
        Some(consumer)
    }
};
```

---

#### Finding 4: TODO Comment in Production Code

**Location**: `engine/crates/wavecraft-macros/src/plugin.rs:477`

```rust
let frame = #krate::MeterFrame {
    peak_l: peak_left,
    peak_r: peak_right,
    rms_l: peak_left * 0.707, // Simplified RMS estimation
    rms_r: peak_right * 0.707,
    timestamp: 0, // TODO: Add proper timestamp
};
```

**Issue**: Production code contains a TODO for timestamp implementation. This means meter frames have no timing information.

**Impact**:
- Minor - UI metering likely doesn't need precise timestamps for display
- Medium - Could cause issues for time-based visualizations or synchronization

**Recommendation**: Either:
1. Implement timestamp (use `context.transport().pos_samples()` or system time)
2. Document in code comment why timestamp=0 is acceptable for DSL plugins
3. Create GitHub issue to track proper implementation

**Suggested Fix**:

```rust
let frame = #krate::MeterFrame {
    peak_l: peak_left,
    peak_r: peak_right,
    rms_l: peak_left * 0.707, // Simplified RMS estimation
    rms_r: peak_right * 0.707,
    // Note: Timestamp not implemented for DSL plugins (acceptable for basic metering).
    // For sample-accurate timing, implement Plugin trait directly.
    timestamp: 0,
};
```

---

#### Finding 5: Breaking Change - VST3 Class ID Generation

**Location**: `engine/crates/wavecraft-macros/src/plugin.rs:130-165`

**Issue**: VST3 Class IDs now use package name instead of vendor for hashing. This means existing plugins will get **new plugin IDs** when upgrading to 0.9.0.

```rust
fn generate_vst3_id(name: &str) -> proc_macro2::TokenStream {
    let package_name = env!("CARGO_PKG_NAME");  // Previously used vendor
    // ...
}
```

**Impact**:
- **High user impact**: DAWs will see plugins as "new" after update
- Users must migrate presets and project files
- This is documented in the low-level design but needs user-facing migration guide

**Recommendation**: Ensure migration guide (`docs/MIGRATION-0.9.md`) includes:
1. Warning about VST3 ID change (DAWs see as new plugin)
2. Preset migration instructions
3. Option to pin to 0.8.x if migration is not feasible
4. **Verify this file exists** before merge

---

#### Finding 6: Email Extraction Format Sensitivity

**Location**: `engine/crates/wavecraft-macros/src/plugin.rs:194-204`

```rust
let email = {
    let authors = env!("CARGO_PKG_AUTHORS");
    authors
        .split(',')
        .next()
        .map(|author| {
            // Extract email from "Name <email@example.com>" format
            author
                .split('<')
                .nth(1)
                .and_then(|s| s.split('>').next())
                .unwrap_or("")
        })
        .unwrap_or("")
};
```

**Issue**: Email extraction assumes specific format ("Name <email@example.com>"). If authors field doesn't match this format, extraction fails silently.

**Edge Cases**:
- `authors = ["name@example.com"]` (no brackets) → Returns ""
- `authors = ["Name"]` (no email) → Returns ""
- `authors = ["Name<no-space@example.com>"]` → Works (robust)
- `authors = []` (empty) → Returns ""

**Recommendation**: Add comment documenting expected format + silent fallback behavior:

```rust
let email = {
    let authors = env!("CARGO_PKG_AUTHORS");
    // Parse first author's email from "Name <email@example.com>" format.
    // Returns empty string if format doesn't match (acceptable for VST3/CLAP).
    authors
        .split(',')
        .next()
        .and_then(|author| {
            author
                .split('<')
                .nth(1)
                .and_then(|s| s.split('>').next())
        })
        .unwrap_or("")
};
```

**Alternatively**: Add format validation with clearer error:

```rust
let email = parse_author_email(env!("CARGO_PKG_AUTHORS"))
    .unwrap_or_else(|| {
        eprintln!("Warning: Could not parse email from Cargo.toml authors field");
        eprintln!("Expected format: 'Name <email@example.com>'");
        ""
    });
```

---

### LOW SEVERITY

#### Finding 7: Missing Migration Guide Reference

**Location**: `engine/crates/wavecraft-macros/src/lib.rs:95`

```rust
/// See `docs/MIGRATION-0.9.md` for migration guide.
```

**Issue**: Documentation references `docs/MIGRATION-0.9.md` but file does not exist (verified via file search).

**Recommendation**: Create migration guide or remove reference. Guide should cover:
- VST3 Class ID changes (presets, DAW projects)
- API changes (removed properties)
- SignalChain! wrapper requirement
- Deprecated Chain! macro

---

#### Finding 8: Magic Constant Without Documentation

**Location**: `engine/crates/wavecraft-macros/src/plugin.rs:475-476`

```rust
rms_l: peak_left * 0.707, // Simplified RMS estimation
rms_r: peak_right * 0.707,
```

**Issue**: The constant `0.707` (≈ 1/√2) is used for RMS estimation but lacks explanation of why this is acceptable or what trade-offs are made.

**Recommendation**: Add detailed comment:

```rust
// Simplified RMS estimation: Uses peak * 1/√2 (0.707) as approximation.
// This is exact for sine waves but approximate for other signals.
// Acceptable for basic metering; for accurate RMS, use moving average.
rms_l: peak_left * 0.707,
rms_r: peak_right * 0.707,
```

---

#### Finding 9: FFI Export Error Handling

**Location**: `engine/crates/wavecraft-macros/src/plugin.rs:555-557`

```rust
let json = #krate::__internal::serde_json::to_string(&params)
    .unwrap_or_else(|_| "[]".to_string());

::std::ffi::CString::new(json)
    .map(|s| s.into_raw())
    .unwrap_or(::std::ptr::null_mut())
```

**Issue**: Uses `unwrap_or` to return null pointer on error. While this is standard FFI practice, it lacks documentation explaining why this is safe.

**Recommendation**: Add comment:

```rust
// Serialize parameters to JSON. Returns "[]" on serialization failure
// (should never happen for well-typed ParameterInfo structs).
let json = #krate::__internal::serde_json::to_string(&params)
    .unwrap_or_else(|_| "[]".to_string());

// Convert to C string. Returns null on failure (e.g., embedded null bytes).
// Caller must check for null before dereferencing.
::std::ffi::CString::new(json)
    .map(|s| s.into_raw())
    .unwrap_or(::std::ptr::null_mut())
```

---

## Architectural Concerns

> ⚠️ **The following items require Architect review before implementation.**

### 1. Parameter Sync Limitation - Fundamental Design Issue?

**Description**: The DSL-generated plugins cannot sync parameters from nih-plug to processor params. This means automation doesn't affect DSP processing.

**Questions for Architect**:
1. Is this limitation acceptable for 0.9.0 release with prominent documentation?
2. Should we block the release until parameter sync is implemented?
3. What's the long-term plan to address this? (Proc-macro complexity vs manual Plugin impl)
4. Should we add a warning at compile-time or runtime to make users aware?

**Trade-Off**: Simplicity (DSL with 4 lines) vs Functionality (parameter automation)

---

### 2. Unsafe Pointer Write in Audio Thread

**Description**: The sample-by-sample processing loop uses unsafe pointer writes to modify nih-plug's buffer. This appears necessary due to buffer API design, but safety justification is weak.

**Questions for Architect**:
1. Is this the recommended pattern for nih-plug buffer writes?
2. Can we use a safe API instead? (e.g., `buffer.as_slice_mut()`)
3. Should we refactor to avoid sample-by-sample copies? (Performance impact)

---

### 3. VST3 Class ID Breaking Change Impact

**Description**: Using package name instead of vendor for VST3 ID generation breaks existing plugin IDs.

**Questions for Architect**:
1. Is this breaking change acceptable for 0.9.0? (User impact: preset migration)
2. Should we provide a compatibility shim for existing plugins?
3. Should this be deferred to 1.0.0 for larger breaking change budget?

---

## Code Quality Assessment

### Real-Time Safety ✅ PASS

- [x] No heap allocations on audio thread
- [x] No locks on audio thread
- [x] No syscalls on audio thread
- [x] Uses atomics for shared state (meter_producer)
- [x] Uses lock-free SPSC ring buffer for metering
- [ ] ⚠️ Unsafe pointer write needs better documentation (Finding 2)

### Domain Separation ✅ PASS

- [x] Macro crate only generates code, no runtime DSP logic
- [x] Clear boundaries between generated code and user code
- [x] Template properly uses SDK via `use wavecraft::prelude::*`

### TypeScript/React Patterns ✅ N/A

- Not applicable to this pure Rust feature

### Security & Bug Patterns ✅ PASS (with notes)

- [x] No hardcoded secrets
- [x] Input validation on macro properties (clear error messages)
- [x] Proper error handling in Parse impl
- [ ] ⚠️ Mutex unwrap could panic (Finding 3) - minor issue, not security-critical
- [x] No data races
- [ ] ⚠️ FFI exports use unwrap_or (Finding 9) - acceptable pattern, needs documentation

### Test Coverage ✅ PASS

- [x] Unit tests for SignalChain! macro (chain_macro.rs)
- [x] Doctests updated to use `text` blocks
- [x] CLI template validation (TC-001 through TC-010)
- [x] Deprecation warnings tested (TC-005)
- [x] Error messages tested (TC-006)
- [ ] ⚠️ Missing test for parameter sync limitation (acceptable - documented as known limitation)

---

## User Story Verification

### User Story 1: Minimal Plugin Definition ✅ VERIFIED

- [x] `wavecraft_plugin!` only requires `name` and `signal`
- [x] `vendor`, `url`, `email` removed
- [x] `crate` property removed from required fields (optional with default)
- [x] Minimal working example in template (4 lines)

**Evidence**: Template file (`cli/sdk-templates/new-project/react/engine/src/lib.rs`) shows:

```rust
wavecraft_plugin! {
    name: "{{plugin_name_title}}",
    signal: SignalChain![{{plugin_name_pascal}}Gain],
}
```

---

### User Story 2: Consistent Signal Chain API ✅ VERIFIED

- [x] `signal` property only accepts `SignalChain![...]` syntax
- [x] Single processors wrapped: `SignalChain![MyProcessor]`
- [x] Multiple processors: `SignalChain![A, B, C]`
- [x] `Chain!` deprecated with warning message (TC-005)
- [x] Clear error message for bare processors (TC-006)

**Evidence**: 
- `chain_macro.rs` tests verify SignalChain! for single and multiple processors
- Bare processor validation in plugin.rs (lines 89-101)

---

### User Story 3: Automatic Metadata Derivation ✅ VERIFIED

- [x] Vendor derived from `CARGO_PKG_AUTHORS` (default: "Unknown")
- [x] URL derived from `homepage` or `repository` (default: "")
- [x] Email parsed from authors field (default: "")
- [x] Version derived from `CARGO_PKG_VERSION` (already implemented)

**Evidence**: Implementation in plugin.rs (lines 177-204)

**Note**: Finding 6 documents email parsing fragility, but defaults are acceptable.

---

## Success Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Plugin definition lines | ~4 lines | 4 lines (name + signal) | ✅ ACHIEVED |
| No functionality loss | VST3/CLAP export works | All tests passing | ✅ ACHIEVED |
| Clear compile-time errors | Helpful error messages | Validated in TC-006 | ✅ ACHIEVED |
| Template uses new API | Minimal boilerplate | Template updated | ✅ ACHIEVED |

---

## Performance Considerations

### Generated Code Size

No significant change - metadata derivation happens at compile time (no runtime overhead).

### Runtime Overhead

- **Sample-by-sample processing**: Current implementation processes samples individually, which may have performance implications compared to vectorized processing. However, this is a pre-existing design trade-off for the DSL (not introduced by this change).
- **Meter frame push**: Lock-free SPSC ring buffer has minimal overhead (~few nanoseconds per sample).

---

## Backward Compatibility

### Breaking Changes ⚠️ DOCUMENTED

1. **API Changes**:
   - Removed `vendor`, `url`, `email` properties ❌ BREAKING
   - Signal must use `SignalChain!` wrapper ❌ BREAKING
   - `crate` property now has default (::wavecraft) ⚠️ BEHAVIOR CHANGE

2. **ID Generation**:
   - VST3 Class IDs use package name (not vendor) ❌ BREAKING
   - CLAP IDs use package name ❌ BREAKING
   - Existing plugins will get new IDs → DAWs see as "new" plugin

3. **Deprecated Features**:
   - `Chain!` macro deprecated (still works with warning) ✅ GRACEFUL

**Migration Path**: Requires migration guide (docs/MIGRATION-0.9.md) - **MISSING** (Finding 7)

---

## Recommendations Summary

### Must Fix (High Severity)

1. **Parameter Sync Documentation** (Finding 1):
   - Add prominent limitation documentation to macro docstring
   - Create GitHub issue to track proper parameter sync
   - Consider runtime warning in dev builds
   - **Architect decision required**: Is this acceptable for 0.9.0 release?

2. **Unsafe Code Safety Documentation** (Finding 2):
   - Add comprehensive safety comment explaining pointer write
   - Document nih-plug buffer ownership model
   - Consider investigating safe alternative API

### Should Fix (Medium Severity)

3. **Mutex Error Handling** (Finding 3):
   - Use `expect()` with descriptive message or handle gracefully

4. **TODO in Production** (Finding 4):
   - Either implement timestamp or document why it's deferred

5. **Migration Guide** (Finding 5):
   - Create `docs/MIGRATION-0.9.md` or ensure it exists before merge

6. **Email Parsing Comment** (Finding 6):
   - Document expected format and fallback behavior

### Nice to Have (Low Severity)

7. **Migration Guide Reference** (Finding 7):
   - Create file or remove reference from docstring

8. **Magic Constant Documentation** (Finding 8):
   - Add comment explaining 0.707 RMS approximation

9. **FFI Error Handling Documentation** (Finding 9):
   - Add comment explaining null return strategy

---

## Handoff Decision

**Target Agent**: **Architect** (for review before coder fixes)

**Reasoning**: 

1. **Architectural Concerns Require Design Decisions**:
   - Finding 1 (Parameter Sync) is a fundamental design limitation that affects the DSL's usefulness
   - Finding 2 (Unsafe Code) may require architectural changes to buffer processing strategy
   - Finding 5 (Breaking Changes) needs product decision: acceptable for 0.9.0 or defer to 1.0.0?

2. **Precedence**: Per agent development flow, when architectural issues are found during QA, the Architect must review before the Coder implements fixes. This ensures design trade-offs are consciously accepted.

3. **Next Steps for Architect**:
   - Review Finding 1: Decide if parameter sync limitation is acceptable for 0.9.0 (document vs block)
   - Review Finding 2: Validate unsafe code justification or propose alternative buffer API
   - Review Finding 5: Confirm VST3 Class ID breaking change is acceptable for 0.9.0
   - Update architectural documentation if design decisions change
   - Hand off to Coder with clear guidance on which findings to fix vs accept

4. **If Architect Approves Design**:
   - Hand off to **Coder** to fix Medium/Low severity findings (3-9)
   - Add prominent documentation for Finding 1 (parameter sync limitation)
   - Enhance safety comments for Finding 2 (unsafe code)

---

## Overall Assessment

**Implementation Quality**: Good - code is well-structured, tests pass, follows SDK patterns

**User Story Fulfillment**: 100% - all acceptance criteria met

**Breaking Changes**: Well-documented in low-level design, but missing user-facing migration guide

**Risk Level**: Medium-High due to parameter sync limitation and VST3 ID changes

**Recommendation**: **Requires Architect review** before proceeding to merge. The parameter sync limitation (Finding 1) is a significant design trade-off that should be explicitly approved as acceptable for 0.9.0.
