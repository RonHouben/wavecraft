## Summary

Reorganize the CLI plugin template from `cli/plugin-template/` to `cli/sdk-templates/new-project/react/` to support multiple template variants and establish a scalable SDK template architecture.

This change:
- Moves the plugin template to `cli/sdk-templates/new-project/react/`
- Updates the CLI to use `include_dir!` macro for embedding templates
- Removes the legacy `build.rs` pre-build copy step
- Fixes template variable substitution in generated projects
- Updates SDK version from v0.7.0 to v0.7.1

## Changes

- **CLI**: 
  - Removed `build.rs` pre-build copy step
  - Updated `template/mod.rs` to use `include_dir!` at new path
  - Fixed SDK version default in `main.rs` (v0.7.0 → v0.7.1)
  
- **Template**:
  - Relocated from `cli/plugin-template/` to `cli/sdk-templates/new-project/react/`
  - Fixed template variables in `xtask/src/main.rs` (`{{plugin_name}}`, `{{plugin_name_snake}}`)
  - Removed `package-lock.json` from template
  
- **CI/CD**:
  - Updated `continuous-deploy.yml` path filter to `cli/sdk-templates/**`
  - Removed template copy step from `cli-release.yml`

- **Documentation**:
  - Updated `high-level-design.md` with new template path
  - Updated `ci-pipeline.md` guide
  - Full feature spec with LLD, implementation plan, test plan, and QA report

## Commits

- `227bd4f` docs: archive template-relocation-docs and update roadmap changelog
- `0b01f88` docs: mark LLD as approved after successful implementation
- `b03847f` docs: add QA report for template reorganization feature
- `2620d97` fix: resolve template and CLI issues found during testing
- `73bdf4d` test: add test plan for template reorganization
- `5b07821` refactor: reorganize template from plugin-template to sdk-templates/new-project/react
- `c2dce43` feat(docs): add implementation plan and progress tracker for template reorganization
- `ac055ff` feat(docs): add low-level design document for template reorganization

## Related Documentation

- [Low-Level Design](./low-level-design-template-relocation-docs.md)
- [Implementation Plan](./implementation-plan.md)
- [Implementation Progress](./implementation-progress.md)
- [Test Plan](./test-plan.md) - 10/10 tests PASS
- [QA Report](./QA-report.md) - PASS with 0 issues

## Testing

- [x] Build passes: `cargo xtask bundle`
- [x] Linting passes: `cargo xtask lint`
- [x] CLI scaffolds project: `wavecraft new test-plugin`
- [x] Generated project builds: VST3 + CLAP bundles created
- [x] Template variables resolve correctly
- [x] Manual verification complete (10/10 test cases)

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No linting errors (`cargo xtask lint`)
- [x] QA review passed
- [x] Architect review approved
