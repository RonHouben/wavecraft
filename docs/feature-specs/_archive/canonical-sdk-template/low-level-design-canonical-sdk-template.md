# Low-Level Design: Canonical SDK Template

**Status:** Draft  
**Created:** 2026-02-12  
**Author:** Architect Agent

---

## Problem Statement

The Wavecraft SDK currently maintains **three overlapping sources** for what a plugin project looks like:

1. **`cli/sdk-templates/new-project/react/`** — the template embedded by the CLI into new projects via `wavecraft create`.
2. **`engine/crates/wavecraft-example/`** — the SDK-internal dev fixture used by `cargo xtask dev` as the engine target.
3. **`ui/src/` (top-level)** — a development app that serves as the UI harness for SDK package iteration.

This creates:

- **Template drift** — changes to `wavecraft-example` or `ui/src` don't automatically propagate to what users get from `wavecraft create`.
- **Developer confusion** — it's unclear where to edit UI code when testing SDK packages (`ui/src`? `cli/sdk-templates/.../ui/src`? `wavecraft-example`?).
- **Parity risk** — the dev environment (`cargo xtask dev`) runs a different codebase than what users scaffold, so bugs can hide.

### Goal

Establish a **single canonical scaffold** at repo root (`sdk-template/`) that serves as:

- the source for `wavecraft create` (embedded into CLI binary),
- the dev target for `cargo xtask dev` (both engine and UI),
- the living reference implementation of a Wavecraft plugin.

Remove `engine/crates/wavecraft-example/`, remove `cli/sdk-templates/new-project/react/`, and make `ui/` a pure npm workspace for publishable packages only (no app harness).

---

## Architecture Overview

### Current State

```
wavecraft/
├── cli/sdk-templates/new-project/react/   ← embedded into CLI binary
│   ├── engine/                            ← template engine (Cargo.toml.template)
│   └── ui/                               ← template UI
├── engine/crates/wavecraft-example/       ← SDK-mode engine target for xtask dev
├── ui/
│   ├── src/App.tsx                        ← dev app harness (SDK-internal)
│   └── packages/                          ← publishable npm packages
```

**Problem:** three sources, three mental models, easy to drift.

### Target State

```
wavecraft/
├── sdk-template/                          ← single source of truth
│   ├── engine/                            ← template engine + dev target
│   │   ├── Cargo.toml.template            ← used by CLI for scaffolding
│   │   └── src/lib.rs                     ← reference plugin implementation
│   └── ui/                                ← template UI + dev target
│       ├── package.json                   ← standalone (not workspace member)
│       ├── src/App.tsx                     ← reference plugin UI
│       └── vite.config.ts                 ← aliased to local packages in SDK mode
├── ui/
│   └── packages/                          ← publishable npm packages only
│       ├── core/                           ← @wavecraft/core
│       └── components/                     ← @wavecraft/components
```

**Result:** one canonical scaffold, tested via `cargo xtask dev`, embedded by CLI.

---

## Data Flow Changes

### `wavecraft create` (CLI)

```
Before:  include_dir!("$CARGO_MANIFEST_DIR/sdk-templates/new-project/react")
After:   include_dir!("$CARGO_MANIFEST_DIR/../sdk-template")
```

No functional change to output — same template files, same variable substitution, same local-dev overrides.

### `cargo xtask dev` (SDK development)

```
Before:
  engine_dir  → engine/crates/wavecraft-example/
  ui_dir      → ui/  (top-level dev app)

After:
  engine_dir  → sdk-template/engine/
  ui_dir      → sdk-template/ui/
```

Requires a one-time bootstrap step to process `.template` files for local dev (see Phase 4).

### Package development (`ui/packages/*`)

```
Before:  Vite aliases in ui/vite.config.ts resolve packages from source
After:   Vite aliases in sdk-template/ui/vite.config.ts resolve packages from source
```

The alias mechanism is preserved — it just lives in a different `vite.config.ts`.

---

## Detailed Design by Phase

### Phase 0: Baseline Validation

**Goal:** Confirm current state is green before any changes.

**Actions:**

