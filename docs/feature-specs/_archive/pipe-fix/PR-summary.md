## Summary

Switch npm publishing from token-based authentication to OIDC trusted publishing. This eliminates the need for the `NPM_TOKEN` secret and adds cryptographic provenance attestation to published packages.

## Changes

### CI/CD
- Remove `NODE_AUTH_TOKEN` environment variable from npm publish steps
- Add `--provenance` flag to `npm publish` commands for cryptographic attestation
- Update permission comments to accurately describe OIDC usage

### Documentation
- Update `docs/guides/ci-pipeline.md` secrets table to remove `NPM_TOKEN`
- Add note about OIDC trusted publishing and provenance

### Package Versions
- Bump `@wavecraft/core` to `0.7.2`
- Bump `@wavecraft/components` to `0.7.2`

## Commits

- `a873705` ci: switch npm publishing to OIDC trusted publisher
- `434bf57` fix: enhance npm publishing with provenance support in CI workflow

## Benefits

1. **No secrets to manage** — Authentication via GitHub OIDC, not stored tokens
2. **Provenance attestation** — Users can verify packages were built from this repo via `npm audit signatures`
3. **Security** — Tokens can leak; OIDC tokens are short-lived and scoped

## Prerequisites

npm.js trusted publisher must be configured for both packages:
- `@wavecraft/core` ✅
- `@wavecraft/components` ✅

## Testing

- [ ] Workflow triggers on merge to main
- [ ] Both npm packages publish successfully without `NPM_TOKEN`
- [ ] Packages include provenance attestation
