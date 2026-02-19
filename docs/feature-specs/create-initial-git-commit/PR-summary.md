## Summary

Improve new-project UX by initializing git repositories with an initial commit when scaffolding via `wavecraft create` (and legacy `new` path), so users start from a clean working tree instead of seeing many untracked files immediately.

Also updates the getting-started guide to document the default behavior and the `--no-git` opt-out.

## Changes

- **Build/CLI**
  - Updated git initialization flow in `create` command to:
    - run `git init`
    - stage scaffolded files with `git add .`
    - create initial commit (`Initial commit`)
  - Preserved graceful fallback behavior when git commands fail (warn and continue).
  - Applied same behavior to legacy `new` command for consistency.

- **Testing**
  - Added integration test coverage for `wavecraft create`:
    - default mode creates `.git`, includes initial commit, and leaves clean `git status`
    - `--no-git` mode skips git initialization

- **Documentation**
  - Updated getting-started guide with explicit note that `wavecraft create` now performs git init + initial commit by default, and that `--no-git` skips both.

## Commits

- `feat: initialize git repository and create initial commit in create and new commands`
- `docs: clarify create default git init + initial commit behavior`

## Related Documentation

- [SDK Getting Started](../../guides/sdk-getting-started.md)

## Testing

- [x] CLI targeted tests pass: `cargo test --manifest-path cli/Cargo.toml --test create_command`
- [x] CLI test suite passes: `cargo test --manifest-path cli/Cargo.toml`
- [x] CLI lint/format passes:
  - `cargo fmt --manifest-path cli/Cargo.toml`
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`
- [x] Repo checks pass: `cargo xtask ci-check --fix`
- [x] Docs links pass: `scripts/check-links.sh`

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No linting/type-check/test failures in validation runs
- [ ] PO-owned roadmap changelog row added in `docs/roadmap.md` (requested handoff; pending PO-only edit)
