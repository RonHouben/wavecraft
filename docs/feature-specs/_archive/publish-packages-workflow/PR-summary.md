## Summary

Improve package publishing reliability by moving npm publishes to OIDC/provenance in `continuous-deploy.yml`, adding required Linux deps for CLI dry-run, and documenting the trusted publishing rollout and validation results.

## Changes

- **Engine/DSP**: Update `engine` Cargo metadata and dev-server assets/webview integration for publishing and packaging.
- **UI**: Small VersionBadge tweak to trigger npm publish and validate package flow.
- **Build/Config**: Harden `continuous-deploy.yml` with OIDC publishing, npm auth cleanup, npm CLI update, and Linux GTK/WebKit deps for CLI dry-run.
- **Documentation**: Add/refresh feature specs, implementation progress, test plans, backlog and roadmap entries for crates.io publishing fixes and npm OIDC trusted publishing.

## Commits

- 0b18d05 feat(docs): add implementation plans and progress for npm OIDC trusted publishing and CI-prefixed xtask subcommands
- 7d4cf3a fix(docs): update implementation progress and test plan for npm OIDC trusted publishing validation
- cb22297 chore(cli): release v0.8.3 [skip ci]
- fe9f301 Merge remote-tracking branch 'refs/remotes/origin/fix/publish-packages-workflow' into fix/publish-packages-workflow
- f2d9cbd fix(ci): update npm to latest version in continuous deploy workflow
- 820e1b4 chore(cli): release v0.8.2 [skip ci]
- bcfb00e fix(ci): clear npm auth for OIDC publish
- 4c68aa4 chore(cli): release v0.8.1 [skip ci]
- 830b334 docs(ui): update test plans and version badge
- b689bc0 feat: add CLI `update` command to backlog for updating project dependencies
- ec51034 fix(ci): add webkit2gtk dev package
- 1ee61f9 fix(ci): add libsoup/javascriptcore deps
- f5e686b fix(ci): install gtk deps for cli dry-run
- 8041d2d feat: add test plans for npm OIDC Trusted Publishing and CI workflow validation
- ac4bd9c fix: enhance CI workflow for OIDC trusted publishing and update Cargo.toml metadata
- a7b6f7d fix: update CLI dependencies to version 0.7.3 and enhance publish validation in CI workflow
- 728ab56 feat: implement OIDC Trusted Publishing for npm packages and update CI workflows

## Related Documentation

- [npm OIDC Trusted Publishing](../npm-oidc-trusted-publishing/implementation-plan.md)
- [npm OIDC Implementation Progress](../npm-oidc-trusted-publishing/implementation-progress.md)
- [npm OIDC Test Plan](../npm-oidc-trusted-publishing/test-plan.md)
- [crates.io Publishing Fix](../crates-io-publishing-fix/implementation-plan.md)
- [Roadmap](../../roadmap.md)

## Testing

- [ ] Build passes: `cargo xtask build`
- [ ] Linting passes: `cargo xtask lint`
- [ ] Tests pass: `cargo xtask test`
- [ ] Workflow validation: `continuous-deploy.yml` workflow_dispatch on branch

## Checklist

- [ ] Code follows project coding standards
- [ ] Tests added/updated as needed
- [x] Documentation updated
- [ ] No linting errors (`cargo xtask lint`)
