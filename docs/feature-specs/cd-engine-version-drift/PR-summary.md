## Summary

Fixes CI/CD engine crate version drift that could cause `publish-cli` failures when crate availability lagged behind exported engine version metadata (e.g., `wavecraft-protocol` lagging `wavecraft-core`).

This change makes engine publishing version-aware and availability-gated before downstream jobs consume engine version outputs.

## Changes

- **Build/Config** (`.github/workflows/continuous-deploy.yml`)
  - Updated `publish-engine` to verify all required engine crates are available on crates.io before exporting `ENGINE_VERSION`.
  - Added downstream guard so `publish-cli` and `publish-dev-server` require a verified engine version when `engine/` or `cli/` scope applies.
  - Prevented drift scenarios where downstream publish jobs proceeded on an optimistic version while one or more engine crates were not yet resolvable.

- **CI Scripts** (`scripts/ci/wait-for-crate.sh`)
  - Improved timeout diagnostics to make crate propagation failures easier to triage.
  - Added clearer failure context to reduce ambiguity during publish incidents.

## Commits

- `6948822` fix(cd): enhance version verification for engine crates in CI workflow

## Related Documentation

- Feature folder: `docs/feature-specs/cd-engine-version-drift/`
- PR summary: `docs/feature-specs/cd-engine-version-drift/PR-summary.md`

## Testing

- Validation scope covered by CI/CD workflow logic and crate-availability wait script behavior.
- Tester approval received after one follow-up patch.
- QA approval received after one follow-up patch.

## Risks

- **Residual risk:** crates.io propagation timing can still vary, potentially extending publish duration.
- **Mitigation:** explicit crate availability checks and improved timeout diagnostics reduce false-success cascades and speed up incident diagnosis.

## Rollout Notes

- No runtime/plugin behavior changes; this is CI/CD workflow hardening only.
- Applies to release/publish paths where engine and/or CLI scopes are detected.
- Safe to roll out immediately; expected effect is fewer publish retries and fewer downstream failures caused by version drift.

## Checklist

- [x] Changes are scoped to CI/CD drift fix only
- [x] No roadmap edits
- [x] No archived feature spec edits
- [x] Tester and QA approvals included
