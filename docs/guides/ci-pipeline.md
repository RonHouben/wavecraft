# CI/CD Pipeline

This document describes the CI/CD pipeline architecture for Wavecraft.

## Overview

The CI pipeline runs on all pull requests to `main` (not on merge/push). It consists of two independent pipelines that run in **parallel**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CI PIPELINE                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  UI Pipeline                         Engine Pipeline                        │
│  ────────────                        ───────────────                        │
│                                                                             │
│  ┌──────────┐                        ┌────────────────┐                     │
│  │ check-ui │                        │ prepare-engine │                     │
│  │ ~20s     │                        │ ~4min          │                     │
│  └────┬─────┘                        └───────┬────────┘                     │
│       │                                      │                              │
│       ▼                                      ▼                              │
│  ┌─────────┐                         ┌──────────────┐                       │
│  │ test-ui │                         │ check-engine │                       │
│  │ ~15s    │                         │ ~30s         │                       │
│  └─────────┘                         └───────┬──────┘                       │
│                                              │                              │
│                                              ▼                              │
│                                      ┌─────────────┐                        │
│                                      │ test-engine │                        │
│                                      │ ~1min       │                        │
│                                      └─────────────┘                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Jobs

### UI Pipeline

| Job          | Runner        | Duration | Description                               |
| ------------ | ------------- | -------- | ----------------------------------------- |
| **check-ui** | ubuntu-latest | ~20s     | Prettier → ESLint → TypeScript type-check |
| **test-ui**  | ubuntu-latest | ~15s     | Vitest unit tests                         |

### Engine Pipeline

| Job                | Runner        | Duration | Description                                      |
| ------------------ | ------------- | -------- | ------------------------------------------------ |
| **prepare-engine** | ubuntu-latest | ~4min    | Build UI dist + compile with clippy              |
| **check-engine**   | ubuntu-latest | ~30s     | `cargo fmt --check` → `cargo clippy -D warnings` |
| **test-engine**    | ubuntu-latest | ~1min    | `cargo test --workspace`                         |

## Design Principles

### 1. Parallel Pipelines

UI and Engine pipelines run **completely independently**. This means:

- UI checks don't block Engine compilation
- Fast feedback on UI issues (~35s total)
- Engine issues don't delay UI feedback

### 2. Fail Fast

Each pipeline orders checks from fastest to slowest:

**UI:** Prettier (syntax) → ESLint (style) → TypeScript (types)  
**Engine:** `cargo fmt` (syntax) → `cargo clippy` (lint + types)

### 3. Clippy-Compatible Build

The `prepare-engine` job uses `cargo clippy --all-targets --no-deps` instead of `cargo build`. This is intentional:

