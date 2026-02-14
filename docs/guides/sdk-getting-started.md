# Wavecraft SDK — Getting Started

This guide walks you through building your first VST3/CLAP audio plugin using the Wavecraft SDK.

---

## Prerequisites

- **Rust** (1.75+) — Install via [rustup](https://rustup.rs/)
- **Node.js** (18+) — For building the React UI
- **macOS** (primary) — Windows/Linux support is secondary

---

## Quick Start (< 30 minutes)

### 1. Install Wavecraft CLI

```bash
cargo install wavecraft
```

> **Troubleshooting:** If you see `command not found: wavecraft`, your shell PATH may not include Cargo's bin directory. Either restart your terminal, or add it manually:
>
> **zsh:** `echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc`
>
> **bash:** `echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc && source ~/.bashrc`
>
> Or run directly: `~/.cargo/bin/wavecraft create my-plugin`

To verify installation:

```bash
wavecraft --help
```

To check the installed version:

```bash
wavecraft --version
# or
wavecraft -V
```

### 2. Create a New Plugin Project

```bash
# Simple - uses placeholder values for vendor/email/url
wavecraft create my-plugin

# With custom vendor information (optional)
wavecraft create my-plugin \
  --vendor "My Company" \
  --email "info@example.com" \
  --url "https://example.com"

cd my-plugin
```

The CLI creates a complete project with:

- Rust engine configured with Wavecraft dependencies
- React UI with TypeScript and Tailwind CSS
- xtask build system
- Ready-to-build plugin template

> **Tip:** You can customize vendor, email, and URL later by editing `engine/Cargo.toml`

### 3. Install Dependencies

```bash
# Install UI dependencies
cd ui && npm install && cd ..
```

### 4. Build Your Plugin

```bash
# Build VST3 and CLAP bundles
cargo xtask bundle

# Bundles are created in:
# target/bundled/my-plugin.vst3/
# target/bundled/my-plugin.clap
```

### 5. Test in Your DAW

Copy the plugin to your DAW's plugin directory:

```bash
# Install to system directories (macOS)
cargo xtask install
```

---

## CLI Reference

### Creating New Projects

```bash
# Simple - uses placeholder values
wavecraft create my-plugin

# With custom vendor information
wavecraft create my-plugin \
  --vendor "My Company" \
  --email "info@example.com" \
  --url "https://example.com"

# Custom output directory
wavecraft create my-plugin --output ~/projects/my-plugin

# Skip git initialization
wavecraft create my-plugin --no-git

# View all options
wavecraft create --help
```

### Starting Dev Servers

```bash
# Start WebSocket + UI dev servers
wavecraft start

# Use a different WebSocket port
wavecraft start --port 9010

# Use a different UI dev server port
wavecraft start --ui-port 5174

# Auto-install UI dependencies if missing
wavecraft start --install
```

**Note:** The UI dev server requires a free port. If the port is already in use,
`wavecraft start` will exit with an error. Stop the other process or pass
`--ui-port` to choose a different port.

### Updating CLI and Dependencies

```bash
# Update CLI + project dependencies (works from any directory)
wavecraft update
```

This command runs in two phases:

1. **CLI self-update** — Runs `cargo install wavecraft` to ensure you have the latest CLI version. Shows version change (e.g., "updated to 0.9.1, was 0.9.0") or confirms you're up to date.
2. **Project dependency update** — If run from inside a Wavecraft plugin project:
   - Updates Rust dependencies if `engine/Cargo.toml` exists (runs `cargo update` in `engine/`)
   - Updates npm dependencies if `ui/package.json` exists (runs `npm update` in `ui/`)

**Key behaviors:**

- Works from **any directory** — outside a project, only the CLI is updated
- CLI self-update failures are non-fatal — project dependency updates still proceed
- Reports success/failure for each phase independently

**Use case:** Run `wavecraft update` regularly to keep both the CLI tool and your project dependencies current.

### CLI Options

| Option         | Description               | Default           |
| -------------- | ------------------------- | ----------------- |
| `--vendor, -v` | Company or developer name | `"Your Company"`  |
| `--email, -e`  | Contact email (optional)  | —                 |
| `--url, -u`    | Website URL (optional)    | —                 |
| `--output, -o` | Output directory          | `./<plugin-name>` |
| `--no-git`     | Skip git initialization   | false             |

---

## Project Structure

```
my-plugin/
├── engine/                  # Rust audio engine
│   ├── Cargo.toml           # Dependencies on wavecraft-* crates (git tags)
│   └── src/
│       ├── lib.rs           # Plugin assembly (signal chain + metadata)
│       └── processors/      # Your custom DSP processors
│           ├── mod.rs        # Module exports
│           └── oscillator.rs # Example: sine-wave oscillator
│
├── ui/                      # React UI (TypeScript + Tailwind)
│   ├── package.json         # Dependencies: @wavecraft/core + @wavecraft/components
│   └── src/
│       ├── App.tsx          # Your UI layout
│       └── generated/       # Build artifacts (gitignored, created by `wavecraft start`)
│           └── parameters.ts # Type-safe parameter IDs (auto-generated)
│
└── xtask/                   # Build automation
    └── src/main.rs          # xtask commands (bundle, dev, install, etc.)
```

**Key files:**

| File                                  | Purpose                                                  |
| ------------------------------------- | -------------------------------------------------------- |
| `engine/src/lib.rs`                   | Plugin assembly — signal chain + `wavecraft_plugin!` DSL |
| `engine/src/processors/`              | Folder for your custom `Processor` implementations       |
| `engine/src/processors/oscillator.rs` | Example oscillator (sine wave, frequency + level)        |
| `ui/src/App.tsx`                      | User interface layout with parameter controls            |

**Note:** The generated project references:

- **Rust crates** via git tags (e.g., `git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0"`)
- **npm packages** from the `@wavecraft` organization (`@wavecraft/core`, `@wavecraft/components`)

---

## Implementing Your DSP

The CLI-generated plugin uses the **declarative DSL**. Your plugin is assembled in `engine/src/lib.rs`:

```rust
use wavecraft::prelude::*;

mod processors;
use processors::Oscillator;

// Built-in processors need named wrappers (parameter-ID prefix)
wavecraft_processor!(InputGain => Gain);
wavecraft_processor!(OutputGain => Gain);

// Custom processors (like Oscillator) are used directly — they already
// implement the Processor trait with their own parameters.

// Vendor/URL metadata is derived from Cargo.toml.
// Email is set explicitly in the macro (optional).
wavecraft_plugin! {
    name: "My Plugin",
    signal: SignalChain![InputGain, OutputGain],
  email: "info@example.com",
    // Enable the oscillator by switching to:
    // signal: SignalChain![InputGain, Oscillator, OutputGain],
}
```

Custom DSP code lives in the `engine/src/processors/` folder. The template includes a working oscillator example in `processors/oscillator.rs`.

### Writing a Custom Processor

Every processor needs two parts: a **parameter struct** and a **processor struct**.

```rust
// engine/src/processors/oscillator.rs
use wavecraft::prelude::*;
use wavecraft::ProcessorParams;

// 1. Define parameters with the derive macro
#[derive(ProcessorParams, Default, Clone)]
pub struct OscillatorParams {
    #[param(range = "20.0..=5000.0", default = 440.0, unit = "Hz", factor = 2.5)]
    pub frequency: f32,

    #[param(range = "0.0..=1.0", default = 0.5, unit = "%")]
    pub level: f32,
}

// 2. Implement the Processor trait
#[derive(Default)]
pub struct Oscillator {
    sample_rate: f32,
    phase: f32,
}

impl Processor for Oscillator {
    type Params = OscillatorParams;

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        buffer: &mut [&mut [f32]],
        _transport: &Transport,
        params: &Self::Params,
    ) {
        if self.sample_rate == 0.0 {
            return;
        }

        let phase_delta = params.frequency / self.sample_rate;

        // Save phase so every channel gets the same waveform
        let start_phase = self.phase;

        for channel in buffer.iter_mut() {
            self.phase = start_phase;
            for sample in channel.iter_mut() {
                *sample = (self.phase * std::f32::consts::TAU).sin() * params.level;
                self.phase += phase_delta;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
            }
        }
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}
```

### Key Concepts

1. **`Processor` trait** — Your audio processing logic.
   - `process()` — Called per audio buffer. Must be real-time safe!
   - `set_sample_rate()` — Called when sample rate changes.
   - `reset()` — Called when playback stops. Clear delay lines, etc.
2. **`ProcessorParams` derive** — Generates parameter metadata for UI + host automation.
3. **`Transport`** — Contains tempo, time signature, playhead position.
4. **`SignalChain![]`** — Chains processors in order. Each processor gets its own namespaced parameters.

### Adding a Processor to Your Project

1. Create a file: `engine/src/processors/filter.rs`
2. Implement `Processor` + `ProcessorParams` (see oscillator example)
3. Export in `processors/mod.rs`:
   ```rust
   pub mod filter;
   pub use filter::Filter;
   ```
4. Wire into the signal chain in `lib.rs`:
   ```rust
   use processors::{Oscillator, Filter};
   // Custom processors are used directly in SignalChain (no wavecraft_processor! wrapper needed)
   // signal: SignalChain![InputGain, Oscillator, Filter, OutputGain],
   ```

The UI automatically discovers new parameters — no React changes needed.

### Real-Time Safety Rules

The `process()` method runs on the audio thread. You **must not**:

- Allocate memory (`Box::new`, `Vec::push`, `String::from`)
- Lock mutexes or use blocking operations
- Make system calls (file I/O, network)
- Log or print

---

## Defining Parameters

Parameters are defined using the `#[derive(ProcessorParams)]` macro on a struct:

```rust
use wavecraft::prelude::*;

#[derive(ProcessorParams, Default, Clone)]
struct MyParams {
    #[param(range = "-24.0..=24.0", default = 0.0, unit = "dB")]
    gain: f32,

    #[param(range = "0.0..=1.0", default = 1.0, unit = "%")]
    mix: f32,

    #[param(range = "20.0..=20000.0", default = 1000.0, unit = "Hz", factor = 2.5)]
    frequency: f32,
}
```

### `#[param]` Attribute Options

| Attribute | Required | Description                         | Example               |
| --------- | -------- | ----------------------------------- | --------------------- |
| `range`   | Yes      | Value range as `"MIN..=MAX"`        | `range = "0.0..=1.0"` |
| `default` | No       | Default value (midpoint if omitted) | `default = 0.0`       |
| `unit`    | No       | Unit string for display             | `unit = "dB"`         |
| `factor`  | No       | Skew factor (>1 = log, <1 = exp)    | `factor = 2.5`        |
| `group`   | No       | UI grouping name                    | `group = "Input"`     |

---

## Customizing the UI

The UI is a React application in the `ui/` folder. Wavecraft provides ready-to-use components:

### Built-in Components

Wavecraft provides two npm packages:

- **`@wavecraft/core`** — IPC bridge, hooks, and utilities
- **`@wavecraft/components`** — Pre-built React components

```tsx
import { Meter, ParameterSlider, ParameterGroup } from '@wavecraft/components';
import { useAllParameters, useParameterGroups } from '@wavecraft/core';

function App() {
  const { params, isLoading } = useAllParameters();
  const groups = useParameterGroups(params);

  return (
    <div className="plugin-ui">
      {/* Stereo level meter */}
      <Meter />

      {/* Automatic parameter discovery */}
      {groups.length > 0
        ? groups.map((group) => (
            <ParameterGroup key={group.name} group={group} />
          ))
        : params?.map((p) => <ParameterSlider key={p.id} id={p.id} />)}
    </div>
  );
}
```

### Custom Components with Hooks

```tsx
import { useParameter } from '@wavecraft/core';
import type { ParameterId } from '@wavecraft/core';

function MyKnob({ id }: { id: ParameterId }) {
  const { value, setValue, info } = useParameter(id);

  return (
    <div className="knob">
      <label>{info?.name}</label>
      <input
        type="range"
        min={info?.min}
        max={info?.max}
        value={value ?? info?.default}
        onChange={(e) => setValue(parseFloat(e.target.value))}
      />
      <span>
        {value?.toFixed(2)} {info?.unit}
      </span>
    </div>
  );
}
```

### Available Hooks

All hooks are exported from `@wavecraft/core`:

| Hook                         | Purpose                                                                     |
| ---------------------------- | --------------------------------------------------------------------------- |
| `useParameter(id)`           | Read/write a single parameter (type-safe `ParameterId` with autocompletion) |
| `useAllParameters()`         | Fetch all plugin parameters (automatic discovery)                           |
| `useParameterGroups(params)` | Group parameters by their `group` attribute                                 |
| `useMeterFrame()`            | Access real-time meter data                                                 |
| `useConnectionStatus()`      | WebSocket connection status (dev mode)                                      |

---

## Development Workflow

### Browser-Based Development (Recommended)

Run the UI in a browser with hot reload and real engine communication:

```bash
cargo xtask dev
```

This starts:

- **Rust WebSocket server** — Real parameter sync and metering
- **Vite dev server** — Hot module reload for React

Open `http://localhost:5173` in your browser.

> **TypeScript autocompletion:** `wavecraft start` (and `cargo xtask dev` in SDK mode) automatically generates type-safe parameter IDs. After the dev server starts, your IDE will autocomplete parameter IDs in `useParameter()` calls and flag invalid IDs as type errors.

### Building for Production

```bash
# Debug build (fast, for testing)
cargo xtask bundle --debug

# Release build (optimized)
cargo xtask bundle --release
```

### Testing in DAW

```bash
# Build and install in one step
cargo xtask bundle && cargo xtask install
```

---

## Common Tasks

### Adding a New Parameter

1. Add to `wavecraft_params!` in your Rust code
2. Add a `<ParameterSlider id="new-param" />` to your UI
3. Rebuild: `cargo xtask bundle`

### Adding Metering to DSP

```rust
use wavecraft_core::prelude::*;

pub struct MyProcessor {
    meter_producer: Option<MeterProducer>,
    // ...
}

impl Processor for MyProcessor {
    fn process(&mut self, _transport: &Transport, buffer: &mut Buffer) {
        // Your processing code...

        // Send meter data (non-blocking)
        if let Some(ref mut meter) = self.meter_producer {
            let (peak_l, rms_l, peak_r, rms_r) = calculate_stereo_meters(buffer);
            meter.push(MeterFrame { peak_l, rms_l, peak_r, rms_r });
        }
    }
}
```

### Changing Plugin Metadata

Edit `engine/Cargo.toml`:

```toml
[package]
name = "my-awesome-plugin"
version = "1.0.0"

[package.metadata.nih-plug]
name = "My Awesome Plugin"
vendor = "Your Company"
```

---

## Build Commands Reference

| Command                        | Description                           |
| ------------------------------ | ------------------------------------- |
| `cargo xtask dev`              | Start dev servers (WebSocket + Vite)  |
| `cargo xtask bundle`           | Build VST3/CLAP bundles               |
| `cargo xtask bundle --release` | Build optimized release               |
| `cargo xtask test`             | Run all tests                         |
| `cargo xtask lint`             | Run linters                           |
| `cargo xtask install`          | Install plugins to system directories |
| `cargo xtask sign`             | Sign plugins for macOS                |
| `cargo xtask clean`            | Clean build artifacts                 |

---

## SDK Development (Contributing to Wavecraft)

If you're working on the Wavecraft SDK itself (not building a plugin), the CLI **automatically detects** when it's running from the monorepo source checkout:

1. Prepare the canonical SDK template once:

```bash
./scripts/setup-dev-template.sh
```

2. Run the browser/dev workflow against `sdk-template/`:

```bash
cargo xtask dev
```

3. Optionally scaffold separate throwaway plugins for CLI validation:

```bash
# From the wavecraft repo root:
cargo run -p wavecraft -- create TestPlugin --output target/tmp/test-plugin

# The CLI will print:
# ℹ Detected SDK development mode (running from source checkout)
#   → Using local path dependencies instead of git tags
```

**What this does:** Instead of generating `Cargo.toml` dependencies that reference a git tag (which may not exist yet for unreleased versions), the CLI generates **local path dependencies** pointing to the SDK crates in your checkout. This means `wavecraft start` works immediately without needing a published release.

**When does it trigger?**

- The running binary is inside a cargo `target/` directory (i.e., built via `cargo run`)
- The wavecraft monorepo marker (`engine/crates/wavecraft-nih_plug/Cargo.toml`) is found by walking up from the binary location

**When does it NOT trigger?**

- Binaries installed via `cargo install wavecraft` (located in `~/.cargo/bin/`)
- The explicit `--local-sdk` hidden flag still works as a manual override

---

## Troubleshooting

### Plugin doesn't appear in DAW

1. Verify the bundle was created: `ls target/bundled/`
2. Check install location: `cargo xtask install --dry-run`
3. macOS: Rescan plugins in your DAW preferences
4. Try running the plugin validator: `pluginval /path/to/plugin.vst3`

### UI doesn't update with parameter changes

1. Ensure parameter ID in UI matches Rust definition
2. Check browser console for IPC errors
3. Verify WebSocket connection in dev mode

### UI dev server port already in use

If you see an error indicating the UI dev server port is already in use:

1. Stop any running Vite dev servers using that port
2. Re-run with a different UI port: `wavecraft start --ui-port 5174`

### Audio glitches/dropouts

1. Check for allocations in `process()` — use `#[deny(clippy::all)]`
2. Reduce buffer size in DAW to test
3. Profile with Instruments (macOS) or perf (Linux)

---

## Next Steps

- **[High-Level Design](../architecture/high-level-design.md)** — Architecture overview and detailed topic docs
- **[Coding Standards](../architecture/coding-standards.md)** — Coding conventions and language-specific guides
- **[macOS Signing Guide](./macos-signing.md)** — Prepare for distribution

---

## Getting Help

- **Issues:** File bugs or feature requests on GitHub
- **Discussions:** Ask questions in GitHub Discussions
- **Examples:** See the `examples/` folder in the SDK repository
