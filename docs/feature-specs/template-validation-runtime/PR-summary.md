## Summary

Optimize the template validation pull request workflow to run faster and more reliably by improving cache usage, tightening what changes trigger validation, and strengthening startup checks. This also aligns multiple custom agent configurations with VS Code tool integration updates used by the workflow context.

## Changes

- **Build/Config**
  - Updated `.github/workflows/template-validation.yml` to optimize template validation CI behavior.
  - Added/improved npm caching behavior in the template validation flow.
  - Refined PR path triggers so template validation runs for additional relevant changes.
  - Improved engine start validation checks in the workflow.
- **Agent Configuration**
  - Updated several files under `.github/agents/` to include VS Code tool integration adjustments and related agent configuration updates.
- **Documentation**
  - Added this PR summary at `docs/feature-specs/template-validation-runtime/PR-summary.md`.

## Commits

- `a854a68` fix: update template validation workflow to include additional paths for PR checks
- `50f9ba9` fix: enhance template validation workflow by adding npm caching and improving engine start checks
- `d1955aa` feat: add vscode tool integration to multiple agent configurations

## Related Documentation

No existing feature-spec implementation docs were found for `template-validation-runtime`.

## Testing

- [ ] Build passes: `cargo xtask build`
- [ ] Linting passes: `cargo xtask lint`
- [ ] Tests pass: `cargo xtask test`

## Checklist

- [x] Code follows project coding standards
- [x] Documentation updated
- [ ] Tests added/updated as needed
- [ ] No linting errors (`cargo xtask lint`)
