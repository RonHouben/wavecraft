# Canonical SDK Template (`sdk-template/`)

This directory is the **single source of truth** for the Wavecraft plugin scaffold.

It is used for two purposes:

1. **CLI generation** (`wavecraft create`) — files in `sdk-template/` are embedded into the CLI binary.
2. **SDK development mode** (`cargo xtask dev` from this repository) — the dev server targets `sdk-template/engine` and `sdk-template/ui`.

## Important

Template files ending in `.template` (for example `Cargo.toml.template`) contain placeholders and are intended for CLI generation.

Before using this folder directly in SDK development mode, run:

`./scripts/setup-dev-template.sh`

That script generates concrete files, fills template variables with local development defaults, switches SDK dependencies to local path dependencies, and installs UI dependencies.

## Layout

- `Cargo.toml.template` — workspace manifest template
- `engine/` — plugin engine template
- `ui/` — React UI template
- `LICENSE`, `.gitignore`, `README.md` — project-level template assets

## Related docs

- `docs/architecture/high-level-design.md`
- `docs/architecture/development-workflows.md`
- `docs/architecture/sdk-architecture.md`
- `docs/guides/sdk-getting-started.md`