1. Run `cargo xtask ci-check`.
2. Verify `cargo xtask dev` starts successfully.
3. Verify `wavecraft create TestPlugin --output target/tmp/test0` produces a compilable project.

**Go/No-Go:** All three pass → proceed.

---

### Phase 1: Add `sdk-template/` (Non-Breaking)

**Goal:** Introduce canonical directory without changing any behavior.

| File                                     | Action | Details                                                                                     |
| ---------------------------------------- | ------ | ------------------------------------------------------------------------------------------- |
| `sdk-template/`                          | CREATE | Copy from `cli/sdk-templates/new-project/react/`                                            |
| `sdk-template/README.md`                 | CREATE | Document purpose: canonical Wavecraft plugin template                                       |
| `.gitignore`                             | EDIT   | Add `sdk-template/engine/target/`, `sdk-template/ui/node_modules/`, `sdk-template/ui/dist/` |
| `docs/architecture/high-level-design.md` | EDIT   | Add `sdk-template/` to repo structure diagram                                               |

**Verification:**

- `cargo xtask ci-check` green.
- `wavecraft create` unchanged (still uses old embedded path).

**Rollback:** Delete `sdk-template/`.

---

### Phase 2: Switch CLI Embedding

**Goal:** `wavecraft create` reads from `sdk-template/`.

| File                      | Action | Details                                                             |
| ------------------------- | ------ | ------------------------------------------------------------------- |
| `cli/src/template/mod.rs` | EDIT   | Change `include_dir!` path to `$CARGO_MANIFEST_DIR/../sdk-template` |

**Verification:**

```bash
cargo run --manifest-path cli/Cargo.toml -- create test-p2 --output target/tmp/test-p2 --no-git
cd target/tmp/test-p2/engine && cargo clippy --all-targets -- -D warnings
```

- Generated project identical in structure and content.
- CLI tests pass.

**Risk:** `include_dir!` path resolution during `cargo publish`. **Mitigation:** CLI is git-only (not published to crates.io yet).

**Rollback:** Revert one line in `cli/src/template/mod.rs`.

---

### Phase 3: Update CI Workflows

**Goal:** Automation follows new source of truth.

| File                                             | Action | Details                                                                                   |
| ------------------------------------------------ | ------ | ----------------------------------------------------------------------------------------- |
| `.github/workflows/template-validation.yml`      | EDIT   | Update path triggers and working directories from `cli/sdk-templates/` to `sdk-template/` |
| `.github/workflows/continuous-deploy.yml`        | EDIT   | Update path filter from `cli/sdk-templates/**` to `sdk-template/**`                       |
| `engine/xtask/src/commands/validate_template.rs` | REVIEW | Confirm it validates generated output (not source) — likely no change needed              |

**Verification:**

- CI green on feature branch.
- `cargo xtask validate-template` passes locally.

**Rollback:** Revert workflow files.

---

### Phase 4: Rewire `cargo xtask dev` (Highest-Risk Phase)

**Goal:** Dev mode uses `sdk-template/engine` and `sdk-template/ui`.

#### 4a. Template bootstrap for dev mode

The template engine has `Cargo.toml.template` (with placeholders like `{{PLUGIN_NAME}}`). For dev mode, we need a concrete `Cargo.toml`. Two options:

| Option                                                                                       | Pros                | Cons                                         |
| -------------------------------------------------------------------------------------------- | ------------------- | -------------------------------------------- |
| **A. Setup script** (`scripts/setup-dev-template.sh`) processes `.template` → concrete files | Explicit, no magic  | Extra manual step                            |
| **B. Detection code auto-processes on first run**                                            | Zero-step dev start | Implicit side effect, harder to reason about |

**Recommendation:** Option A with a clear error message in detection code pointing to the script. Explicit is better for a developer-facing SDK.

#### 4b. File changes

| File                                         | Action | Details                                                                                 |
| -------------------------------------------- | ------ | --------------------------------------------------------------------------------------- |
| `cli/src/project/detection.rs`               | EDIT   | SDK-mode branch: set `engine_dir` → `sdk-template/engine`, `ui_dir` → `sdk-template/ui` |
| `scripts/setup-dev-template.sh`              | CREATE | Process `.template` files with dev defaults, run `npm install` in `sdk-template/ui`     |
| `docs/architecture/development-workflows.md` | EDIT   | Document new dev setup flow                                                             |

