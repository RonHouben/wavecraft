# Investigation Report & Bugfix Plan: Hot-Reload Parameter Update Failure

## Related Documents

- [Low-Level Design](./low-level-design-rust-hot-reload.md) — Original hot-reload architecture
- [Implementation Progress](./implementation-progress.md) — Original implementation status
- [Development Workflows](../../architecture/development-workflows.md) — Dev workflow documentation

## Date

February 10, 2026

## Bug Summary

When `wavecraft start` detects Rust source file changes and rebuilds the plugin, the terminal shows "Build succeeded in X.Xs" but the parameter update chain silently stops. The UI never updates to reflect new/changed parameters. No error message is visible.

---

## 1. Root Cause Analysis

### Critical Path

The failure occurs in `RebuildPipeline::do_build()` at `cli/src/dev_server/rebuild.rs` lines 322–338. After successfully printing "Build succeeded", the code calls `self.load_parameters_from_dylib()` inside `std::panic::catch_unwind`. This function either **hangs** (most likely) or **fails with an error that goes to stderr only** (less likely).

### Most Likely Root Cause: `dlopen` Hang During Hot-Reload

`load_parameters_from_dylib()` calls `PluginLoader::load(&temp_path)` which calls `dlopen()` via `libloading::Library::new()`. On macOS, `dlopen` runs all module initializer functions in the loaded library. During hot-reload, **two copies of the plugin library coexist**:

1. **Original**: Loaded during initial `wavecraft start` startup via `load_parameters()` in `start.rs`. The `PluginLoader` (and thus the `Library` handle) is kept alive for the session lifetime because the audio system uses the processor vtable.

2. **Hot-reload copy**: The rebuilt dylib is copied to `/tmp/wavecraft_hotreload_{timestamp}.dylib` and loaded via a second `dlopen` call.

Potential hang causes:
- **macOS `dyld` interaction**: Two loaded instances of the same library (identical code, different paths) may cause `dyld` to block on internal locks or caches
- **Transitive static initializers**: While `_param-discovery` gates `nih_export_clap!`/`nih_export_vst3!`, nih-plug's other code and transitive dependencies may still have initializers that interact with macOS audio subsystems
- **Processor vtable construction**: `PluginLoader::load()` always calls `try_load_processor_vtable()` which invokes `wavecraft_dev_create_processor` FFI symbol — this constructs processor vtable function pointers and is unnecessary for hot-reload's parameter-only use case

### Secondary Possibility: Silent Error to stderr

The error branch in `handle_change()` uses `eprintln!` (stderr), while success messages use `println!` (stdout). If the user's terminal setup separates these streams, a "Build failed: Failed to load parameters from dylib: ..." message would appear on stderr and be missed. The `find_plugin_dylib()` or `PluginLoader::load()` error would be swallowed by this stdout/stderr split.

---

## 2. Investigation Checklist (Pre-Fix Verification)

Before implementing fixes, verify the diagnosis:

### Check 1: Combined stdout+stderr Output
```bash
wavecraft start 2>&1 | tee /tmp/wavecraft-debug.log
# Edit engine/src/lib.rs, then:
grep -i "failed\|error\|panic\|Build failed" /tmp/wavecraft-debug.log
```
**Purpose**: Confirm whether a "Build failed" error IS being printed to stderr.

### Check 2: Temp Dylib Creation
```bash
# After triggering a rebuild, check:
ls -la /tmp/wavecraft_hotreload_*
```
**Purpose**: If the file exists but wasn't cleaned up, `PluginLoader::load()` is the hang point. If no file exists, `find_plugin_dylib()` or `create_temp_dylib_copy()` failed.

### Check 3: macOS Library Loading Trace
```bash
DYLD_PRINT_LIBRARIES=1 wavecraft start 2>&1 | grep -i "hotreload\|test_hot"
```
**Purpose**: Confirm whether `dyld` successfully loads the temp dylib. If `dyld` never prints the load event, `dlopen` is blocking.

### Check 4: Manual Dylib Load
```bash
# After a rebuild, verify the dylib is loadable:
nm -gU target/tmp/test-hotreload/target/debug/libtest_hot_reload.dylib | grep wavecraft
```
**Purpose**: Verify FFI symbols exist in the rebuilt dylib.

---

## 3. Plan of Action

### Fix 1: Add Step-by-Step Diagnostics (CRITICAL — do first)

