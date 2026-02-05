# Test Plan: CLI `--local-dev` Flag

## Overview
- **Feature**: CLI `--local-dev` flag for local SDK development
- **Spec Location**: `docs/feature-specs/ci-local-dev-dependencies/`
- **Date**: 2026-02-04
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 10 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] `cargo xtask check` passes (all lint + tests)
- [x] CLI compiles without errors
- [x] Unit tests pass (10/10)

## Test Cases

### TC-001: Help text shows --local-dev flag

**Description**: Verify the `--local-dev` flag appears in help output with correct description

**Preconditions**:
- CLI is built

**Steps**:
1. Run `wavecraft new --help`
2. Check for `--local-dev` in output
3. Verify description mentions "local SDK path"

**Expected Result**: Flag is documented with clear description

**Status**: ✅ PASS

**Actual Result**: Help output shows:
```
--local-dev <LOCAL_DEV>      Use local SDK path for development (path to engine/crates directory). 
                             When provided, generates path dependencies instead of git tag dependencies. 
                             Mutually exclusive with a custom --sdk-version
```

**Notes**: Description is clear and mentions mutual exclusivity with --sdk-version

---

### TC-002: Standard mode generates git dependencies (default)

**Description**: Without `--local-dev`, generated project uses git tag dependencies

**Preconditions**:
- CLI is built
- Output directory doesn't exist

**Steps**:
1. Run `wavecraft new test-standard --vendor "Test" --no-git` in temp directory
2. Check `engine/Cargo.toml` for dependency format
3. Verify dependencies use `git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0"`

**Expected Result**: All 5 SDK crates use git tag dependencies

**Status**: ✅ PASS

**Actual Result**: All 5 crates use git tag dependencies:
```
wavecraft-core = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
wavecraft-protocol = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
wavecraft-dsp = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
wavecraft-bridge = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
wavecraft-metering = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
```

**Notes**: Default behavior works correctly

---

### TC-003: Local dev mode generates path dependencies

**Description**: With `--local-dev`, generated project uses path dependencies

**Preconditions**:
- CLI is built
- Valid SDK path exists (engine/crates)
- Output directory doesn't exist

**Steps**:
1. Run `wavecraft new test-local --vendor "Test" --no-git --local-dev ./engine/crates` from workspace root
2. Check `engine/Cargo.toml` for dependency format
3. Verify all 5 SDK crates use `path = "..."` format

**Expected Result**: All SDK crates (wavecraft-core, wavecraft-protocol, wavecraft-dsp, wavecraft-bridge, wavecraft-metering) use path dependencies

**Status**: ✅ PASS

**Actual Result**: All 5 crates use absolute path dependencies:
```
wavecraft-core = { path = "/Users/ronhouben/code/private/wavecraft/engine/crates/wavecraft-core" }
wavecraft-protocol = { path = "/Users/ronhouben/code/private/wavecraft/engine/crates/wavecraft-protocol" }
wavecraft-dsp = { path = "/Users/ronhouben/code/private/wavecraft/engine/crates/wavecraft-dsp" }
wavecraft-bridge = { path = "/Users/ronhouben/code/private/wavecraft/engine/crates/wavecraft-bridge" }
wavecraft-metering = { path = "/Users/ronhouben/code/private/wavecraft/engine/crates/wavecraft-metering" }
```

**Notes**: Core functionality works as expected

---

### TC-004: Invalid path rejected with clear error

**Description**: Non-existent path for `--local-dev` produces helpful error

**Preconditions**:
- CLI is built

**Steps**:
1. Run `wavecraft new test-bad --vendor "Test" --no-git --local-dev /nonexistent/path`
2. Check error message

**Expected Result**: Clear error message indicating path doesn't exist

**Status**: ✅ PASS

**Actual Result**: 
```
Error: Failed to extract template

Caused by:
    0: Invalid local-dev path: /nonexistent/path/that/does/not/exist
    1: No such file or directory (os error 2)
```

**Notes**: Error message is clear and helpful

---

### TC-005: Relative path is canonicalized to absolute

**Description**: Relative paths are converted to absolute paths in output

**Preconditions**:
- CLI is built
- Valid SDK path exists

**Steps**:
1. Run `wavecraft new test-rel --vendor "Test" --no-git --local-dev ./engine/crates` from workspace root
2. Check dependency paths in generated `engine/Cargo.toml`
3. Verify paths are absolute (start with `/`)

