# CI/CD Pipeline

This document describes the CI/CD pipeline architecture for Wavecraft.

## Overview

The CI pipeline runs on every push to `main` and on all pull requests. It consists of two independent pipelines that run in **parallel**:

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
│  └────┬────┘                         └───────┬──────┘                       │
│       │                                      │                              │
│       │                                      ▼                              │
│       │                              ┌─────────────┐                        │
│       │                              │ test-engine │                        │
│       │                              │ ~1min       │                        │
│       │                              └──────┬──────┘                        │
│       │                                     │                               │
│       └─────────────┬───────────────────────┘                               │
│                     ▼                                                       │
│             ┌──────────────┐                                                │
│             │ build-plugin │  (main branch only)                            │
│             │ ~5min        │                                                │
│             └──────────────┘                                                │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Jobs

### UI Pipeline

| Job | Runner | Duration | Description |
|-----|--------|----------|-------------|
| **check-ui** | ubuntu-latest | ~20s | Prettier → ESLint → TypeScript type-check |
| **test-ui** | ubuntu-latest | ~15s | Vitest unit tests |

### Engine Pipeline

| Job | Runner | Duration | Description |
|-----|--------|----------|-------------|
| **prepare-engine** | ubuntu-latest | ~4min | Build UI dist + compile with clippy |
| **check-engine** | ubuntu-latest | ~30s | `cargo fmt --check` → `cargo clippy -D warnings` |
| **test-engine** | ubuntu-latest | ~1min | `cargo test --workspace` |
| **build-plugin** | macos-latest | ~5min | Bundle VST3/CLAP + ad-hoc signing (main only) |

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

| Cache | Scope | What's Cached |
|-------|-------|---------------|
| npm cache | Cross-run | Downloaded packages from npm registry |
| apt cache | Cross-run | Linux system dependencies (GTK, WebKit) |
| Cargo cache | Cross-run | Crates.io registry + compiled dependencies |
| Artifacts | Within-run | `ui/dist` + `engine/target/debug` |

## Artifacts

### Within-Run Artifacts (1 day retention)

| Artifact | Source | Used By |
|----------|--------|---------|
| `ui-dist` | prepare-engine | check-engine, test-engine, build-plugin |
| `engine-target` | prepare-engine | check-engine, test-engine |

### Release Artifacts (30 day retention)

| Artifact | Description |
|----------|-------------|
| `wavecraft-vst3-adhoc-signed` | VST3 plugin bundle (ad-hoc signed) |
| `wavecraft-clap-adhoc-signed` | CLAP plugin bundle (ad-hoc signed) |

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

| Platform | Usage | Cost |
|----------|-------|------|
| `ubuntu-latest` | All checks and tests | Cheapest |
| `macos-latest` | Final plugin build only | More expensive |

Using Ubuntu for most jobs significantly reduces CI costs while macOS is only used for the final build (which requires native toolchain for proper signing).

## Triggers

```yaml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
```

- **Pull Requests:** Full pipeline except `build-plugin`
- **Main branch:** Full pipeline including `build-plugin`

## Local Testing

The CI pipeline can be tested locally using `act` and a custom Docker image with all dependencies pre-installed.

### Quick Start

```bash
# Build the custom image (one-time)
docker build --platform linux/amd64 -t wavecraft-ci:latest \
    .github/skills/run-ci-pipeline-locally/

# Run a specific job
act -j check-engine -W .github/workflows/ci.yml \
    --container-architecture linux/amd64 \
    -P ubuntu-latest=wavecraft-ci:latest \
    --pull=false \
    --artifact-server-path /tmp/act-artifacts
```

**Note:** The `--artifact-server-path` flag enables local artifact upload/download between jobs.

### What Can Be Tested Locally

| Job | Local Testing |
|-----|---------------|
| check-ui | ✅ Works |
| test-ui | ✅ Works |
| prepare-engine | ✅ Works |
| check-engine | ✅ Works |
| test-engine | ✅ Works |
| build-plugin | ❌ Requires macOS |

For detailed local testing instructions, see the [Run CI Pipeline Locally skill](/.github/skills/run-ci-pipeline-locally/SKILL.md).

## Related Documentation

- [Coding Standards](../architecture/coding-standards.md) — Code conventions including linting rules
- [macOS Signing Guide](./macos-signing.md) — Plugin signing and notarization
