# Versioning and Distribution

This document covers Wavecraft's version management strategy, build-time injection, CI-automated versioning, and packaging/distribution for all supported platforms.

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview, component design, data flows
- [Coding Standards Overview](./coding-standards.md) — Code conventions and navigation hub
- [Plugin Formats](./plugin-formats.md) — VST3, CLAP, and AU format details
- [Development Workflows](./development-workflows.md) — Build commands, CI/CD pipelines
- [CI/CD Pipeline Guide](../guides/ci-pipeline.md) — Pipeline details and troubleshooting

---

## Versioning

Wavecraft uses semantic versioning (SemVer) with automated version management via the CD pipeline. The CLI version is the **user-facing entry point** (`cargo install wavecraft`), and the workspace version (`engine/Cargo.toml`) is kept aligned with it. All version bumping is handled by CI — developers do not manually bump versions during feature development.

### Version Flow

```
┌─────────────────────────────────────────────────────────┐
│              CD Pipeline (continuous-deploy.yml)         │
│                                                         │
│  Push to main → detect changes → auto-bump patch        │
│  → publish to crates.io / npm → push git tags           │
└────────────┬────────────────────────┬───────────────────┘
             │                        │
             ▼                        ▼
  ┌─────────────────────┐  ┌─────────────────────┐
  │ CLI (crates.io)     │  │ npm packages        │
  │ cargo install       │  │ @wavecraft/core     │
  │ wavecraft           │  │ @wavecraft/components│
  └─────────────────────┘  └─────────────────────┘
             │
             ▼
  ┌─────────────────────┐
  │ engine/Cargo.toml   │  [workspace.package]
  │ (aligned with CLI)  │  version = "X.Y.Z"
  └─────────┬───────────┘
            │
            ├────────────────────────────────┐
            │                                │
            ▼                                ▼
  ┌─────────────────────┐       ┌─────────────────────┐
  │ Plugin Binary       │       │ Vite Build          │
  │ env!("CARGO_PKG_    │       │ __APP_VERSION__     │
  │      VERSION")      │       │ compile-time const  │
  │ → VST3/CLAP metadata│       │ → VersionBadge UI   │
  └─────────────────────┘       └─────────────────────┘
```

### Key Design Decisions

1. **CI-automated versioning** — All version bumps are handled by the CD pipeline. No manual version bumping is required — not per feature, not at milestones.

2. **CLI as entry point** — The CLI version (`cargo install wavecraft`) is the user-facing version. The workspace version is aligned with the CLI version.

3. **Build-time injection** — Version is embedded at compile time, not fetched via IPC at runtime. This ensures zero runtime cost and no startup latency.

4. **Vite `define` block** — The `__APP_VERSION__` constant is injected via Vite's `define` configuration, which performs compile-time string replacement.

5. **Development fallback** — When building without xtask (e.g., `npm run dev`), the version is read directly from `engine/Cargo.toml` using a regex parser in `vite.config.ts`.

6. **No manual sync** — CI keeps all versions in sync automatically.

### How It Works

The version is defined in `engine/Cargo.toml` under `[workspace.package]` and propagates to plugin metadata and the UI at build time via Vite's `define` configuration. The CLI version (`cli/Cargo.toml`) is the **user-facing entry point** — the workspace version should be kept aligned with the CLI version.

1. A push to `main` triggers the CD pipeline (`continuous-deploy.yml`)
2. `detect-changes` identifies which SDK components changed
3. If **any** component changed, the CLI is also published (cascade trigger)
4. For each package, CI compares the local version against the published registry version
5. If the local version is not ahead, CI auto-bumps the patch version, commits as `github-actions[bot]`, and publishes
6. The `github-actions[bot]` author is detected by the pipeline to prevent infinite re-triggering

### Packages Auto-Bumped by CI

- CLI (`cli/Cargo.toml`)
- `@wavecraft/core` (npm)
- `@wavecraft/components` (npm)
- `engine/Cargo.toml` workspace version

### What Developers Should Do

- Do **not** manually bump any versions during feature development
- If you need a specific version (e.g., minor bump for a breaking change), bump it in your PR — CI will respect the manual bump

### What CI Does

- Auto-patches all distribution packages when their local version ≤ registry version
- Commits version bumps **locally only** (not pushed to `main`) — branch protection rulesets prevent direct pushes
- Pushes **git tags only** for each published version (tags are not subject to branch protection)

### Infinite Loop Prevention

- Auto-bump commits are no longer pushed to `main`, so the infinite loop scenario does not arise
- The `detect-changes` guard (`github-actions[bot]` author check) is kept as defense-in-depth
- Preferred over `[skip ci]` because other workflows (CI, template validation) should still run normally

The VersionBadge component in the UI displays the current version (e.g., "v0.9.0").

---

## Packaging & Distribution

### macOS

Notarization and signing required; package VST3 (`.vst3`), CLAP (`.clap`), and AU (`.component` via clap-wrapper); embed React assets into plugin bundle resources.

- VST3: `/Library/Audio/Plug-Ins/VST3/Wavecraft.vst3`
- CLAP: `/Library/Audio/Plug-Ins/CLAP/Wavecraft.clap`
- AU: `/Library/Audio/Plug-Ins/Components/Wavecraft.component` (built via clap-wrapper from CLAP)
- AU requires valid `Info.plist` with `AudioComponents` array (clap-wrapper generates this)
- **Signing**: `cargo xtask sign` (or `--adhoc` for local dev)
- **Notarization**: `cargo xtask notarize --full` (requires Apple Developer account)
- **Release workflow**: `cargo xtask release` (bundle → sign → notarize)

### Windows

Ensure WebView2 runtime installed or include evergreen bootstrap in installer; produce .dll VST3 and installer (MSI). AU not applicable. Signing is deprioritized.

### Linux

Many host distros vary; recommend shipping CLAP/VST3 and provide AppImage/Flatpak for GUI testing. AU not applicable. Deprioritized.

Docs for VST3 build process: Steinberg dev portal.