#### 4c. Vite aliases for local package development

The current `ui/vite.config.ts` has aliases pointing at `packages/core/src` and `packages/components/src` so that the dev app imports live package sources. This must be replicated in `sdk-template/ui/vite.config.ts` for SDK-mode dev:

```typescript
// sdk-template/ui/vite.config.ts (SDK-mode only, via env or config flag)
resolve: {
  alias: {
    '@wavecraft/core': path.resolve(__dirname, '../../ui/packages/core/src'),
    '@wavecraft/components': path.resolve(__dirname, '../../ui/packages/components/src'),
  }
}
```

**Important:** These aliases only apply when developing the SDK itself. Generated user projects do NOT get these aliases — they use published npm packages.

#### 4d. Verification (manual smoke test required)

- [ ] `./scripts/setup-dev-template.sh` completes without errors
- [ ] `cargo xtask dev` starts both servers
- [ ] `http://localhost:5173` loads UI from `sdk-template/ui`
- [ ] WebSocket connects and parameters load
- [ ] Edit `sdk-template/ui/src/App.tsx` → HMR updates browser
- [ ] Edit `sdk-template/engine/src/lib.rs` → engine hot-reloads
- [ ] Edit `ui/packages/core/src/*.ts` → change reflected in dev UI (alias test)
- [ ] `cargo xtask ci-check` passes

**Rollback:** Revert detection.rs + delete setup script.

---

### Phase 5: Delete Legacy Paths

**Goal:** Remove now-redundant directories.

| File                                     | Action | Details                                                    |
| ---------------------------------------- | ------ | ---------------------------------------------------------- |
| `cli/sdk-templates/`                     | DELETE | Entire tree — CLI now embeds from `sdk-template/`          |
| `engine/crates/wavecraft-example/`       | DELETE | Entire crate — dev mode now targets `sdk-template/engine/` |
| `engine/Cargo.toml`                      | EDIT   | Remove `crates/wavecraft-example` from workspace members   |
| `docs/guides/sdk-getting-started.md`     | EDIT   | Point example references to `sdk-template/`                |
| `docs/architecture/high-level-design.md` | EDIT   | Remove `cli/sdk-templates/` from repo structure            |

**Pre-delete validation:**

```bash
grep -r "sdk-templates" . --include="*.rs" --include="*.yml" --include="*.md"
grep -r "wavecraft-example" . --include="*.rs" --include="*.yml" --include="*.toml" --include="*.md"
```

All matches must be in git history or documentation referencing the migration — not live code paths.

**Verification:**

- `cargo check --manifest-path engine/Cargo.toml --workspace` compiles.
- `cargo xtask dev` works.
- `wavecraft create` works.
- `cargo xtask ci-check` passes.

**Rollback:** `git revert` the deletion commit.

---

### Phase 6: Deprecate and Remove `ui/src`

**Goal:** Top-level `ui/` becomes a pure npm workspace for publishable packages.

#### 6a. Deprecation (first PR)

| File              | Action | Details                                                |
| ----------------- | ------ | ------------------------------------------------------ |
| `ui/README.md`    | EDIT   | Add deprecation notice                                 |
| `CONTRIBUTING.md` | EDIT   | Document that `sdk-template/ui` is the dev entry point |

#### 6b. Removal (second PR, after cooldown)

| File                    | Action             | Details                                                                                      |
| ----------------------- | ------------------ | -------------------------------------------------------------------------------------------- |
| `ui/src/`               | DELETE             | App.tsx, main.tsx, index.css, vite-env.d.ts                                                  |
| `ui/src/test/`          | DELETE or MOVE     | Test utilities may belong in `ui/packages/core/test/`                                        |
| `ui/index.html`         | DELETE             | Entry point for old dev app                                                                  |
| `ui/vite.config.ts`     | DELETE or SIMPLIFY | No longer needs dev app config; may keep for package builds                                  |
| `ui/vitest.config.ts`   | KEEP               | Still needed for package test runner                                                         |
| `ui/tailwind.config.js` | KEEP               | Still needed for package builds                                                              |
| `ui/package.json`       | EDIT               | Remove `dev`/`build`/`preview` scripts that reference the app; keep workspace + test scripts |

