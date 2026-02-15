## Summary

Improve version parsing logic in the continuous deploy workflow to handle extracted version values more reliably and avoid deploy-time parsing failures.

## Changes

- **Build/Config**
  - Updated `.github/workflows/continuous-deploy.yml`.
  - Refined version extraction/parsing steps used by the deployment pipeline.
  - Kept behavior equivalent while improving robustness of shell parsing logic.

## Commits

- `9e386e5` fix(actions): improve version parsing in continuous deploy workflow

## Related Documentation

- No existing feature-spec design/plan documents were found for this bugfix branch.

## Testing

- [x] Full repo checks pass: `cargo xtask ci-check --full`
- [x] Version alignment validation script executed successfully
- [ ] Validate workflow behavior in next GitHub Actions run (post-merge)

## Checklist

- [x] Code follows project coding standards
- [x] Tests/checks updated and passing where applicable
- [x] Documentation artifact added for PR workflow
- [x] No linting/test failures observed in local CI check
