# Wavecraft

[![CI](https://github.com/RonHouben/wavecraft/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/RonHouben/wavecraft/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/wavecraft.svg)](https://crates.io/crates/wavecraft)
[![npm](https://img.shields.io/npm/v/@wavecraft/core.svg)](https://www.npmjs.com/package/@wavecraft/core)

> **🚧 Under Construction — Not Production Ready**
>
> Wavecraft is in active development. See the [Roadmap](docs/roadmap.md) for current status. Expect breaking changes until v1.0.

A cross-platform audio effects plugin framework built with **Rust** and **React**.

## Overview

Wavecraft is an audio effects plugin framework (VST3) for **macOS + Ableton Live**. It combines a real-time safe Rust audio engine with a modern React-based UI.

> **Note:** Windows and Linux support is deprioritized. The architecture supports cross-platform via wry, but current development focuses exclusively on macOS + Ableton Live.

## Architecture

| Layer            | Technology                                  | Purpose                    |
| ---------------- | ------------------------------------------- | -------------------------- |
| **Audio/DSP**    | Rust (nih-plug)                             | Real-time audio processing |
| **Plugin API**   | VST3 (primary), CLAP, AU (via clap-wrapper) | DAW integration            |
| **UI**           | React (Vite)                                | User interface             |
| **UI Embedding** | wry (WKWebView on macOS)                    | WebView embedding          |

Communication between UI and audio uses lock-free parameter systems and ring buffers to maintain real-time safety.

## Platforms

- **macOS (WKWebView)** — Primary, actively developed
- Windows (WebView2) — Deprioritized
- Linux (WebKitGTK) — Deprioritized

## Target DAWs

- **Ableton Live (macOS)** — Primary target
- Logic Pro (macOS, AU) — Secondary, nice-to-have
- Other DAWs — Deprioritized

## npm Packages

Wavecraft's UI code is published as two npm packages under the `@wavecraft` organization:

| Package                                                                      | Description                                               |
| ---------------------------------------------------------------------------- | --------------------------------------------------------- |
| [@wavecraft/core](https://www.npmjs.com/package/@wavecraft/core)             | IPC bridge, React hooks, and utilities                    |
| [@wavecraft/components](https://www.npmjs.com/package/@wavecraft/components) | Pre-built React components (Meter, ParameterSlider, etc.) |

```bash
npm install @wavecraft/core @wavecraft/components
```

## Rust Crates

Wavecraft's Rust SDK is published to crates.io:

| Crate                                                             | Description                                   |
| ----------------------------------------------------------------- | --------------------------------------------- |
| [wavecraft](https://crates.io/crates/wavecraft)                   | CLI tool for scaffolding new plugins          |
| [wavecraft-core](https://crates.io/crates/wavecraft-core)         | nih-plug VST3/CLAP integration                |
| [wavecraft-dsp](https://crates.io/crates/wavecraft-dsp)           | Pure DSP algorithms, `Processor` trait        |
| [wavecraft-protocol](https://crates.io/crates/wavecraft-protocol) | Shared parameter definitions                  |
| [wavecraft-macros](https://crates.io/crates/wavecraft-macros)     | Procedural macros (`wavecraft_plugin!`, etc.) |
| [wavecraft-bridge](https://crates.io/crates/wavecraft-bridge)     | UI ↔ Audio IPC handling                       |
| [wavecraft-metering](https://crates.io/crates/wavecraft-metering) | SPSC ring buffer for real-time meters         |

```bash
# Install CLI
cargo install wavecraft

# Create a new plugin
wavecraft create my-plugin
```

## Project Structure

```
wavecraft/
├── cli/                          # Wavecraft CLI tool
│   ├── src/                      # CLI source code
│   └── template/                 # Template extraction and variable processing
│
├── sdk-template/                 # Canonical project template (embedded by CLI)
│   ├── Cargo.toml.template
│   ├── engine/
│   └── ui/
│
├── dev-server/                   # Unified dev server (standalone crate)
│   ├── Cargo.toml                # Not in engine workspace
│   └── src/                      # Dev server source
│       ├── ws/                   # WebSocket server
│       ├── audio/                # Audio I/O (feature-gated)
│       ├── reload/               # Hot-reload & file watching
│       ├── host.rs               # DevServerHost implementation
│       └── session.rs            # Session management
│
├── engine/                       # Rust audio engine & plugin
│   ├── Cargo.toml                # Workspace root
│   └── crates/
│       ├── wavecraft-core/       # Framework integration (macros, nih-plug wrapper)
│       ├── wavecraft-dsp/        # Pure DSP (Processor trait, built-in processors)
│       ├── wavecraft-bridge/     # UI ↔ Audio IPC (ParameterHost trait)
│       ├── wavecraft-protocol/   # Shared contracts (param IDs, JSON-RPC types)
│       └── wavecraft-metering/   # SPSC ring buffer for real-time meters
│
├── ui/                           # React package workspace (npm)
│   ├── packages/
│   │   ├── core/                 # @wavecraft/core package source
│   │   └── components/           # @wavecraft/components package source
│   └── test/                     # Shared test utilities
│
├── docs/                         # Architecture & specs
└── packaging/                    # Platform installers
```

## Documentation

- [SDK Getting Started](docs/guides/sdk-getting-started.md) — Build your first plugin with Wavecraft
- [High-Level Design](docs/architecture/high-level-design.md) — Architecture overview and navigation hub for detailed topic docs
- [Coding Standards](docs/architecture/coding-standards.md) — Coding conventions overview and navigation hub for language-specific guides
- [Agent Development Flow](docs/architecture/agent-development-flow.md) — How specialized agents collaborate through the development lifecycle
- [Roadmap](docs/roadmap.md) — Milestone tracking and implementation progress
- [Visual Testing Guide](docs/guides/visual-testing.md) — VS Code Simple Browser visual testing with screenshot evidence workflows

## Building

Wavecraft uses a Rust-based `xtask` build system for cross-platform builds.

### Prerequisites

- **Rust** (stable, 2024 edition)
- **CMake** (for AU wrapper on macOS)

### Quick Start

```bash
cd engine

# Run tests
cargo xtask test

# Build VST3 and CLAP plugins
cargo xtask bundle

# Build everything and install (macOS)
cargo xtask all
```

### Available Commands

| Command                                 | Description                                              |
| --------------------------------------- | -------------------------------------------------------- |
| `cargo xtask ci-check`                  | Pre-push validation (6 phases: docs, build, lint, tests) |
| `cargo xtask ci-check --full`           | Full validation (adds template + CD dry-run)             |
| `cargo xtask lint`                      | Run linters (ESLint, Prettier, tsc, cargo fmt, clippy)   |
| `cargo xtask validate-template`         | Validate CLI template generation                         |
| `cargo xtask bundle`                    | Build and bundle VST3/CLAP plugins                       |
| `cargo xtask bundle --debug`            | Debug build (faster compile, no optimizations)           |
| `cargo xtask bundle --install`          | Build and install plugins in one step                    |
| `cargo xtask test`                      | Run all tests (engine + UI)                              |
| `cargo xtask test --ui`                 | Run UI tests only (npm test)                             |
| `cargo xtask test --engine`             | Run engine tests only (cargo test)                       |
| `cargo xtask test --all`                | Test all workspace crates                                |
| `cargo xtask au`                        | Build AU wrapper (macOS only, requires CMake)            |
| `cargo xtask install`                   | Install plugins to system directories                    |
| `cargo xtask clean`                     | Clean build artifacts                                    |
| `cargo xtask clean --installed --force` | Also remove installed plugins                            |
| `cargo xtask all`                       | Full pipeline: test → bundle → au → install              |
| `cargo xtask all --dry-run`             | Preview what would be done                               |

### Feature Flags

Feature flags enable optional functionality in the plugin. Pass them via `-f` or `--features`:

```bash
# Enable runtime allocation detection (debug builds)
cargo xtask bundle -f assert_process_allocs
```

| Feature                 | Description                                                                                                                            |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `assert_process_allocs` | Enable runtime allocation detection on the audio thread (debug builds only). Useful for verifying real-time safety during development. |

**Note:** The React UI is built automatically as part of every plugin build.

### Build Outputs

After building, plugins are located at:

```
engine/target/bundled/
├── wavecraft.vst3    # VST3 plugin
└── wavecraft.clap    # CLAP plugin

packaging/macos/au-wrapper/build/
└── Wavecraft.component    # AU plugin (macOS only)
```

### Installation Directories

| Platform | VST3                                  | CLAP                                  | AU                                     |
| -------- | ------------------------------------- | ------------------------------------- | -------------------------------------- |
| macOS    | `~/Library/Audio/Plug-Ins/VST3/`      | `~/Library/Audio/Plug-Ins/CLAP/`      | `~/Library/Audio/Plug-Ins/Components/` |
| Windows  | `C:\Program Files\Common Files\VST3\` | `C:\Program Files\Common Files\CLAP\` | N/A                                    |
| Linux    | `~/.vst3/`                            | `~/.clap/`                            | N/A                                    |

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Wavecraft is released under the [MIT License](LICENSE).
