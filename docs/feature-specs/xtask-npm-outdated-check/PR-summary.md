## Summary

Adds a new `xtask` command to check for outdated npm packages across the Wavecraft monorepo, so dependency drift can be detected quickly from a single Rust-driven workflow.

## Changes

### Build / Tooling

- `engine/xtask/src/commands/npm_updates.rs` — Adds the new command implementation for npm outdated checks.
- `engine/xtask/src/commands/mod.rs` — Registers the new command module.
- `engine/xtask/src/main.rs` — Wires CLI argument parsing and dispatch for the new command.
- `engine/xtask/src/tests.rs` — Adds/updates tests covering command behavior.

### Documentation / Repository housekeeping

- `docs/feature-specs/_archive/oscilloscope-v1/PR-summary.md` — Path move in branch history (no content changes).

## Commits

- `117cc24` feat(npm-updates): add command to check for outdated npm packages in monorepo

## Testing

Previously reported validation from the implementation phase was used for this PR body completion step.

Additional note for this change set:

- This final step only adds PR documentation (`PR-summary.md`) and does not modify runtime code.

## Checklist

- [x] PR summary includes scope and implementation details
- [x] Files touched and purpose documented
- [x] Testing context captured
- [x] Ready to create PR against `main`
