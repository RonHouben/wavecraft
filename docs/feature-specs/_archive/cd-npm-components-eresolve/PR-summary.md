# PR Summary: Fix CD npm-components ERESOLVE

## Summary
Fix the `publish-npm-components` CD pipeline job that fails with an npm ERESOLVE peer dependency conflict.

## Problem
The "Sync @wavecraft/core dependency version" step updated the peer dependency to `^0.7.23` before `npm version` ran. Since the local workspace only had `@wavecraft/core@0.7.5`, npm's dependency resolver rejected the version bump.

## Fix
Reordered the steps in the `publish-npm-components` job so that:
1. Version determination and auto-bump happen first (peer dep still `^0.7.0`, satisfied by local `0.7.5`)
2. Sync @wavecraft/core dependency version happens after bump (before build/publish)
3. Commit step captures both changes (same `package.json` file)

## Changes
- `.github/workflows/continuous-deploy.yml` — Moved "Sync @wavecraft/core dependency version" step after "Auto-bump patch version" in the `publish-npm-components` job

## Testing
- CD pipeline re-run after merge should confirm the fix
