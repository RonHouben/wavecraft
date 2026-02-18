## Summary

Add fallback embedded UI assets for `wavecraft-nih_plug` and strengthen template validation around `wavecraft start` in git-tag/local-sdk scenarios. This improves resilience when dist assets are unavailable and tightens CI validation for generated projects.

## Changes

- **Engine/DSP**
  - Added fallback UI asset files under `engine/crates/wavecraft-nih_plug/assets/ui-dist/` (`index.html`, CSS, JS).
  - Updated `engine/crates/wavecraft-nih_plug/src/editor/assets.rs` to embed/validate fallback assets in tests.
  - Updated `engine/crates/wavecraft-nih_plug/src/editor/mod.rs` for conditional editor compilation with `_param-discovery` feature.
  - Updated `engine/crates/wavecraft-nih_plug/src/prelude.rs` and `Cargo.toml` for related feature/export wiring.
  - Refactored `engine/xtask/src/commands/validate_template.rs` to improve template validation flow (local-sdk and git-source modes).

- **UI**
  - Updated UI package dependency metadata in:
    - `ui/package.json`
    - `ui/package-lock.json`
    - `ui/packages/core/package.json`
    - `ui/packages/components/package.json`
    - `sdk-template/ui/package.json`

- **Build/Config**
  - Updated `.github/workflows/template-validation.yml` with expanded validation checks.
  - Updated `sdk-template/engine/Cargo.toml.template` for feature/dependency alignment.
  - Adjusted `.gitignore` as part of template/asset validation changes.

- **Documentation**
  - Added/updated feature planning docs in:
    - `docs/feature-specs/wavecraft-start-git-tag-assets/implementation-plan.md`

## Commits

- `e2d7c96` test: enhance fallback asset validation to check for CSS and JS files
- `1721207` fix(dependencies): update @wavecraft/core and @wavecraft/components to version 0.7.31
- `226e3b7` feat: add fallback UI assets and improve validation workflow

## Related Documentation

- [Implementation Plan](./implementation-plan.md)

## Testing

- [ ] Build passes: `cargo xtask build`
- [ ] Linting passes: `cargo xtask lint`
- [ ] Tests pass: `cargo xtask test`
- [ ] Template validation passes for local-sdk/git-source generated projects

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No unrelated code changes included in this PR summary update
