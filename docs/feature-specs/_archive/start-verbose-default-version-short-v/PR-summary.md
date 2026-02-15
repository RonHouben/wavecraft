## Summary

This PR removes the `--verbose` flag from `wavecraft start` and makes the startup flow consistently emit key operational logs without requiring a verbosity toggle. It also standardizes CLI version behavior by exposing `-v, --version` (instead of `-V`) via explicit clap version flag configuration.

## Changes

- **Engine/DSP**
  - No engine or DSP code changes.

- **UI**
  - No UI package or React component changes.

- **Build/Config**
  - Updated CLI argument parsing in `cli/src/main.rs`:
    - Enabled `propagate_version` and `disable_version_flag` on root clap command.
    - Added explicit global `-v, --version` flag.
    - Removed `verbose` from the `start` subcommand wiring.
  - Refactored start command execution in `cli/src/commands/start.rs` to remove verbose plumbing and always print operational diagnostics/status messages for parameter discovery, cache staleness, and audio startup failures.
  - Updated WebSocket server API and internals in `dev-server/src/ws/mod.rs` to remove the `verbose` constructor parameter and make request/response debug logging unconditional at `debug` level.

- **Documentation**
  - Added this PR summary file under feature specs for traceability.

## Commits

- `19ab802` fix(start): remove verbose flag from Start command execution
- `a320751` fix(start): remove verbose flag and adjust related logging in development commands

## Related Documentation

- No existing feature-spec docs were found for `start-verbose-default-version-short-v` (for example `implementation-plan.md`, `implementation-progress.md`, `low-level-design-*.md`, or `user-stories.md`).

## Testing

- [x] `cargo test --manifest-path cli/Cargo.toml`
- [ ] Build passes: `cargo xtask build`
- [ ] Linting passes: `cargo xtask lint`
- [ ] Tests pass: `cargo xtask test`
- [ ] Manual SDK dev startup verification (`wavecraft start`) in a sample plugin project

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed (`cli/tests/version_flag.rs`)
- [ ] Documentation updated beyond PR summary (if required by feature workflow)
- [ ] No linting errors (`cargo xtask lint`)
