# VSTKit

A cross-platform audio effects plugin framework built with **Rust** and **React**.

## Overview

VSTKit is a cross-platform audio effects plugin framework (VST3). It combines a real-time safe Rust audio engine with a modern React-based UI, targeting professional DAW environments.

## Architecture

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Audio/DSP** | Rust (nih-plug) | Real-time audio processing |
| **Plugin API** | VST3 (CLAP/AU optional) | DAW integration |
| **UI** | React (Vite) | User interface |
| **UI Embedding** | wry (WebView2/WKWebView) | Cross-platform webview |

Communication between UI and audio uses lock-free parameter systems and ring buffers to maintain real-time safety.

## Platforms

- macOS (WKWebView)
- Windows (WebView2)
- Linux (WebKitGTK)

## Target DAWs

- Ableton Live (primary)
- Logic Pro, Reaper, and other VST3-compatible hosts

## Project Structure

```
vstkit/
├── engine/                       # Rust audio engine & plugin
│   ├── Cargo.toml                # Workspace root
│   └── crates/
│       ├── dsp/                  # Pure DSP (no plugin deps)
│       ├── plugin/               # nih-plug host integration
│       ├── bridge/               # UI ↔ Audio IPC
│       └── protocol/             # Shared contracts (param IDs, ranges)
│
├── ui/                           # React SPA (Vite + TypeScript)
│   ├── src/
│   └── dist/                     # Build output (embedded in plugin)
│
├── docs/                         # Architecture & specs
├── scripts/                      # Build & CI helpers
├── packaging/                    # Platform installers
│   ├── macos/
│   ├── windows/
│   └── linux/
│
└── tests/
    ├── integration/              # Host-in-the-loop tests
    └── dsp/                      # Offline DSP correctness tests
```

## Documentation

- [High-Level Design](docs/architecture/high-level-design.md) — Architecture overview, component design, data flows, and implementation roadmap
- [Coding Standards](docs/architecture/coding-standards.md) — TypeScript, Rust, and React conventions
- [Roadmap](docs/roadmap.md) — Milestone tracking and implementation progress

## Building

VSTKit uses a Rust-based `xtask` build system for cross-platform builds.

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

| Command | Description |
|---------|-------------|
| `cargo xtask bundle` | Build and bundle VST3/CLAP plugins |
| `cargo xtask bundle --debug` | Debug build (faster compile, no optimizations) |
| `cargo xtask bundle -f webview_editor` | Build with WebView UI (see Feature Flags) |
| `cargo xtask bundle --install` | Build and install plugins in one step |
| `cargo xtask test` | Run unit tests for dsp and protocol crates |
| `cargo xtask test --all` | Test all workspace crates |
| `cargo xtask au` | Build AU wrapper (macOS only, requires CMake) |
| `cargo xtask install` | Install plugins to system directories |
| `cargo xtask clean` | Clean build artifacts |
| `cargo xtask clean --installed --force` | Also remove installed plugins |
| `cargo xtask all` | Full pipeline: test → bundle → au → install |
| `cargo xtask all --dry-run` | Preview what would be done |

### Feature Flags

Feature flags enable optional functionality in the plugin. Pass them via `-f` or `--features`:

```bash
# Single feature
cargo xtask bundle -f webview_editor

# Multiple features (comma-separated)
cargo xtask bundle -f webview_editor,assert_process_allocs
```

| Feature | Description |
|---------|-------------|
| `webview_editor` | Enable the React-based WebView UI. Automatically builds the React app from `ui/` before bundling the plugin. Required for the full UI experience. |
| `assert_process_allocs` | Enable runtime allocation detection on the audio thread (debug builds only). Useful for verifying real-time safety during development. |

**Note:** When `webview_editor` is enabled, the build system will:
1. Run `npm run build` in the `ui/` directory
2. Embed the built assets into the plugin binary
3. Bundle the plugin with WebView support

### Build Outputs

After building, plugins are located at:

```
engine/target/bundled/
├── vstkit.vst3    # VST3 plugin
└── vstkit.clap    # CLAP plugin

packaging/macos/au-wrapper/build/
└── VstKit.component    # AU plugin (macOS only)
```

### Installation Directories

| Platform | VST3 | CLAP | AU |
|----------|------|------|----|
| macOS | `~/Library/Audio/Plug-Ins/VST3/` | `~/Library/Audio/Plug-Ins/CLAP/` | `~/Library/Audio/Plug-Ins/Components/` |
| Windows | `C:\Program Files\Common Files\VST3\` | `C:\Program Files\Common Files\CLAP\` | N/A |
| Linux | `~/.vst3/` | `~/.clap/` | N/A |

## License

TBD
