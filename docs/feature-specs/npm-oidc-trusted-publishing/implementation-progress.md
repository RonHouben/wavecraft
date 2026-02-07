# Implementation Progress — npm OIDC Trusted Publishing

## Status

- [x] Configure npm Trusted Publishing for `@wavecraft/core`
- [x] Configure npm Trusted Publishing for `@wavecraft/components`
- [x] Remove token-based npm auth from `continuous-deploy.yml`
- [x] Ensure `id-token: write` permissions for npm publish jobs
- [ ] Validate OIDC publish with `workflow_dispatch`
- [ ] Remove npm token secrets after validation

## Notes

- npm publish jobs now explicitly clear `NODE_AUTH_TOKEN`/`NPM_TOKEN` and disable `always-auth` to enforce OIDC.
- Branch validation run (21779095466) succeeded for `publish-npm-components`; `publish-npm-core` was skipped due to no changes.
- Main validation run (21779071434) failed because token auth was still injected on `main`.
