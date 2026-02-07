# Implementation Progress — npm OIDC Trusted Publishing

## Status

- [ ] Configure npm Trusted Publishing for `@wavecraft/core`
- [ ] Configure npm Trusted Publishing for `@wavecraft/components`
- [x] Remove token-based npm auth from `continuous-deploy.yml`
- [x] Ensure `id-token: write` permissions for npm publish jobs
- [ ] Validate OIDC publish with `workflow_dispatch`
- [ ] Remove npm token secrets after validation

## Notes

- npm publish jobs now explicitly clear `NODE_AUTH_TOKEN`/`NPM_TOKEN` and disable `always-auth` to enforce OIDC.
