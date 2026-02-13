# Implementation Plan: tsconfig `paths` for SDK Development Mode

## Overview

TypeScript in `sdk-template/ui/` resolves `@wavecraft/core` from `node_modules/` (npm-published types with `id: string`), while Vite's SDK-mode alias resolves to local source (`ui/packages/core/src/` with `ParameterIdMap`/`ParameterId`). This split means the IDE shows wrong types and parameter ID autocomplete doesn't work. This plan adds tsconfig `paths` injection during SDK dev mode setup so TypeScript and Vite agree on where `@wavecraft/*` packages live.

## Requirements

- In SDK development mode, TypeScript must resolve `@wavecraft/core` and `@wavecraft/components` from local monorepo source (`ui/packages/core/src/`, `ui/packages/components/src/`)
- In end-user projects (`wavecraft create`), `tsconfig.json` stays unchanged — no `paths`, no `baseUrl`
- No new `.template` files; extend existing mutation patterns
- No changes to `sdk-template/ui/tsconfig.json` in git, `cli/build.rs`, or `sdk-template/ui/vite.config.ts`
- Must pass `cargo xtask ci-check`

## Architecture Decision

Extend the existing **runtime mutation** pattern rather than adding a `.template` file:

| Mechanism                                                  | What it mutates                 | How               |
| ---------------------------------------------------------- | ------------------------------- | ----------------- |
| `apply_local_dev_overrides()` in `cli/src/template/mod.rs` | Cargo.toml git deps → path deps | Regex replacement |
| `scripts/setup-dev-template.sh` step 3                     | Same Cargo.toml mutation        | `sed` replacement |

Both mechanisms are extended to **also** inject `baseUrl` + `paths` into `tsconfig.json` when operating in SDK dev mode. The pattern is consistent: detect SDK mode → mutate config files → verify.

## Target State

### tsconfig.json after SDK-mode injection

```jsonc
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,

    /* Bundler mode */
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",

    /* Linting */
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,

    /* SDK development — resolve @wavecraft packages from monorepo source */
    "baseUrl": ".",
    "paths": {
      "@wavecraft/core": ["../../ui/packages/core/src/index.ts"],
      "@wavecraft/core/*": ["../../ui/packages/core/src/*"],
      "@wavecraft/components": ["../../ui/packages/components/src/index.ts"],
      "@wavecraft/components/*": ["../../ui/packages/components/src/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

### tsconfig.json in git / end-user projects (unchanged)

No `baseUrl`, no `paths`. Identical to current `sdk-template/ui/tsconfig.json`.

## Implementation Steps

### Phase 1: Extend `apply_local_dev_overrides()` in Rust CLI

#### Step 1.1: Add tsconfig detection and injection logic

**File:** `cli/src/template/mod.rs` (inside `apply_local_dev_overrides()`, after the Cargo dependency replacements)

- **Action:** After the existing Cargo git→path replacement block (ends ~line 210), add a new section that detects tsconfig content and injects `paths`.
- **Detection heuristic:** Content contains `"compilerOptions"` — this identifies tsconfig files without relying on filenames (since `apply_local_dev_overrides` operates on content strings, not file paths).
- **Injection approach:** Use regex to find the closing of the last `compilerOptions` property and insert `baseUrl` + `paths` before the closing `}` of `compilerOptions`.
- **Anchor point:** Match `"noFallthroughCasesInSwitch": true` (the last property line in `compilerOptions`) and append the new properties after it.
- **Why regex (not serde_json):** The tsconfig file uses JSONC (comments like `/* Bundler mode */`). `serde_json` would strip comments. Regex preserves the file structure and comments, consistent with the Cargo.toml mutation approach.

**Concrete regex strategy:**

```rust
// Detect tsconfig content
if result.contains("\"compilerOptions\"") {
    let tsconfig_paths_re = Regex::new(
        r#"("noFallthroughCasesInSwitch":\s*true)"#
    ).context("Invalid regex for tsconfig paths injection")?;

    let paths_block = format!(
        r#"$1,

    /* SDK development — resolve @wavecraft packages from monorepo source */
    "baseUrl": ".",
    "paths": {{
      "@wavecraft/core": ["../../ui/packages/core/src/index.ts"],
      "@wavecraft/core/*": ["../../ui/packages/core/src/*"],
      "@wavecraft/components": ["../../ui/packages/components/src/index.ts"],
      "@wavecraft/components/*": ["../../ui/packages/components/src/*"]
    }}"#
    );

    result = tsconfig_paths_re.replace(&result, paths_block.as_str()).to_string();
}
```

- **Risk:** Medium — the regex anchors on `"noFallthroughCasesInSwitch": true` which could change if someone reorders tsconfig properties. However, this is the canonical Vite TypeScript template layout and is unlikely to change.
- **Mitigation:** The unit test (Step 3.1) uses the exact tsconfig content from `sdk-template/ui/tsconfig.json`, so any drift between the template and the regex will be caught immediately.

#### Step 1.2: Ensure `paths` are relative (not absolute)

- **Action:** The paths in the injected block use hardcoded relative paths (`../../ui/packages/...`), NOT the `sdk_path` variable. This is correct because:
  - `sdk-template/ui/tsconfig.json` is always at a fixed relative position to `ui/packages/` in the monorepo
  - The Vite config also uses relative paths (`../../ui/packages/core/src`) — see `sdk-template/ui/vite.config.ts` lines 16–17
  - Absolute paths would break if the repo is cloned to a different location

### Phase 2: Update shell-based setup script

#### Step 2.1: Add tsconfig paths injection to `setup-dev-template.sh`

**File:** `scripts/setup-dev-template.sh`

- **Action:** Add a new step after step 3 (Cargo dependency rewriting) and before step 4 (npm install). Use `sed` for consistency with the rest of the script (steps 2 and 3 use `sed`) and to preserve JSONC comments.

**Implementation (`sed` approach, preserves comments):**

```bash
# 4) Inject tsconfig paths for SDK development mode
TSCONFIG="$TEMPLATE_DIR/ui/tsconfig.json"
if [ -f "$TSCONFIG" ]; then
  echo "🔧 Injecting tsconfig paths for SDK development mode..."
  sed -i.bak \
    's/"noFallthroughCasesInSwitch": true/"noFallthroughCasesInSwitch": true,\n\n    \/* SDK development — resolve @wavecraft packages from monorepo source *\/\n    "baseUrl": ".",\n    "paths": {\n      "@wavecraft\/core": ["..\/..\/ui\/packages\/core\/src\/index.ts"],\n      "@wavecraft\/core\/*": ["..\/..\/ui\/packages\/core\/src\/*"],\n      "@wavecraft\/components": ["..\/..\/ui\/packages\/components\/src\/index.ts"],\n      "@wavecraft\/components\/*": ["..\/..\/ui\/packages\/components\/src\/*"]\n    }/' \
    "$TSCONFIG"
  rm -f "$TSCONFIG.bak"

  # Verify injection
  if ! grep -q '"baseUrl"' "$TSCONFIG"; then
    echo "❌ Failed to inject tsconfig paths for SDK development mode"
    exit 1
  fi
  echo "  • injected paths into $(realpath --relative-to="$REPO_ROOT" "$TSCONFIG" 2>/dev/null || echo "$TSCONFIG")"