**Expected Result**: All path dependencies contain absolute paths

**Status**: ✅ PASS

**Actual Result**: Relative path `./engine/crates` was converted to absolute:
```
wavecraft-core = { path = "/Users/ronhouben/code/private/wavecraft/engine/crates/wavecraft-core" }
```

**Notes**: fs::canonicalize() works correctly

---

### TC-006: --local-dev conflicts with custom --sdk-version

**Description**: Providing both flags produces an error

**Preconditions**:
- CLI is built

**Steps**:
1. Run `wavecraft new test-conflict --vendor "Test" --sdk-version v1.0.0 --local-dev ./path`
2. Check for conflict error from clap

**Expected Result**: clap produces error about conflicting arguments

**Status**: ✅ PASS

**Actual Result**:
```
error: the argument '--sdk-version <SDK_VERSION>' cannot be used with '--local-dev <LOCAL_DEV>'

Usage: wavecraft new --vendor <VENDOR> --sdk-version <SDK_VERSION> <NAME>

For more information, try '--help'.
```

**Notes**: clap's `conflicts_with` attribute works correctly

---

### TC-007: Generated project compiles with local SDK

**Description**: Project generated with `--local-dev` actually compiles

**Preconditions**:
- CLI is built
- Full SDK crates exist at specified path
- Output directory doesn't exist

**Steps**:
1. Run `wavecraft new test-compile --vendor "Test" --no-git --local-dev ./engine/crates` from workspace root
2. Run `cargo check` in generated `test-compile/engine` directory
3. Verify compilation succeeds

**Expected Result**: cargo check passes without errors

**Status**: ✅ PASS

**Actual Result**: `cargo check` completed successfully in 21.10s with 344 packages locked

**Notes**: This is the critical integration test - PASSES

---

### TC-008: All 5 SDK crates are replaced

**Description**: Verify all wavecraft-* crates are replaced, not just some

**Preconditions**:
- CLI is built
- Valid SDK path exists

**Steps**:
1. Generate project with `--local-dev`
2. grep for each crate in `engine/Cargo.toml`:
   - wavecraft-core
   - wavecraft-protocol
   - wavecraft-dsp
   - wavecraft-bridge
   - wavecraft-metering
3. Verify each uses path dependency

**Expected Result**: All 5 crates use path dependencies

**Status**: ✅ PASS

**Actual Result**: grep count shows exactly 5 wavecraft crates, all with path dependencies

**Notes**: SDK_CRATES constant ensures completeness

---

### TC-009: Non-SDK dependencies unchanged

**Description**: Dependencies that aren't wavecraft-* remain unchanged

**Preconditions**:
- CLI is built
- Valid SDK path exists

**Steps**:
1. Generate project with `--local-dev`
2. Check that nih_plug still uses git dependency
3. Check that serde still uses version dependency

**Expected Result**: Non-SDK dependencies are not modified

**Status**: ✅ PASS

**Actual Result**:
```
nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "..." }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Notes**: Regex only matches wavecraft-* crates

---

### TC-010: Unit tests pass

**Description**: All automated unit tests pass

**Preconditions**:
- CLI source code is available

**Steps**:
1. Run `cargo test` in cli directory
2. Verify all tests pass

**Expected Result**: 10/10 tests pass

**Status**: ✅ PASS

**Actual Result**: 
```
running 10 tests
test template::tests::test_apply_local_dev_overrides_no_local_dev ... ok
test template::variables::tests::test_case_transformations ... ok
test template::tests::test_apply_local_dev_overrides_invalid_path ... ok
test validation::tests::invalid_names ... ok
test validation::tests::valid_names ... ok
test template::variables::tests::test_apply ... ok
test template::variables::tests::test_unreplaced_variable ... ok
test template::tests::test_apply_local_dev_overrides ... ok
test template::variables::tests::test_empty_optional_variables ... ok
test template::tests::test_extract_template ... ok

test result: ok. 10 passed; 0 failed; 0 ignored
```

**Notes**: All unit tests pass including the 3 new tests for local dev mode

---

## Issues Found

None. All tests passed.

## Testing Notes

- All tests executed successfully on 2026-02-04
- The `--local-dev` flag works as designed
- Path canonicalization correctly handles relative paths
- Generated projects compile successfully against local SDK
- Error messages are clear and helpful

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass
- [x] Issues documented for coder agent
- [x] Ready for release: **YES**
