# Low-Level Design — crates.io Publishing Fix

**Status:** Draft  
**Created:** 2026-02-07  
**Author:** Architect Agent

---

## Summary

This design fixes crates.io publishing for the Wavecraft CLI by enforcing valid versioned dependencies and ensuring all CLI runtime dependencies are publishable. It also tightens the Continuous Deploy workflow to prevent publish failures caused by unpublished or path-only dependencies.

---

## Goals

- Ensure `cargo publish` for the CLI succeeds on crates.io.
- Enforce explicit `version` fields for all CLI dependencies, even when `path` is used.
- Make all CLI runtime dependencies publishable on crates.io (or feature‑gate them if optional).
- Preserve existing release automation and tagging behavior.

---

## Non‑Goals

- Changing the CLI release cadence or semantic versioning policy.
- Refactoring CLI runtime behavior or internal architecture.
- Replacing `cargo publish` with an alternative publish tool.

---

## Current State

The CLI (`cli/Cargo.toml`) depends on engine crates via **path-only** dependencies. crates.io rejects publishing when dependencies do not have explicit versions. Additionally, the CLI depends on `wavecraft-dev-server`, which is currently `publish = false`, making it unavailable on crates.io.

Observed failure mode:
- `all dependencies must have a version requirement specified when publishing`
- Specifically: `dependency 'wavecraft-bridge' does not specify a version`

---

## Proposed Design

### 1) Add Version Requirements to CLI Dependencies

In `cli/Cargo.toml`, each path dependency must include a `version` field matching the crate’s published version.

**Required dependencies:**
- `wavecraft-protocol`
- `wavecraft-bridge`
- `wavecraft-metering`
- `wavecraft-dev-server`

Example shape:
```toml
[dependencies.wavecraft-bridge]
path = "../engine/crates/wavecraft-bridge"
version = "0.7.3"
```

### 2) Publish `wavecraft-dev-server`

Because the CLI links to `wavecraft-dev-server` at runtime, the crate must be publishable:

- Set `publish = true` in `engine/crates/wavecraft-dev-server/Cargo.toml`.
- Ensure its version aligns with the engine workspace version (`0.7.x`).
- Include it in `cargo ws publish` (automatic when publishable).

### 3) Workflow Guardrails

Update `continuous-deploy.yml` to:

- Verify publishability of CLI before attempting `cargo publish`.
- Fail fast if any CLI dependency is `publish = false` or missing version.
- Optionally run `cargo publish --dry-run` for the CLI in CI.

---

## Failure Modes & Mitigations

| Failure Mode | Cause | Mitigation |
|------------|-------|------------|
| `cargo publish` fails: missing version | Path-only dependency | Add `version` fields for all CLI deps |
| `cargo publish` fails: dependency not on crates.io | `publish = false` crate | Make crate publishable or feature‑gate it |
| Tag/commit mismatch | Version bump without tag | Keep existing tag creation step |

---

## Security Considerations

- No additional secrets required.
- Publishing remains gated by `CARGO_REGISTRY_TOKEN`.

---

## Rollout Plan

1) Add `version` fields to all CLI path dependencies.
2) Set `publish = true` for `wavecraft-dev-server`.
3) Run `cargo publish --dry-run` locally for CLI to validate.
4) Re-run Continuous Deploy to validate publish success.

---

## Testing & Validation

- Local: `cargo publish --manifest-path cli/Cargo.toml --dry-run`
- CI: observe `publish-cli` job success on merge to `main`.

---

## Open Questions

1) Should `wavecraft-dev-server` remain a hard dependency, or be feature‑gated to allow publishing without it?
2) Should the CLI publish step include a `--dry-run` stage before real publish in CI?

---

## Documentation

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview  
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions  
- [Roadmap](../../roadmap.md) — Implementation progress
