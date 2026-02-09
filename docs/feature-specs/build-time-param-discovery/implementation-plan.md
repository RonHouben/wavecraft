# Implementation Plan: Build-Time Parameter Discovery

## Overview

`wavecraft start` hangs at "Loading plugin parameters..." because loading the plugin dylib triggers nih-plug's VST3/CLAP static initializers (`nih_export_clap!`, `nih_export_vst3!`), which block on macOS audio subsystem services (`AudioComponentRegistrar`). The fix is to feature-gate the nih-plug export macros behind a `_param-discovery` Cargo feature. When building with this feature active, the dylib only contains param FFI functions — no plugin factory registrations, no static initializers. The CLI builds with this feature for fast param extraction, caches the result as a sidecar JSON file, and only does a full build (with nih-plug exports) if audio-dev is needed.

## Requirements

- `wavecraft start` must not hang when loading plugin parameters
- Zero user friction — no new manual steps required
- Backward-compatible with existing plugins that lack the `_param-discovery` feature
- Sidecar JSON cache to skip rebuild on subsequent runs when code hasn't changed
- Full nih-plug build still happens (in background) when `audio-dev` is needed
- Existing test suites and CI must continue to pass

## Architecture Changes

1. **`engine/crates/wavecraft-macros/src/plugin.rs`** — Wrap `nih_export_clap!` / `nih_export_vst3!` invocations in `#[cfg(not(feature = "_param-discovery"))]` guards. The `#[cfg]` attribute applies to the *user's crate* where the macro expands (not to the proc-macro crate itself).
2. **`cli/sdk-templates/new-project/react/engine/Cargo.toml.template`** — Add `[features]` section with `_param-discovery = []`.
3. **`cli/src/commands/start.rs`** — Replace the single-phase build+load with a two-phase flow: (1) build with `_param-discovery` for fast param extraction, (2) optionally build without it in the background for audio-dev.
4. **`engine/crates/wavecraft-bridge/src/plugin_loader.rs`** — Add `load_params_from_file()` static method for reading cached sidecar JSON.

## Implementation Steps

### Phase 1: Macro Feature Gate

#### Step 1.1: Wrap nih-plug export macros with `#[cfg]` guard