**Pre-delete validation:**

```bash
grep -r "ui/src" . --include="*.ts" --include="*.yml" --include="*.md" --include="*.json"
```

**Verification:**

- `npm test` in `ui/` still runs package tests.
- `cargo xtask dev` works (uses `sdk-template/ui`).
- `cargo xtask ci-check` passes.

---

## Risk Register

| Risk                                                  | Likelihood | Impact | Mitigation                                                           | Phase |
| ----------------------------------------------------- | ---------- | ------ | -------------------------------------------------------------------- | ----- |
| `include_dir!` path breaks during `cargo package`     | Medium     | High   | CLI is git-only; test with `cargo package --no-verify`               | 2     |
| Developers forget setup script before `xtask dev`     | High       | Low    | Clear error message in `ProjectMarkers::detect()` with exact command | 4     |
| Template variable replacement fails in setup script   | Low        | Medium | Explicit sed patterns; integration test                              | 4     |
| Vite aliases break for package development            | Medium     | High   | Verify alias resolution in Phase 4d smoke test                       | 4     |
| Hidden CI references to old paths                     | Medium     | Medium | Repo-wide grep sweep before each deletion phase                      | 5, 6  |
| Hot-reload breaks after path changes                  | Low        | High   | Manual HMR test in Phase 4d                                          | 4     |
| Loss of package-dev ergonomics after `ui/src` removal | Medium     | Medium | Replicate alias strategy in `sdk-template/ui/vite.config.ts` first   | 6     |

---

## PR Slicing Strategy

| PR  | Phases | Title                                                          | Risk     | Size          |
| --- | ------ | -------------------------------------------------------------- | -------- | ------------- |
| 1   | 0–1    | `refactor: add canonical sdk-template/ directory`              | Minimal  | S             |
| 2   | 2      | `feat(cli): embed template from canonical sdk-template/`       | Low      | S             |
| 3   | 3      | `ci: migrate template validation to sdk-template/`             | Low      | S             |
| 4   | 4      | `feat(xtask): dev mode uses canonical sdk-template/`           | **High** | M             |
| 5   | 5      | `chore: remove legacy cli/sdk-templates and wavecraft-example` | Low      | L (deletions) |
| 6a  | 6a     | `docs: deprecate top-level ui/src`                             | Minimal  | S             |
| 6b  | 6b     | `chore: remove top-level ui/src dev app`                       | Low      | M             |

### Go/No-Go Gates Between PRs

- **PR 2 requires:** PR 1 merged, CI green.
- **PR 3 requires:** PR 2 merged, CI green.
- **PR 4 requires:** PR 2 + PR 3 merged. Manual smoke test pass. **This is the critical gate.**
- **PR 5 requires:** PR 4 stable for at least one dev cycle (dev flow validated by team).
- **PR 6b requires:** PR 6a merged. Confirm no workflows/scripts depend on `ui/src`.

---

## Success Criteria

- [ ] Single canonical scaffold at `sdk-template/`.
- [ ] `wavecraft create` embeds from `sdk-template/`.
- [ ] `cargo xtask dev` runs against `sdk-template/`.
- [ ] `cli/sdk-templates/new-project/react/` deleted.
- [ ] `engine/crates/wavecraft-example/` deleted.
- [ ] `ui/src/` removed; `ui/` is package workspace only.
- [ ] CI green on all workflows.
- [ ] No grep matches for old paths in live code/config/docs.
- [ ] Package development workflow preserved via Vite aliases in `sdk-template/ui`.

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview, repo structure
- [SDK Architecture](../../architecture/sdk-architecture.md) — Crate structure, npm packages
- [Development Workflows](../../architecture/development-workflows.md) — Browser dev mode, build system
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
