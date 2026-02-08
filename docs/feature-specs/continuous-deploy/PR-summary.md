## Summary

Fix Continuous Deploy publishing failures by installing required Linux GTK/GLib/WebKit dependencies in the engine publish job and avoiding protected-branch pushes by opening PRs for version bumps. Update CI documentation to reflect the current workflow filters, secrets, and dependency expectations.

## Changes

- **Build/Config**: ensure engine publish installs GTK/GLib/WebKit dependencies; avoid direct pushes to protected `main` by creating PRs for version bumps.
- **Documentation**: align CI guide with current path filters, secrets, and dependency notes.

## Commits

- 6bd0365 fix(ci): avoid protected branch pushes in deploy
- 67c50c5 docs: add PR summary for continuous deploy fix
- 958a63a docs: add QA report for continuous deployment fix and update CI pipeline documentation
- cf8a82d feat(tests): add test plan for continuous deployment engine dependency validation
- bf9314e fix(ci): add installation of Linux system dependencies for continuous deployment

## Related Documentation

- [Test Plan](../_archive/continuous-deploy-fix/test-plan.md)
- [QA Report](../_archive/continuous-deploy-fix/QA-report.md)

## Testing

- [ ] Build passes: `cargo xtask build`
- [x] Linting passes: `cargo run -p xtask -- ci-check`
- [x] Tests pass: `cargo run -p xtask -- ci-check`
- [ ] Manual UI verification
- [ ] Audio processing verification

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No linting errors (`cargo run -p xtask -- ci-check`)