**File**: `cli/src/dev_server/rebuild.rs`
**Function**: `load_parameters_from_dylib()`

Add `println!` statements at every major step:

```rust
fn load_parameters_from_dylib(&self) -> Result<Vec<ParameterInfo>> {
    use crate::project::find_plugin_dylib;

    println!("  {} Finding plugin dylib...", style("→").dim());
    let lib_path = find_plugin_dylib(&self.engine_dir)
        .context("Failed to find plugin dylib after rebuild")?;
    println!("  {} Found: {}", style("→").dim(), lib_path.display());

    println!("  {} Copying to temp location...", style("→").dim());
    let temp_path = self.create_temp_dylib_copy(&lib_path)?;
    println!("  {} Temp: {}", style("→").dim(), temp_path.display());

    println!("  {} Loading dylib via FFI...", style("→").dim());
    let loader = PluginLoader::load(&temp_path)
        .with_context(|| format!("Failed to load dylib: {}", temp_path.display()))?;
    println!("  {} Loaded {} parameters via FFI", style("→").dim(), loader.parameters().len());

    let params = loader.parameters().to_vec();

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    Ok(params)
}
```

**Impact**: Immediately reveals WHERE the hang occurs.

### Fix 2: Unify Error Output to stdout (CRITICAL — do first)

**File**: `cli/src/dev_server/rebuild.rs`
**Function**: `handle_change()`

Change ALL `eprintln!` calls to `println!` so errors are visible alongside success messages in the same output stream. Apply to:
- Parameter replacement errors
- UI notification errors
- "Hot-reload aborted" message
- "Build failed" message

**Impact**: Errors appear in the same stream as success messages — no risk of missing them.

### Fix 3: Extract Dylib Loading to Standalone Function + Timeout (RECOMMENDED)

**File**: `cli/src/dev_server/rebuild.rs`

Make `load_parameters_from_dylib` a standalone function that takes `engine_dir` by value (no `&self` needed), so it can be moved into `spawn_blocking` with a timeout:

```rust
/// Load parameters from the rebuilt dylib (standalone, suitable for spawn_blocking).
fn load_parameters_from_dylib_standalone(engine_dir: PathBuf) -> Result<Vec<ParameterInfo>> {
    use crate::project::find_plugin_dylib;

    println!("  {} Finding plugin dylib...", style("→").dim());
    let lib_path = find_plugin_dylib(&engine_dir)
        .context("Failed to find plugin dylib after rebuild")?;

    // Copy to temp to avoid macOS caching
    let temp_path = create_temp_dylib_copy(&lib_path)?;

    println!("  {} Loading parameters via FFI...", style("→").dim());
    let loader = PluginLoader::load_params_only(&temp_path)
        .with_context(|| format!("Failed to load dylib: {}", temp_path.display()))?;

    let params = loader.to_vec();
    let _ = std::fs::remove_file(&temp_path);

    println!("  {} Extracted {} parameters", style("→").dim(), params.len());
    Ok(params)
}
```

Then in `do_build()`:
```rust
let engine_dir = self.engine_dir.clone();
let load_result = tokio::time::timeout(
    std::time::Duration::from_secs(30),
    tokio::task::spawn_blocking(move || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            load_parameters_from_dylib_standalone(engine_dir)
        }))
    }),
).await;
```

**Impact**: Clean separation, proper async design, timeout safety.

### Fix 4: Add `load_params_only()` to PluginLoader

**File**: `engine/crates/wavecraft-bridge/src/plugin_loader.rs`

Add a method that loads ONLY parameters — skipping the processor vtable:

