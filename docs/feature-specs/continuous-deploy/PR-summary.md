## Summary

Fix Continuous Deploy publishing failures by installing required Linux GTK/GLib/WebKit dependencies in the engine publish job. Add QA/test documentation and align the CI guide with the current workflow filters, secrets, and dependency expectations.

## Changes

- **Build/Config**: install Linux GTK/GLib/WebKit dependencies in `publish-engine` for crates.io publishing.
- **Documentation**: add test plan and QA report for the deploy fix; update the CI guide to match current workflow filters, secrets, and dependency notes.

## Commits

- 958a63a docs: add QA report for continuous deployment fix and update CI pipeline documentation
- cf8a82d feat(tests): add test plan for continuous deployment engine dependency validation
- bf9314e fix(ci): add installation of Linux system dependencies for continuous deployment

## Related Documentation

- [Test Plan](../continuous-deploy-fix/test-plan.md)
- [QA Report](../continuous-deploy-fix/QA-report.md)

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