**File:** [engine/crates/wavecraft-macros/src/plugin.rs](engine/crates/wavecraft-macros/src/plugin.rs#L687-L688)

**Action:** Change the current unconditional export:

```rust
#krate::__nih::nih_export_clap!(__WavecraftPlugin);
#krate::__nih::nih_export_vst3!(__WavecraftPlugin);
```

to a conditionally-compiled version:

```rust
#[cfg(not(feature = "_param-discovery"))]
#krate::__nih::nih_export_clap!(__WavecraftPlugin);
#[cfg(not(feature = "_param-discovery"))]
#krate::__nih::nih_export_vst3!(__WavecraftPlugin);
```

**Why:** The `#[cfg]` guards prevent nih-plug from registering VST3/CLAP factory static initializers when the `_param-discovery` feature is active. This makes `dlopen` instant with no system-level audio init.

**Technical Note:** `nih_export_clap!` and `nih_export_vst3!` expand to `#[no_mangle] extern "C"` functions and `#[used]` static items at crate root. Putting `#[cfg]` directly before the macro invocation is valid because the attribute applies to whatever item the macro produces. However, since these are *macro invocations* (not items), the `#[cfg]` attribute on a macro invocation is evaluated by the compiler on the invocation statement itself — if the condition is false, the macro is not expanded at all. This is the desired behavior.

**Risk:** Medium — If `nih_export_*!` macros produce multiple top-level items, a single `#[cfg]` on the macro invocation may only gate the first item. Mitigation: test by inspecting the dylib symbols with `nm -g`. If this doesn't work, the fallback is wrapping in a `cfg`-gated module (see Step 1.2).

**Dependencies:** None

---

#### Step 1.2 (Contingency): Module-wrapped exports

**File:** [engine/crates/wavecraft-macros/src/plugin.rs](engine/crates/wavecraft-macros/src/plugin.rs#L687-L688)

**Action:** Only if Step 1.1's `#[cfg]` on macro invocations doesn't work, wrap in a conditional module:

```rust
#[cfg(not(feature = "_param-discovery"))]
mod __wavecraft_nih_exports {
    use super::*;
    #krate::__nih::nih_export_clap!(super::__WavecraftPlugin);
    #krate::__nih::nih_export_vst3!(super::__WavecraftPlugin);
}
```

**Why:** This guarantees the entire module (and all items it produces) is excluded. The risk is that `nih_export_*!` macros may require crate-root placement for their `#[used]` statics to work in a sub-module. This must be tested.

**Dependencies:** Only needed if Step 1.1 fails during testing.

---

### Phase 2: Template Update

#### Step 2.1: Add `_param-discovery` feature to template Cargo.toml

**File:** [cli/sdk-templates/new-project/react/engine/Cargo.toml.template](cli/sdk-templates/new-project/react/engine/Cargo.toml.template)

**Action:** Add a `[features]` section after the `[build-dependencies]` section:

```toml
[features]
default = []
_param-discovery = []   # Internal: used by `wavecraft start` for fast param loading
```

**Why:** The `_param-discovery` feature must exist on the user's plugin crate because `#[cfg(not(feature = "_param-discovery"))]` in the macro expansion is evaluated against the *expanding crate's* features. The underscore prefix signals this is an internal/SDK feature not meant for user manipulation.

**Dependencies:** None (independent of Phase 1)

---

#### Step 2.2: Update template Cargo.lock

**File:** `cli/sdk-templates/new-project/react/Cargo.lock`

**Action:** After adding the feature declaration in Step 2.1, regenerate the lock file by running `cargo generate-lockfile` or `cargo build` in a test project created from the updated template. The lock file must be updated to reflect the new feature.

**Why:** The template ships a pre-generated `Cargo.lock` for reproducible builds.

**Dependencies:** Step 2.1

---

### Phase 3: Sidecar JSON Cache in `PluginParamLoader`

#### Step 3.1: Add `load_params_from_file()` to `PluginParamLoader`

**File:** [engine/crates/wavecraft-bridge/src/plugin_loader.rs](engine/crates/wavecraft-bridge/src/plugin_loader.rs#L80)

**Action:** Add a new static method to `PluginParamLoader`:

```rust
/// Load parameters from a sidecar JSON file (bypasses FFI/dlopen).
///
/// Used by `wavecraft start` to read cached parameter metadata without
/// loading the plugin dylib (which triggers nih-plug static initializers).
pub fn load_params_from_file<P: AsRef<Path>>(
    json_path: P,
) -> Result<Vec<ParameterInfo>, PluginLoaderError> {
    let contents = std::fs::read_to_string(json_path.as_ref())
        .map_err(|e| PluginLoaderError::LibraryLoad(
            libloading::Error::DlOpen { desc: format!("Failed to read sidecar JSON: {}", e) }
        ))?;
    let params: Vec<ParameterInfo> =
        serde_json::from_str(&contents).map_err(PluginLoaderError::JsonParse)?;
    Ok(params)
}
```

**Technical Note:** Reusing `PluginLoaderError::LibraryLoad` for a file-read error is expedient but semantically imprecise. A cleaner approach is to add a `FileRead(std::io::Error)` variant to `PluginLoaderError`. The Coder should evaluate which is more appropriate — keeping the error enum small vs. accurate error reporting.

**Why:** This method gives the CLI a clean way to read cached params without touching `dlopen` at all.

**Dependencies:** None

---

#### Step 3.2: Add unit test for `load_params_from_file()`

**File:** [engine/crates/wavecraft-bridge/src/plugin_loader.rs](engine/crates/wavecraft-bridge/src/plugin_loader.rs#L197) (test module)

**Action:** Add test that writes a JSON file, reads it back with `load_params_from_file()`, and verifies the deserialized `ParameterInfo` values match.

**Why:** Ensures the sidecar path works independently of FFI.

**Dependencies:** Step 3.1

---

### Phase 4: CLI Two-Phase Build + Sidecar Cache

#### Step 4.1: Add sidecar JSON helper functions

**File:** [cli/src/commands/start.rs](cli/src/commands/start.rs)

**Action:** Add the following helper functions:

```rust
/// Path to the sidecar parameter cache file.
fn sidecar_json_path(engine_dir: &Path) -> Result<PathBuf> {
    let debug_dir = resolve_debug_dir(engine_dir)?;
    Ok(debug_dir.join("wavecraft-params.json"))
}

/// Try reading cached parameters from the sidecar JSON file.
///
/// Returns `Some(params)` if the file exists and is newer than the dylib
/// (i.e., no source changes since last extraction). Returns `None` otherwise.
fn try_read_cached_params(
    engine_dir: &Path,
    verbose: bool,
) -> Option<Vec<ParameterInfo>> {
    let sidecar_path = sidecar_json_path(engine_dir).ok()?;
    if !sidecar_path.exists() {
        return None;
    }

    // Check if sidecar is still valid (newer than any source file change)
    let dylib_path = find_plugin_dylib(engine_dir).ok()?;
    let sidecar_mtime = std::fs::metadata(&sidecar_path).ok()?.modified().ok()?;
    let dylib_mtime = std::fs::metadata(&dylib_path).ok()?.modified().ok()?;

    if dylib_mtime > sidecar_mtime {
        if verbose {
            println!("  Sidecar cache stale (dylib newer), rebuilding...");
        }
        return None;
    }

    PluginLoader::load_params_from_file(&sidecar_path).ok()
}

/// Write parameter metadata to the sidecar JSON cache.
fn write_sidecar_cache(
    engine_dir: &Path,
    params: &[ParameterInfo],
) -> Result<()> {
    let sidecar_path = sidecar_json_path(engine_dir)?;
    let json = serde_json::to_string_pretty(params)
        .context("Failed to serialize parameters")?;
    std::fs::write(&sidecar_path, json)
        .context("Failed to write sidecar cache")?;
    Ok(())
}
```

**Why:** Separates cache logic into testable units. The mtime comparison is conservative: if the dylib is newer than the sidecar, we re-extract (the dylib was rebuilt, so params may have changed).

**Dependencies:** Step 3.1 (for `load_params_from_file`)

---

#### Step 4.2: Refactor `run_dev_servers()` to use two-phase param loading

**File:** [cli/src/commands/start.rs](cli/src/commands/start.rs#L247-L292)

**Action:** Replace the current single-phase build+load flow (lines ~254–292) with the new two-phase flow:

**Current flow (to replace):**
```
1. cargo build --lib
2. find_plugin_dylib()
3. PluginLoader::load(&dylib_path) ← HANGS HERE
4. loader.parameters().to_vec()
```

**New flow:**
```
1. Try reading cached sidecar JSON → if valid, skip to step 6
2. cargo build --lib --features _param-discovery (fast, no nih-plug init)
3. find_plugin_dylib()
4. PluginLoader::load(&dylib_path) ← safe, no static initializers
5. Write sidecar JSON cache
6. Use params for DevServerHost
7. (If audio-dev enabled) cargo build --lib in background (full build)
8. Start WebSocket server + UI server
9. (When full build completes) dlopen for audio-dev vtable
```

**Detailed pseudocode for the replacement block:**
```rust
// --- Parameter discovery (new two-phase approach) ---
let (params, loader) = load_parameters(&project.engine_dir, verbose)?;

// --- Audio-dev build (background, if needed) ---
#[cfg(feature = "audio-dev")]
let audio_build_handle = {
    let engine_dir = project.engine_dir.clone();
    Some(std::thread::spawn(move || {
        Command::new("cargo")
            .args(["build", "--lib"])
            .current_dir(&engine_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    }))
};
```

The `load_parameters()` function (new, see Step 4.3) encapsulates the two-phase logic.

**Why:** Decouples fast param extraction from the full nih-plug build. Eliminates the hang.

**Dependencies:** Steps 4.1, 4.3

---

#### Step 4.3: Implement the `load_parameters()` function

**File:** [cli/src/commands/start.rs](cli/src/commands/start.rs)

**Action:** Add the core two-phase param loading function:

```rust
/// Load plugin parameters using cached sidecar or feature-gated build.
///
/// Attempts in order:
/// 1. Read cached `wavecraft-params.json` (instant, no build)
/// 2. Build with `_param-discovery` feature (no nih-plug static init)
/// 3. Fall back to normal build + FFI load (for older plugins)
fn load_parameters(
    engine_dir: &Path,
    verbose: bool,
) -> Result<(Vec<ParameterInfo>, Option<PluginLoader>)> {
    // 1. Try cached sidecar
    if let Some(params) = try_read_cached_params(engine_dir, verbose) {
        println!("{} Loaded {} parameters (cached)", style("✓").green(), params.len());
        return Ok((params, None));
    }

    // 2. Build with _param-discovery feature (skip nih-plug exports)
    println!("{} Building for parameter discovery...", style("→").cyan());
    let build_result = Command::new("cargo")
        .args(["build", "--lib", "--features", "_param-discovery"])
        .current_dir(engine_dir)
        .stdout(if verbose { Stdio::inherit() } else { Stdio::null() })
        .stderr(Stdio::inherit())
        .status();

    match build_result {
        Ok(status) if status.success() => {
            // Discovery build succeeded — load params from safe dylib
            let dylib_path = find_plugin_dylib(engine_dir)
                .context("Failed to find plugin library after discovery build")?;

            if verbose {
                println!("  Found dylib: {}", dylib_path.display());
            }

            println!("{} Loading plugin parameters...", style("→").cyan());
            let loader = PluginLoader::load(&dylib_path)
                .context("Failed to load plugin for parameter discovery")?;
            let params = loader.parameters().to_vec();

            // Write sidecar cache for next run
            if let Err(e) = write_sidecar_cache(engine_dir, &params) {
                if verbose {
                    println!("  Warning: failed to write param cache: {}", e);
                }
            }

            println!("{} Loaded {} parameters", style("✓").green(), params.len());
            Ok((params, Some(loader)))
        }
        _ => {
            // 3. Fallback: normal build (for older plugins without _param-discovery)
            if verbose {
                println!("  Discovery build failed, falling back to standard build...");
            }
            println!("{} Building plugin...", style("→").cyan());
            let fallback_status = Command::new("cargo")
                .args(["build", "--lib"])
                .current_dir(engine_dir)
                .stdout(if verbose { Stdio::inherit() } else { Stdio::null() })
                .stderr(Stdio::inherit())
                .status()
                .context("Failed to run cargo build")?;

            if !fallback_status.success() {
                anyhow::bail!("Plugin build failed. Please fix the errors above.");
            }

            let dylib_path = find_plugin_dylib(engine_dir)?;
            println!("{} Loading plugin parameters...", style("→").cyan());
            let loader = PluginLoader::load(&dylib_path)
                .context("Failed to load plugin")?;
            let params = loader.parameters().to_vec();
            println!("{} Loaded {} parameters", style("✓").green(), params.len());
            Ok((params, Some(loader)))
        }
    }
}
```

**Why:** This function encapsulates the complete param loading strategy with cache → fast build → fallback. The fallback ensures backward compatibility with plugins that don't have the `_param-discovery` feature.

**Dependencies:** Steps 4.1, 4.2

---

#### Step 4.4: Update audio-dev vtable loading for deferred build

**File:** [cli/src/commands/start.rs](cli/src/commands/start.rs#L305-L330)

**Action:** When `load_parameters()` used the `_param-discovery` feature build, the loaded dylib does NOT have `nih_export_*` symbols, but it DOES still have `wavecraft_dev_create_processor`. So the audio-dev vtable should still be available from the `_param-discovery` build (since `wavecraft_dev_create_processor` is NOT gated by the feature).

However, to be safe for the full audio pipeline (which may depend on plugin initialization), a full rebuild in the background is advisable. The current `try_start_audio_in_process()` function already handles the vtable being absent gracefully (`None` fallback).

**Approach:** If `load_parameters()` returned `Some(loader)`, pass it to `try_start_audio_in_process()` as today. The `wavecraft_dev_create_processor` FFI export is NOT behind the `_param-discovery` `#[cfg]` gate, so it remains available. No code change needed for audio-dev vtable loading — it continues to work from the discovery-built dylib.

**If the discovery build's dylib doesn't include the vtable** (e.g., due to linking differences), the fallback is already handled by existing code in `try_start_audio_in_process()`.

**Why:** Minimizes changes to the audio pipeline path.

**Dependencies:** Step 4.3

---

#### Step 4.5: Add `use` imports for new types

**File:** [cli/src/commands/start.rs](cli/src/commands/start.rs#L1-L24)

**Action:** Ensure the following are imported:

```rust
use wavecraft_protocol::ParameterInfo;  // For sidecar read/write
```

The `serde_json` import for writing the sidecar is already available since `serde_json` is in `cli/Cargo.toml` dependencies.

**Dependencies:** Phase 4 steps

---

### Phase 5: Invalidate Sidecar on Source Changes

#### Step 5.1: Conservative cache invalidation via dylib mtime

**File:** Already implemented in Step 4.1 (`try_read_cached_params`)

**Action:** The mtime comparison between sidecar and dylib handles the common case: after a `cargo build` (which updates the dylib), the dylib's mtime will be newer than the sidecar's, triggering a re-extraction.

**Edge case:** If the user runs `cargo build` independently (outside `wavecraft start`), the dylib is rebuilt but the sidecar is stale. The mtime check catches this.

**Edge case:** If the user modifies source but doesn't build, the sidecar appears valid (it's newer than the old dylib). This is acceptable because `wavecraft start` always builds before loading, so the new build will produce a newer dylib.

**Why:** Simple, reliable, no extra file watching needed.

**Dependencies:** Step 4.1

---

### Phase 6: Testing

#### Step 6.1: Verify `_param-discovery` feature gate works (symbol check)

**Action:** Build a test plugin (from template) with `--features _param-discovery` and verify:
- `nm -g <dylib> | grep clap_entry` → NOT found
- `nm -g <dylib> | grep GetPluginFactory` → NOT found (VST3)
- `nm -g <dylib> | grep wavecraft_get_params_json` → FOUND
- `nm -g <dylib> | grep wavecraft_free_string` → FOUND

Then build without the feature and verify ALL symbols are present.

**Why:** Confirms the `#[cfg]` guard actually removes nih-plug static initializers.

**Dependencies:** Steps 1.1, 2.1

---

#### Step 6.2: Integration test — `wavecraft start` loads params without hang

**Action:** Create or update a test plugin, run `wavecraft start` and verify:
- Parameters load within a reasonable timeout (< 5 seconds)
- The sidecar JSON file is written to `target/debug/wavecraft-params.json`
- Subsequent runs read from cache (check for "cached" in output)

**Dependencies:** Steps 4.1–4.5

---

#### Step 6.3: Backward compatibility test — fallback for old plugins

**Action:** Test with a plugin that does NOT have the `_param-discovery` feature in its `Cargo.toml`. Verify:
- The discovery build fails (feature not found)
- The fallback triggers a normal `cargo build --lib`
- Parameters load (may hang on macOS — this is the pre-fix behavior, acceptable for old plugins)

**Dependencies:** Step 4.3

---

#### Step 6.4: Cache invalidation test

**Action:** 
1. Run `wavecraft start` → verify sidecar is created
2. Stop the server
3. Modify a processor's `ProcessorParams` (e.g., add a parameter)
4. Run `wavecraft start` again → verify the sidecar is regenerated with the new parameter

**Dependencies:** Steps 4.1, 5.1

---

#### Step 6.5: Template validation test

**Action:** Run the standard template validation workflow:
```bash
cargo run --manifest-path cli/Cargo.toml -- create TestPlugin \
  --output target/tmp/test-plugin
cd target/tmp/test-plugin/engine
cargo clippy --all-targets -- -D warnings
cargo build --lib --features _param-discovery
nm -g target/debug/lib*.dylib | grep -c "clap_entry\|GetPluginFactory"  # should be 0
cargo build --lib
nm -g target/debug/lib*.dylib | grep -c "clap_entry\|GetPluginFactory"  # should be > 0
cd ../../../..
rm -rf target/tmp/test-plugin
```

**Why:** Ensures the template correctly includes the feature and generates working code.

**Dependencies:** Steps 2.1, 2.2

---

#### Step 6.6: Run `cargo xtask ci-check`

**Action:** Run the full CI check suite to verify no regressions:
```bash
cargo xtask ci-check
```

**Dependencies:** All previous steps

---

## Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| `#[cfg]` on `nih_export_*!` macro invocations doesn't gate all generated items | High | Test with `nm -g` on the dylib. If it fails, use the module-wrapping approach (Step 1.2). |
| Feature switching causes full recompilation of cdylib | Medium | Only the final link step re-runs; all dependency crates are cached. Measured overhead: ~0.5–2s. |
| Sidecar cache returns stale params after code changes | Low | Conservative mtime comparison: sidecar must be newer than dylib. Always rebuild before loading. |
| `nih_export_*!` macros require crate-root scope for their `#[used]` statics | Medium | Test in sub-module first. Macro invocations typically expand to items that work in any scope, but `#[used]` statics may need crate root for LTO. Verify with both debug and release builds. |
| Template `Cargo.lock` becomes out of sync after feature addition | Low | Regenerate lock file as part of Step 2.2. Include in template validation CI. |
| Older plugins without `_param-discovery` feature fail on `wavecraft start` | Low | Fallback to current behavior (standard build + FFI load) is built-in. |

## Success Criteria

- [ ] `wavecraft start` loads parameters within 5 seconds (no hang)
- [ ] Sidecar JSON file is created at `target/debug/wavecraft-params.json`
- [ ] Subsequent runs with no code changes read from cache (< 1 second)
- [ ] Plugins built without `_param-discovery` fall back to current FFI loading
- [ ] `nm -g` on discovery-built dylib shows NO `clap_entry` / `GetPluginFactory` symbols
- [ ] `nm -g` on normally-built dylib shows ALL expected symbols
- [ ] `cargo xtask ci-check` passes
- [ ] Template validation (create → clippy → build) passes
- [ ] Audio-dev still works (vtable loading from discovery build or full build)

## Documentation References

- [Low-Level Design](./low-level-design-build-time-param-discovery.md) — Full design analysis
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
- [SDK Architecture](../../architecture/sdk-architecture.md) — Crate structure and distribution
- [Development Workflows](../../architecture/development-workflows.md) — Build system and dev mode
- [Declarative Plugin DSL](../../architecture/declarative-plugin-dsl.md) — Macro system