fi
```

- **Dependencies:** Must run after step 3 (Cargo deps) and before npm install.

### Phase 3: Unit Tests

#### Step 3.1: Test tsconfig injection with `local_dev = Some(...)`

**File:** `cli/src/template/mod.rs` (in `#[cfg(test)] mod tests`)

```rust
#[test]
fn test_apply_local_dev_overrides_injects_tsconfig_paths() {
    // Use the exact tsconfig.json content from sdk-template/ui/
    let content = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,

    /* Bundler mode */
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",

    /* Linting */
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}"#;

    // ... setup tempdir with SDK paths ...

    let result = apply_local_dev_overrides(content, &vars).unwrap();

    assert!(result.contains(r#""baseUrl": ".""#));
    assert!(result.contains(r#""@wavecraft/core": ["../../ui/packages/core/src/index.ts"]"#));
    assert!(result.contains(r#""@wavecraft/components": ["../../ui/packages/components/src/index.ts"]"#));
    assert!(result.contains(r#""@wavecraft/core/*": ["../../ui/packages/core/src/*"]"#));
    assert!(result.contains(r#""@wavecraft/components/*": ["../../ui/packages/components/src/*"]"#));
    // Verify JSONC comments preserved
    assert!(result.contains("/* Bundler mode */"));
    assert!(result.contains("/* Linting */"));
    assert!(result.contains("/* SDK development"));
}
```

#### Step 3.2: Test tsconfig unchanged with `local_dev = None`

Assert content returned unchanged when `local_dev` is `None`.

#### Step 3.3: Test non-tsconfig content is unaffected

Pass Cargo.toml-like content with `local_dev = Some(...)`. Assert no `baseUrl` or `paths` injected.

### Phase 4: End-to-End Verification

#### Step 4.1: Verify `setup-dev-template.sh` injects paths

- Run `scripts/setup-dev-template.sh` from the repo root
- Verify `sdk-template/ui/tsconfig.json` contains `"baseUrl": "."` and `"paths"` block

#### Step 4.2: Verify IDE resolution in SDK mode

- Run `cargo xtask dev`, open `sdk-template/ui/src/App.tsx` in VS Code
- Verify hovering over `useParameter` shows `ParameterId` type (not `string`)
- Verify `useParameter('` triggers autocomplete with concrete parameter IDs
- Verify Cmd+Click on `@wavecraft/core` import navigates to `ui/packages/core/src/index.ts`

#### Step 4.3: Verify end-user template is unaffected

- Run `cargo run --manifest-path cli/Cargo.toml -- create TestPlugin --output target/tmp/test-tsconfig`
- Verify `target/tmp/test-tsconfig/ui/tsconfig.json` does NOT contain `baseUrl` or `paths`
- Cleanup: `rm -rf target/tmp/test-tsconfig`

