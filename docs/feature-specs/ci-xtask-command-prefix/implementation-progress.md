# Implementation Progress: CI-Prefixed xtask Subcommands

- [ ] Inventory current xtask subcommands and define the `ci-*` mapping.
- [ ] Locate all references to `cargo xtask` across workflows, docs, scripts, skills, and templates.
- [ ] Rename clap subcommands to `ci-*` in `engine/xtask`.
- [ ] Add backward-compatible aliases and deprecation warnings for legacy names.
- [ ] Update help strings and dry-run outputs to `ci-*` names.
- [ ] Update GitHub Actions workflows to use `ci-*` commands.
- [ ] Update docs and guides to use `ci-*` commands.
- [ ] Update skill guides and scripts to use `ci-*` commands.
- [ ] Align CLI template xtask command naming or document intentional divergence.
- [ ] Verify new `ci-*` commands run locally.
- [ ] Verify legacy alias commands still run and warn.
