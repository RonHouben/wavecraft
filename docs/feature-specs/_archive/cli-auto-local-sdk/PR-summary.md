## Summary

Auto-detect when the CLI is running from the monorepo source checkout (via `cargo run` or `target/debug/wavecraft`) and automatically use path dependencies instead of git tags. This eliminates the chicken-and-egg problem where `wavecraft create` fails with "failed to find tag" when the git tag doesn't exist yet, and removes the need for the `--local-sdk` flag during SDK development.

**Detection approach:** Runtime binary path inspection — check if the binary lives in a `target/` directory, then walk up the filesystem to find the SDK marker (`engine/crates/wavecraft-nih_plug/Cargo.toml`). Safe fallback to git tags on any failure.

## Changes

- **CLI** (`cli/`):
  - New `sdk_detect.rs` module (169 lines) — `detect_sdk_repo()`, `is_cargo_run_binary()`, `find_monorepo_root()` with 9 unit tests
  - `main.rs` — Added `mod sdk_detect;` registration
  - `create.rs` — 3-tier SDK path resolution: (1) explicit `--local-sdk`, (2) auto-detect, (3) git tags
  - `start.rs` — formatting only (`cargo fmt`)

- **Documentation** (`docs/`):
  - `guides/sdk-getting-started.md` — New "SDK Development" section
  - `architecture/high-level-design.md` — Added `sdk_detect.rs` to CLI module listing, updated diagrams
  - `architecture/coding-standards.md` — Updated test workflow (auto-detection replaces `--local-sdk`)
  - `architecture/agent-development-flow.md` — Same test workflow update
  - `roadmap.md` — Changelog entry, archived feature spec, updated immediate tasks

## Commits

- `feat: Implement auto-detection of local SDK for development mode`
- `docs: Update architecture docs, roadmap, and archive feature spec`

## Related Documentation

- [Implementation Plan](./implementation-plan.md)
- [Implementation Progress](./implementation-progress.md)
- [Test Plan](./test-plan.md) (9/9 TCs passed)
- [QA Report](./QA-report.md) (PASS — 0 Critical/High)

## Testing

- [x] `cargo xtask ci-check` passes (164 engine + 28 UI + 32 CLI tests)
- [x] 9/9 manual test cases passed (auto-detect, fallback, override scenarios)
- [x] QA review: PASS (0 Critical/High issues)
- [x] Architecture docs reviewed and updated

## Checklist

- [x] Code follows project coding standards
- [x] Tests added (9 unit tests in `sdk_detect.rs`)
- [x] Documentation updated (4 architecture/guide files)
- [x] No linting errors (`cargo xtask ci-check`)
- [x] Feature spec archived to `_archive/cli-auto-local-sdk/`
- [x] Roadmap updated with changelog entry
