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

- [High-Level Design](docs/architecture/high-level-design.md) — Architecture overview and implementation roadmap

## License

TBD