#### Step 4.4: Run `cargo xtask ci-check`

- Run full CI check to ensure nothing is broken
- Verify all phases pass

## Files Changed

| File                            | Change                                                          | Phase |
| ------------------------------- | --------------------------------------------------------------- | ----- |
| `cli/src/template/mod.rs`       | Add tsconfig `paths` injection in `apply_local_dev_overrides()` | 1     |
| `cli/src/template/mod.rs`       | Add 3 unit tests for tsconfig injection                         | 3     |
| `scripts/setup-dev-template.sh` | Add tsconfig `paths` injection step                             | 2     |

## Files NOT Changed (explicit)

| File                                       | Reason                                                   |
| ------------------------------------------ | -------------------------------------------------------- |
| `sdk-template/ui/tsconfig.json`            | Stays clean in git — paths injected at runtime only      |
| `sdk-template/ui/vite.config.ts`           | Already handles SDK-mode aliases correctly               |
| `cli/build.rs`                             | No changes to template embedding                         |
| `cli/src/template/variables.rs`            | No new template variables needed                         |
| `ui/packages/core/src/types/parameters.ts` | `ParameterIdMap`/`ParameterId` already defined correctly |

## Testing Strategy

### Unit Tests (automated, in `cargo test`)

| Test                                                                 | Description                                                             | Phase |
| -------------------------------------------------------------------- | ----------------------------------------------------------------------- | ----- |
| `test_apply_local_dev_overrides_injects_tsconfig_paths`              | tsconfig + `local_dev = Some(...)` → paths injected, comments preserved | 3     |
| `test_apply_local_dev_overrides_no_tsconfig_paths_without_local_dev` | tsconfig + `local_dev = None` → content unchanged                       | 3     |
| `test_apply_local_dev_overrides_ignores_non_tsconfig_files`          | Cargo.toml + `local_dev = Some(...)` → no tsconfig injection            | 3     |

### Manual Tests (in test plan)

| Test                                              | Description                                   | Phase |
| ------------------------------------------------- | --------------------------------------------- | ----- |
| `setup-dev-template.sh` produces correct tsconfig | Run script, inspect output file               | 4     |
| IDE autocomplete works in SDK mode                | `cargo xtask dev`, check VS Code IntelliSense | 4     |
| End-user template has no paths                    | `wavecraft create`, inspect tsconfig          | 4     |
| CI passes                                         | `cargo xtask ci-check`                        | 4     |

## Risks & Mitigations

| Risk                                                                              | Likelihood | Mitigation                                                                     |
| --------------------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------ |
| Regex anchor on `"noFallthroughCasesInSwitch": true` breaks if tsconfig reordered | Low        | Unit test uses exact tsconfig content; any drift breaks test immediately       |
| tsconfig paths vs Vite aliases diverge                                            | Low        | Both use hardcoded relative paths; document coupling in code comment           |
| `paths` + `moduleResolution: "bundler"` interaction                               | Very low   | `paths` is well-supported with all resolution strategies; verified in E2E test |

## Success Criteria

- [ ] `apply_local_dev_overrides()` injects `baseUrl` + `paths` into tsconfig when `local_dev` is set
- [ ] `apply_local_dev_overrides()` leaves tsconfig unchanged when `local_dev` is `None`
- [ ] `apply_local_dev_overrides()` does not inject paths into non-tsconfig files
- [ ] `setup-dev-template.sh` injects paths into `sdk-template/ui/tsconfig.json`
- [ ] `setup-dev-template.sh` fails with clear error if injection didn't work
- [ ] VS Code resolves `@wavecraft/core` to local source in SDK dev mode
- [ ] `useParameter('` triggers autocomplete with concrete parameter IDs
- [ ] `wavecraft create` produces tsconfig without `paths`/`baseUrl`
- [ ] `cargo xtask ci-check` passes
- [ ] All 3 unit tests pass

## Estimated Effort

| Phase                        | Effort       | Complexity |
| ---------------------------- | ------------ | ---------- |
| Phase 1: Rust CLI injection  | ~30 min      | Low        |
| Phase 2: Shell script update | ~20 min      | Low        |
| Phase 3: Unit tests          | ~30 min      | Low        |
| Phase 4: E2E verification    | ~30 min      | Low        |
| **Total**                    | **~2 hours** | **Low**    |

## Implementation Order

1. **Phase 1** first — core logic in Rust CLI
2. **Phase 3** immediately after — validate with unit tests before touching shell script
3. **Phase 2** — shell script mirrors the Rust logic
4. **Phase 4** — end-to-end verification of both paths

## Related Documents

- [User Stories](./user-stories.md) — Feature requirements and acceptance criteria
- [Test Plan](./test-plan.md) — Test cases and results
- [Coding Standards (Rust)](../../architecture/coding-standards-rust.md) — Rust conventions
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Development Workflows](../../architecture/development-workflows.md) — `cargo xtask dev`, build system
