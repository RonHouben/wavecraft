## Summary

Simplify GitHub Actions workflow triggers to eliminate redundant CI runs when PRs are merged to main.

**Problem:** Currently, CI and Template Validation workflows run twice for every merged PR:
1. On the PR (required quality gate) ✅
2. Again after merge to main (redundant) ❌

**Solution:** Remove `push` triggers from CI and Template Validation — they now only run on PRs. Added `workflow_dispatch` for manual runs when needed.

**Impact:**
- ~10-14 CI minutes saved per merge
- ~280 CI minutes/month saved (at 20 merges)
- 66% reduction in workflows triggered on merge

## Changes

### Workflow Configuration
- `.github/workflows/ci.yml` — Removed `push.branches: [main]`, added `workflow_dispatch`
- `.github/workflows/template-validation.yml` — Removed `push.branches: [main]`, added `workflow_dispatch`

### Documentation
- `docs/guides/ci-pipeline.md` — Updated Triggers section with new behavior
- `docs/architecture/high-level-design.md` — Updated CI/CD Pipelines section
- `docs/roadmap.md` — Added changelog entry

### Feature Spec (Archived)
- `docs/feature-specs/_archive/ci-workflow-simplification/` — Low-level design, implementation plan, test plan

## Commits

- `f5bac9d` docs: archive ci-workflow-simplification feature spec
- `c0008b8` docs: update roadmap with CI workflow simplification changelog entry
- `6667439` docs: update implementation progress with test results and arch doc updates
- `9251b6a` docs: update architectural documentation for CI trigger changes
- `9487777` test: add test plan for CI workflow simplification
- `dd96abb` ci: simplify workflow triggers to reduce redundant runs

## Testing

### Automated (via PR)
- [x] CI workflow triggered on PR (TC-001) ✅
- [x] Template Validation triggered on PR (TC-002) ✅

### To Verify After Merge (TC-003)
- [ ] Only Continuous Deploy runs on merge (this validates the feature)
- [ ] CI does NOT run on merge
- [ ] Template Validation does NOT run on merge

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated (test-plan.md)
- [x] Documentation updated (ci-pipeline.md, high-level-design.md)
- [x] Architectural docs reviewed
- [x] Roadmap updated with changelog entry
- [x] Feature spec archived
