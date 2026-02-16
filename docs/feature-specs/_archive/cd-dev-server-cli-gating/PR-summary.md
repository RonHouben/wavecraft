# PR Summary: Publish dev-server gating compatibility fix

## What changed

This PR fixes CD workflow gating for `publish-dev-server` so the job remains compatible when changes originate from either engine or CLI areas.

### Workflow update

- Updated `.github/workflows/continuous-deploy.yml`
- Expanded the `publish-dev-server` gate logic to treat both **engine-related** and **CLI-related** changes as valid triggers for dev-server publishing.

## Why

The previous gating behavior could skip `publish-dev-server` in valid compatibility scenarios. This change aligns publish conditions with actual dependency and release-coupling behavior.

## Commits included

- `95ce920` fix(continuous-deploy): enhance dev-server publish conditions to include engine and CLI changes

## Files changed

- `.github/workflows/continuous-deploy.yml`