- **Why not `cargo build`?** Clippy uses different compiler metadata than regular builds. If we use `cargo build`, the `check-engine` job would recompile everything with clippy.
- **Solution:** Compile with clippy from the start, so downstream jobs can reuse the artifacts.
- **`--no-deps`:** Only lint workspace crates, not dependencies (faster, and we don't control dependency code).

### 4. Artifact Sharing

The `prepare-engine` job uploads:

1. **ui-dist** — Built UI files for embedding in the plugin
2. **engine-target** — Compiled Rust artifacts (deps, build, fingerprint, incremental)

Downstream jobs (`check-engine`, `test-engine`) download these artifacts to avoid recompilation.

### 5. Caching Strategy

| Cache       | Scope      | What's Cached                              |
| ----------- | ---------- | ------------------------------------------ |
| npm cache   | Cross-run  | Downloaded packages from npm registry      |
| apt cache   | Cross-run  | Linux system dependencies (GTK, WebKit)    |
| Cargo cache | Cross-run  | Crates.io registry + compiled dependencies |
| Artifacts   | Within-run | `ui/dist` + `engine/target/debug`          |

## Artifacts

### Within-Run Artifacts (1 day retention)

| Artifact        | Source         | Used By                   |
| --------------- | -------------- | ------------------------- |
| `ui-dist`       | prepare-engine | check-engine, test-engine |
| `engine-target` | prepare-engine | check-engine, test-engine |

## Concurrency

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

When a new commit is pushed to a branch:

- In-progress runs for the same branch are **cancelled**
- Only the latest commit is built
- Saves CI minutes on rapid iterations

## Linux Dependencies

Engine jobs require system libraries for WebView support:

```
libwebkit2gtk-4.1-dev  # WebKit for WebView
libgtk-3-dev           # GTK toolkit
libayatana-appindicator3-dev
librsvg2-dev
libxdo-dev
libx11-xcb-dev         # X11/XCB for windowing
libxcb-render0-dev
libxcb-shape0-dev
libxcb-xfixes0-dev
```

These are cached using `awalsh128/cache-apt-pkgs-action`.

## Runners

| Platform        | Usage                   | Cost           |
| --------------- | ----------------------- | -------------- |
| `ubuntu-latest` | All checks and tests    | Cheapest       |
| `macos-latest`  | Final plugin build only | More expensive |

Using Ubuntu for most jobs significantly reduces CI costs while macOS is only used for the final build (which requires native toolchain for proper signing).

## Triggers

### CI and Template Validation

```yaml
on:
  pull_request:
    branches: [main]
  workflow_dispatch: # Manual trigger for emergencies
```

- **Pull Requests:** Full validation (CI + Template Validation)
- **Main branch:** No automatic runs — code already validated via PR
- **Manual:** Available via GitHub Actions UI (`workflow_dispatch`)

### Continuous Deploy

```yaml
on:
  push:
    branches: [main]
  workflow_dispatch:
```

- **Main branch:** Publishes changed packages after PR merge
- **Manual:** Available via GitHub Actions UI

## Local Testing

### Fast Local Validation (Recommended)

Use `cargo xtask ci-check` for fast pre-push validation that simulates CI checks locally:

```bash
# Standard checks: docs, UI build, lint+typecheck, tests (~1 minute)
cargo xtask ci-check

# Auto-fix linting issues
cargo xtask ci-check --fix

# Full validation (adds template validation + CD dry-run)
cargo xtask ci-check --full   # or -F

# Skip individual phases
cargo xtask ci-check --skip-docs
cargo xtask ci-check --skip-lint
cargo xtask ci-check --skip-tests
cargo xtask ci-check -F --skip-template
cargo xtask ci-check -F --skip-cd
```

**Why use `cargo xtask ci-check`:**

- **26x faster** than Docker-based CI testing (~52s vs ~9-12 min)
- Runs natively on your machine (no Docker overhead)
- Same checks as CI pipeline (docs, UI build, lint+typecheck, tests; plus template validation and CD dry-run with `--full`)
- Recommended before every push

**Visual testing** is done separately via the `playwright-mcp-ui-testing` skill:

```bash
cargo xtask dev  # Start dev servers
# Then use Playwright MCP for browser-based testing
```

### Docker-Based Testing (For CI Workflow Debugging)

The CI pipeline can also be tested locally using `act` and a custom Docker image. This is slower but useful for:

- Debugging GitHub Actions workflow YAML changes
- Testing artifact upload/download between jobs
- Validating container-specific issues

```bash
# Build the custom image (one-time)
docker build --platform linux/amd64 -t wavecraft-ci:latest \
    .github/skills/run-ci-pipeline-locally/

# Run a specific job
act -j check-engine -W .github/workflows/ci.yml \
    --container-architecture linux/amd64 \
    -P ubuntu-latest=wavecraft-ci:latest \
    --pull=false \
    --artifact-server-path ./tmp/act-artifacts
```

**Note:** The `--artifact-server-path` flag enables local artifact upload/download between jobs.

### What Can Be Tested Locally

| Job            | Local Testing |
| -------------- | ------------- |
| check-ui       | ✅ Works      |
| test-ui        | ✅ Works      |
| prepare-engine | ✅ Works      |
| check-engine   | ✅ Works      |
| test-engine    | ✅ Works      |

For detailed local testing instructions, see the [Run CI Pipeline Locally skill](/.github/skills/run-ci-pipeline-locally/SKILL.md).

---

## Template Validation

The `template-validation.yml` workflow validates that the CLI generates working projects. This catches template bugs before release.

### Why `--local-dev`?

Generated plugins reference SDK crates via git tags (e.g., `tag = "v0.7.0"`). However, the tag doesn't exist until **after** the PR is merged. Cargo's `[patch]` mechanism cannot be used because it requires the original source to be resolvable first (chicken-and-egg problem).

**Solution:** The `--local-dev` CLI flag generates path dependencies directly, bypassing git entirely.

### Workflow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     TEMPLATE VALIDATION                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. Build CLI from source                                                   │
│        │                                                                    │
│        ▼                                                                    │
│  2. Generate test plugin with --local-dev                                   │
│     wavecraft create test-plugin --local-dev ${{ github.workspace }}/engine/crates
│        │                                                                    │
│        ▼                                                                    │
│  3. Verify structure (Cargo.toml, lib.rs, package.json, App.tsx)            │
│        │                                                                    │
│        ▼                                                                    │
│  4. cargo check (validates SDK integration)                                 │
│        │                                                                    │
│        ▼                                                                    │
│  5. cargo clippy + cargo fmt                                                │
│        │                                                                    │
│        ▼                                                                    │
│  6. npm install + npm run build (validates UI integration)                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Path Dependency Resolution

With `--local-dev`, generated Cargo.toml uses absolute paths:

```toml
# Instead of:
wavecraft-core = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }

# Generated:
wavecraft-core = { path = "/home/runner/work/wavecraft/wavecraft/engine/crates/wavecraft-core" }
```

This allows validation against the current commit's SDK code, even before release tags exist.

## Related Documentation

- [Coding Standards](../architecture/coding-standards.md) — Code conventions overview (see [Testing & Quality](../architecture/coding-standards-testing.md) for linting rules)
- [macOS Signing Guide](./macos-signing.md) — Plugin signing and notarization

---

## Continuous Deployment

Wavecraft uses automatic continuous deployment for all publishable packages. When changes are merged to `main`, packages are automatically published to their respective registries.

### Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CONTINUOUS DEPLOYMENT                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  PR merged to main                                                          │
│        │                                                                    │
│        ▼                                                                    │
│  ┌──────────────────────────────────────────────────┐                        │
│  │ detect-changes                                   │                        │
│  │ • Skips if author is github-actions[bot]         │                        │
│  │ • Outputs: cli, engine, npm-core, npm-components │                        │
│  │ • Computes: npm-cohort + any_sdk_changed         │                        │
│  └────────┬─────────────────────────────────────────┘                        │
│           │                                                                 │
│     ┌─────┼───────────────────┬───────────────────────┐                      │
│     ▼     ▼                   ▼                       ▼                      │
│  ┌──────┐ ┌─────────────────┐ ┌───────────┐ ┌───────────────┐               │
│  │Engine│ │ npm-cohort-     │ │ npm-core  │ │npm-components │               │
│  │      │ │ prepare         │ │ (lockstep)│ │ (lockstep)    │               │
│  └──┬───┘ └────────┬────────┘ └─────┬─────┘ └──────┬────────┘               │
│     │              └────────────────┴───────────────┘                        │
│     │           │              │                                            │
│     └───────────┼──────────────┘                                            │
│                 │                                                           │
│                 ▼                                                           │
│         ┌──────────────┐                                                    │
│         │  publish-cli │  Cascade: triggers on ANY SDK change               │
│         │  (last)      │  Waits for all upstream jobs                       │
│         └──────┬───────┘                                                    │
│                │                                                            │
│                ▼                                                            │
│         Git tag: wavecraft-cli-vX.Y.Z                                       │
│         (template references this tag)                                      │
│                                                                             │
│  Registry targets:                                                          │
│    Engine → crates.io (6 crates)                                            │
│    npm-core + npm-components → npmjs.org (single cohort target version)     │
│    CLI → crates.io (wavecraft)                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Workflow: `continuous-deploy.yml`

**Trigger:** Push to `main` branch (i.e., PR merge)

| Job                          | Trigger Condition                                 | Publishes To                   |
| ---------------------------- | ------------------------------------------------- | ------------------------------ |
| `publish-engine`             | `engine` or `cli` changed                         | crates.io (6 crates)           |
| `publish-npm-cohort-prepare` | npm cohort changed or forced                      | computes single target version |
| `publish-npm-core`           | `npm-cohort` true                                 | npm (`@wavecraft/core`)        |
| `publish-npm-components`     | `npm-cohort` true (after core)                    | npm (`@wavecraft/components`)  |
| `publish-cli`                | **Any** SDK component changed (`any_sdk_changed`) | crates.io (`wavecraft`)        |

**Key difference:** `publish-cli` is a cascade job — it triggers whenever _any_ SDK package (engine, npm-core, npm-components, or CLI itself) changes. This ensures the CLI git tag always points to the latest SDK state, since scaffolded projects depend on that tag.

### CLI Dependency Validation

The `publish-cli` job validates CLI dependencies before publishing using `cargo xtask validate-cli-deps`. This single command performs two checks for every `wavecraft-*` dependency discovered in `cli/Cargo.toml`:

1. **Version field** — Each `wavecraft-*` dependency must have a `version` field (required for `cargo publish` to crates.io)
2. **Publishability** — The corresponding crate at `engine/crates/{name}/Cargo.toml` must not have `publish = false`

Dependencies are **discovered dynamically** from `cli/Cargo.toml` — adding or removing engine crate dependencies does not require updating the workflow file.

The command can also be run locally:

```bash
cd engine && cargo xtask validate-cli-deps          # basic output
cd engine && cargo xtask validate-cli-deps --verbose # per-dependency details
```

### Packages Published

#### npm Packages

| Package                 | Description                         |
| ----------------------- | ----------------------------------- |
| `@wavecraft/core`       | IPC bridge, hooks, types, utilities |
| `@wavecraft/components` | Pre-built React components          |

#### Rust Crates (crates.io)

| Crate                | Description                      |
| -------------------- | -------------------------------- |
| `wavecraft`          | CLI tool for scaffolding plugins |
| `wavecraft-protocol` | Shared parameter definitions     |
| `wavecraft-macros`   | Procedural macros                |
| `wavecraft-metering` | SPSC ring buffer for audio → UI  |
| `wavecraft-dsp`      | Pure DSP algorithms              |
| `wavecraft-bridge`   | IPC handling                     |
| `wavecraft-core`     | Core SDK types and macros        |

### Two Version Domains

The CD pipeline operates with two distinct version domains:

| Domain                   | Packages                                        | Ownership                    | Bumped By                                |
| ------------------------ | ----------------------------------------------- | ---------------------------- | ---------------------------------------- |
| **Product Version**      | `engine/Cargo.toml` workspace version           | PO decides, Coder implements | Manual — during feature development      |
| **Distribution Version** | CLI, `@wavecraft/core`, `@wavecraft/components` | CI                           | Automatic — patch bump on any SDK change |

### Auto-Bump Pattern

Distribution packages keep CI-managed patch bumps, with npm packages handled as a **single lockstep cohort**:

1. **Compute one cohort target** — `publish-npm-cohort-prepare` computes one version for `@wavecraft/core` and `@wavecraft/components` by taking the highest semver across both local and published versions.
2. **Align manifests locally** — Core and components package manifests are aligned to the cohort target; components also aligns internal `@wavecraft/core` dependency ranges to `^<target>`.
3. **Publish in order** — Core publishes first, then components.
4. **Idempotent reruns** — If `@wavecraft/*@<target>` already exists, publish is skipped safely.
5. **Push tag only** — After publish/skip, tag creation is guarded so existing tags do not fail reruns.

For crates.io publish jobs, a "set final version" step consolidates the version (whether from determine or bump) for downstream publish/tag steps. For npm, `publish-npm-cohort-prepare` computes and fans out one shared cohort target version used by both npm publish jobs.

**Why local-only commit?** Branch protection rulesets on `main` prevent direct pushes. The local commit is needed so `cargo publish` / `npm publish` see the correct version in the committed working tree. The published version is recorded in the git tag and the registry — `main` retains the product baseline version.

**Developer override:** If a developer manually bumps the version in their PR (e.g., minor bump for breaking changes), CI detects the local version is already ahead and publishes it as-is without auto-bumping.

### Infinite Loop Prevention

Since auto-bump commits are no longer pushed to `main`, the infinite loop scenario (auto-bump commit re-triggers CD pipeline) no longer applies. The `detect-changes` guard is kept as defense-in-depth:

```
detect-changes job:
  if: github.event.head_commit.author.name != 'github-actions[bot]'
  → Skips if the triggering commit was authored by the bot
```

**Why keep the guard?** Defense-in-depth — if the workflow is ever modified to push commits again, the guard prevents infinite loops without requiring additional changes.

### CLI Cascade Trigger

The `publish-cli` job has special behavior:

- **`needs`**: Depends on all four jobs (`detect-changes`, `publish-engine`, `publish-npm-core`, `publish-npm-components`)
- **Trigger**: `any_sdk_changed == 'true'` (fires on _any_ SDK change, not just CLI source changes)
- **Safety**: Uses `!cancelled()` with individual upstream result checks (`success || skipped`) to avoid running if an upstream job failed

This ensures the CLI's git tag always reflects the latest SDK state, so `wavecraft create` scaffolds projects with up-to-date dependencies.

### Git Conflict Prevention

Since no commits are pushed to `main`, parallel job conflicts for version bumps are no longer possible. Only tag pushes remain, and each job uses a unique tag prefix per package (e.g., `wavecraft-cli-v`, `@wavecraft/core-v`), so tag conflicts cannot occur.

### Secrets Required

| Secret              | Purpose                                                       |
| ------------------- | ------------------------------------------------------------- |
| `GITHUB_TOKEN`      | Commit version bumps + git push (built-in)                    |
| `RELEASE_TAG_TOKEN` | Fine-grained token used only for tag push commands in CD jobs |

Tag push authentication is performed per command using an authenticated remote URL. No persistent git credential rewrite or git config mutation is used.

**Note:** crates.io publishing uses OIDC trusted publishing (no `CARGO_REGISTRY_TOKEN` required).

**Note:** npm publishing uses OIDC trusted publishing (no secret required). Packages are published with `--provenance` for cryptographic attestation.

### Idempotency

The workflow is **idempotent** — running it multiple times won't cause issues:

1. **Already published?** npm cohort jobs skip safely per package when target versions already exist
2. **Publish failed?** Next run detects unpublished version and retries
3. **No changes?** Jobs skip entirely (path filter returns false)

### Workflow Dispatch (npm)

`workflow_dispatch` uses `force-publish-new-version-npm-packages` as the npm force input.

When `force-publish-new-version-npm-packages=true`, cohort preparation still computes the normal target (highest of local/published core/components). If that exact target is already published for **both** `@wavecraft/core` and `@wavecraft/components`, CI auto-bumps the target to the next patch version and publishes that new cohort version. If only one package is already published at the computed target, no auto-bump is performed so the missing package can be published to restore alignment.

### Engine Crate Publish Order

Engine crates have interdependencies and must be published in order:

```
1. wavecraft-protocol  (no deps)
2. wavecraft-macros    (no deps)
3. wavecraft-metering  (no deps)
4. wavecraft-dsp       (depends on protocol, macros)
5. wavecraft-bridge    (depends on protocol)
6. wavecraft-core      (depends on all above)
```

The workflow waits 30 seconds between publishes for crates.io indexing.

### Engine Publish Dependencies

The `publish-engine` job installs Linux system dependencies required by `wavecraft-dev-server` (GTK/GLib/WebKit). This ensures the publish verification step can compile crates that rely on `gobject-sys` and `glib-sys`.