```rust
/// Load parameters only (skip processor vtable loading).
///
/// Used by hot-reload where only parameter metadata is needed.
/// Avoids potential side effects from vtable construction.
pub fn load_params_only<P: AsRef<Path>>(dylib_path: P) -> Result<Vec<ParameterInfo>, PluginLoaderError> {
    let library = unsafe { Library::new(dylib_path.as_ref()) }
        .map_err(PluginLoaderError::LibraryLoad)?;

    let get_params_json: Symbol<GetParamsJsonFn> = unsafe {
        library.get(b"wavecraft_get_params_json\0").map_err(|e| {
            PluginLoaderError::SymbolNotFound(format!("wavecraft_get_params_json: {}", e))
        })?
    };

    let free_string: Symbol<FreeStringFn> = unsafe {
        library.get(b"wavecraft_free_string\0").map_err(|e| {
            PluginLoaderError::SymbolNotFound(format!("wavecraft_free_string: {}", e))
        })?
    };

    let params = unsafe {
        let json_ptr = get_params_json();
        if json_ptr.is_null() {
            return Err(PluginLoaderError::NullPointer("wavecraft_get_params_json"));
        }
        let c_str = CStr::from_ptr(json_ptr);
        let json_str = c_str.to_str().map_err(PluginLoaderError::InvalidUtf8)?;
        let params: Vec<ParameterInfo> =
            serde_json::from_str(json_str).map_err(PluginLoaderError::JsonParse)?;
        free_string(json_ptr);
        params
    };

    // Intentionally do NOT load processor vtable
    // Library is dropped here (dlclose)
    Ok(params)
}
```

**Impact**: Eliminates unnecessary vtable construction during hot-reload. Reduces risk of FFI side effects.

### Fix 5: Update Sidecar Cache After Hot-Reload (Nice-to-Have)

**File**: `cli/src/dev_server/rebuild.rs`
**Function**: `handle_change()` — after parameters are successfully loaded

```rust
Ok((params, param_count_change)) => {
    // Write updated sidecar cache for faster next start
    if let Err(e) = write_sidecar_cache(&self.engine_dir, &params) {
        eprintln!("  Warning: failed to update param cache: {}", e);
    }
    // ... continue with replace_parameters and broadcast
}
```

**Impact**: Next `wavecraft start` startup is faster (uses cached params).

---

## 4. Implementation Priority

| Priority | Fix | Effort | Impact |
|----------|-----|--------|--------|
| **P0** | Fix 1: Add diagnostics | 10 min | Immediately reveals the exact failure point |
| **P0** | Fix 2: Unify stdout/stderr | 5 min | Ensures errors are always visible |
| **P1** | Fix 3: Standalone function + spawn_blocking + timeout | 30 min | Proper async design + timeout recovery |
| **P1** | Fix 4: `load_params_only()` | 20 min | Eliminates vtable side effects |
| **P2** | Fix 5: Sidecar cache update | 10 min | Improved DX for restart |

### Recommended Implementation Order

1. **First**: Apply Fix 1 + Fix 2 (diagnostics + stderr fix). Deploy to reproduce and confirm the exact hang point.
2. **Then**: Apply Fix 3 + Fix 4 (standalone function + params-only loading). This is the structural fix.
3. **Finally**: Apply Fix 5 (sidecar cache update) as polish.

---

## 5. Testing

### Verification Steps

After implementing fixes:

1. **Basic hot-reload**: Run `wavecraft start`, edit `engine/src/lib.rs` to add a new `wavecraft_processor!` wrapper, verify terminal shows the full chain: "Build succeeded" → "Finding plugin dylib..." → "Loading parameters via FFI..." → "Updating parameter host..." → "UI notified" → "Hot-reload complete — N parameters (+1 new)"

2. **UI updates**: Verify the browser UI shows the new parameter after hot-reload (no page refresh needed).

3. **Timeout recovery**: If Fix 3 is applied, verify that a deliberately hanging FFI call results in a clear "Timeout loading parameters" error after 30s, and the watcher continues listening for subsequent changes.

4. **Error visibility**: Introduce a deliberate compile error in `lib.rs`, verify "Build failed" appears in the same output stream as "Build succeeded" messages.

5. **Multiple rapid saves**: Save `lib.rs` 3 times in quick succession, verify the build guard correctly queues and executes rebuilds without deadlock.

---

## 6. Future Consideration: Subprocess-Based FFI Isolation

If the `dlopen` hang persists even with `load_params_only()`, the nuclear option is to extract parameters via a **subprocess** instead of `dlopen` in the main process:

```
wavecraft start process:
  1. cargo build --lib --features _param-discovery
  2. Spawn: wavecraft-param-extract <dylib_path>  →  stdout: JSON params
  3. Parse JSON from subprocess stdout
  4. If subprocess hangs → kill after timeout
```

This completely isolates the FFI loading from the parent process. The subprocess can be a small helper binary compiled with the CLI, or a `cargo run` invocation of a simple extraction binary.

This approach is more complex but provides absolute safety against any `dlopen`-related hang.
