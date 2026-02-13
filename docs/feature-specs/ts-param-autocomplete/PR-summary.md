## Summary

This PR delivers TypeScript parameter ID autocompletion across the SDK workflow and includes a CI/CD hotfix for npm component sync ordering to prevent `ERESOLVE` version conflicts during release.

Key outcomes:
- Adds generated `ParameterId` typing in plugin UI projects for compile-time safety and IDE autocomplete.
- Wires generation into `wavecraft start` and template setup flow.
- Updates UI/core/components APIs and hooks to consume strongly typed parameter IDs.
- Fixes CD ordering in npm component synchronization to avoid dependency resolution failures.

## Changes

- **Engine/DSP**
  - Updated plugin macro parameter discovery/output logic in `engine/crates/wavecraft-macros/src/plugin.rs`.

- **CLI / Template / Dev Workflow**
  - Added TypeScript parameter ID code generation plumbing (`cli/src/project/ts_codegen.rs`, related project modules).
  - Extended startup and template handling (`cli/src/commands/start.rs`, `cli/src/template/mod.rs`).
  - Updated dev-template setup script and generated template TypeScript config/path behavior (`scripts/setup-dev-template.sh`, `sdk-template/ui/tsconfig.json`, `sdk-template/ui/src/App.tsx`, `sdk-template/.gitignore`).
  - Minor dev-server rebuild behavior/test updates (`dev-server/src/reload/rebuild.rs`, `dev-server/tests/reload_cancellation.rs`).

- **UI Packages**
  - Added/propagated typed parameter identifiers in `@wavecraft/core` and consumers:
    - `ui/packages/core/src/types/parameters.ts`
    - `ui/packages/core/src/ipc/ParameterClient.ts`
    - `ui/packages/core/src/hooks/useParameter.ts`
    - `ui/packages/core/src/hooks/useAllParameters.ts`
    - `ui/packages/core/src/index.ts`
    - `ui/packages/components/src/ParameterSlider.tsx`
    - `ui/packages/components/src/ParameterToggle.tsx`

- **Build/CI**
  - Fixed CD workflow ordering for npm components in `.github/workflows/continuous-deploy.yml` to prevent `ERESOLVE` during publish/sync.

- **Documentation**
  - Added and updated architecture, workflows, guides, backlog, and feature-spec docs for TS parameter autocomplete and related process changes.

## Commits

- `c2fff88` feat: enhance SDK with TypeScript parameter ID autocompletion
- `cc1fc4a` docs: add PR summary for CD hotfix
- `65b2a3a` fix(cd): reorder npm components sync to avoid npm version ERESOLVE

## Related Documentation

- [User Stories](./user-stories.md)
- [Implementation Plan (TSConfig Paths)](./implementation-plan-tsconfig-paths.md)
- [Test Plan](./test-plan.md)

## Testing

- [x] Linting/formatting/typecheck pass via repository CI check (`cargo xtask ci-check --fix`)
- [x] Automated tests pass via repository CI check (`cargo xtask ci-check --fix`)
- [x] Build/packaging flow verified in CI-check pipeline steps
- [ ] Manual UI verification in browser/WebView
- [ ] DAW/plugin runtime verification (if required for release scope)

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No linting/type errors in local CI-check run
