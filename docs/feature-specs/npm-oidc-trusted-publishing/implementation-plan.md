# Implementation Plan: npm OIDC Trusted Publishing

## Overview

Migrate npm publishing for `@wavecraft/core` and `@wavecraft/components` to GitHub Actions Trusted Publishing (OIDC). This plan updates npm-side trust configuration, removes token-based auth from CI, and validates provenance-based publishes while maintaining existing workflow behavior.

## Requirements

- npm publishing must authenticate via OIDC (no long‑lived tokens).
- `publish-npm-core` and `publish-npm-components` must succeed using `npm publish --provenance`.
- Workflow should continue to run on `main` and `workflow_dispatch`.
- Secrets cleanup after validation (remove npm token).

## Architecture Changes

- npm org/package settings: enable Trusted Publishing for `@wavecraft/*`.
- `.github/workflows/continuous-deploy.yml`: remove token usage for npm publishes and rely on OIDC.
- Repo secrets: remove npm token once verified.

## Implementation Steps

### Phase 1: npm Trusted Publishing setup
1. **Enable Trusted Publishing for @wavecraft/core** (npm settings)
   - Action: Configure GitHub OIDC trust for `RonHouben/wavecraft` and `continuous-deploy.yml` on `main`.
   - Why: Allows OIDC-based publishes.
   - Dependencies: None
   - Risk: Medium (misconfiguration blocks publishes)

2. **Enable Trusted Publishing for @wavecraft/components** (npm settings)
   - Action: Apply same OIDC trust settings as `@wavecraft/core`.
   - Why: Consistency across packages.
   - Dependencies: Step 1
   - Risk: Medium

### Phase 2: Workflow updates
3. **Remove token-based auth from npm publish jobs** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Remove `NODE_AUTH_TOKEN` usage and any secret-based npm auth wiring.
   - Why: Use OIDC only.
   - Dependencies: Phase 1
   - Risk: Low/Medium

4. **Confirm OIDC permissions are set** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Ensure `permissions: id-token: write` is present for `publish-npm-core` and `publish-npm-components`.
   - Why: Required for OIDC token minting.
   - Dependencies: Step 3
   - Risk: Low

5. **Keep provenance publishing enabled** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Ensure `npm publish --provenance` remains for both packages.
   - Why: Generate signed provenance.
   - Dependencies: Step 3
   - Risk: Low

### Phase 3: Validation & cleanup
6. **Trigger a controlled publish test** (Workflow: `continuous-deploy.yml` via `workflow_dispatch`)
   - Action: Run the workflow in a safe state (e.g., with a patch bump or test version).
   - Why: Validate OIDC publish path end‑to‑end.
   - Dependencies: Phase 2
   - Risk: Medium

7. **Remove npm token secret** (Repo settings)
   - Action: Delete `NPM_TOKEN` / `NODE_AUTH_TOKEN` secrets after successful validation.
   - Why: Remove unused secrets to reduce risk.
   - Dependencies: Step 6
   - Risk: Low

## Testing Strategy

- **Publish validation**: successful `publish-npm-core` and `publish-npm-components` jobs using OIDC.
- **Provenance check**: verify the published versions appear in Sigstore transparency log.

## Risks & Mitigations

- **Risk**: npm OIDC trust scoped to wrong workflow or branch.
  - **Mitigation**: Ensure npm Trusted Publishing config matches `continuous-deploy.yml` on `main`.
- **Risk**: Publish failure due to missing `id-token: write`.
  - **Mitigation**: Explicitly set permissions in workflow jobs.
- **Risk**: Hidden reliance on token-based auth for `npm ci`.
  - **Mitigation**: Validate installs without tokens; keep registry config only.

## Success Criteria

- [ ] `@wavecraft/core` publish succeeds via OIDC.
- [ ] `@wavecraft/components` publish succeeds via OIDC.
- [ ] No npm tokens are required or present in repo secrets.
- [ ] Provenance is generated and visible for both packages.
