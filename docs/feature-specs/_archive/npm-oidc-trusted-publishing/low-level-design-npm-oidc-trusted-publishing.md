# Low-Level Design — npm Trusted Publishing (OIDC)

**Status:** Draft  
**Created:** 2026-02-07  
**Author:** Architect Agent

---

## Summary

This design migrates npm publishing in GitHub Actions from token-based authentication to **Trusted Publishing (OIDC)** for the `@wavecraft/*` packages. The goal is to eliminate long‑lived npm tokens, reduce secret rotation overhead, and standardize provenance generation through GitHub OIDC + npm provenance.

---

## Goals

- Use GitHub Actions **OIDC** to authenticate npm publishes for `@wavecraft/core` and `@wavecraft/components`.
- Remove dependency on `NODE_AUTH_TOKEN` / `NPM_TOKEN` for package publishing.
- Maintain provenance generation with `npm publish --provenance`.
- Preserve existing version bump and tagging behavior.

---

## Non‑Goals

- Changing package versioning strategy.
- Refactoring package build steps or output artifacts.
- Altering release cadence or CI trigger conditions.

---

## Current State

The Continuous Deploy workflow uses `actions/setup-node@v4` with `registry-url` and `scope`, and publishes using `npm publish --provenance`. Authentication is currently token‑based, which has caused failures when the token expires or is revoked.

Observed failure mode from recent runs:
- `Access token expired or revoked` leading to 404 on scoped package publish.

---

## Proposed Design

### 1) npm Trusted Publishing Configuration (npm-side)

For each npm package under `@wavecraft/*`:

- Enable **Trusted Publishing** in npm settings.
- Authorize the GitHub repository `RonHouben/wavecraft`.
- Restrict access to the `main` branch and the `continuous-deploy.yml` workflow (or a dedicated publish workflow, if we later split it).

This creates an OIDC trust relationship between npm and GitHub Actions for publishes originating from this workflow.

### 2) GitHub Actions Workflow Adjustments

In `continuous-deploy.yml`:

- Keep `permissions: id-token: write` for `publish-npm-core` and `publish-npm-components`.
- Remove all reliance on `NODE_AUTH_TOKEN` / `NPM_TOKEN`.
- Keep `npm publish --provenance` to generate signed provenance.
- Ensure `actions/setup-node@v4` is configured for the npm registry but **without** token inputs.

**Important:** `npm ci` still needs the registry scope config but does not require publish auth.

### 3) Secrets Hygiene

- Remove any npm publish tokens from repo secrets once OIDC is validated.
- Keep other secrets unchanged (e.g., Cargo tokens for crates.io).

---

## Detailed Workflow Changes (Conceptual)

**Before:**
- `actions/setup-node@v4` configures registry and writes `.npmrc` with token.
- `npm publish` uses token authentication.

**After:**
- `actions/setup-node@v4` configures registry only.
- `npm publish --provenance` uses GitHub OIDC to mint a short‑lived token at publish time.

**Invariant:** `id-token: write` permission must remain for publish jobs.

---

## Failure Modes & Mitigations

| Failure Mode | Cause | Mitigation |
|------------|-------|------------|
| `npm publish` 401/403 | OIDC trust not configured | Verify npm Trusted Publishing settings (repo + workflow) |
| `npm publish` fails with provenance error | Missing `id-token: write` | Ensure workflow permissions include `id-token: write` |
| 404 for scoped package | Package not created or org permissions missing | Ensure package exists and publishing user/org has rights |
| Build succeeds, publish fails | OIDC trust scoped to different workflow | Align workflow name/path with npm settings |

---

## Security Considerations

- OIDC provides short‑lived tokens bound to a specific workflow and ref, reducing blast radius.
- Provenance metadata improves traceability for published artifacts.
- Removing static tokens eliminates secret leakage risk.

---

## Rollout Plan

1) Configure npm Trusted Publishing for both packages.
2) Update `continuous-deploy.yml` to remove token usage and rely on OIDC.
3) Trigger a controlled publish (workflow_dispatch) to validate.
4) Remove token secrets from GitHub once verified.

---

## Testing & Validation

- Perform a manual `workflow_dispatch` on `continuous-deploy.yml` with a no-op version bump (or dev branch test) to confirm OIDC auth succeeds.
- Verify provenance at `https://search.sigstore.dev/` for the published versions.

---

## Open Questions

1) Should npm publishing be moved into a dedicated workflow (e.g., `publish-npm.yml`) to reduce coupling and simplify OIDC scoping?
2) Should we enforce `main` branch protection as an additional guardrail for OIDC publishing?

---

## Documentation

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview  
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions  
- [Roadmap](../../roadmap.md) — Implementation progress
