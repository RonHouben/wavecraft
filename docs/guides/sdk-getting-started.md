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

### CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `--vendor, -v` | Company or developer name | `"Your Company"` |
| `--email, -e` | Contact email (optional) | — |
| `--url, -u` | Website URL (optional) | — |
| `--output, -o` | Output directory | `./<plugin-name>` |
| `--no-git` | Skip git initialization | false |

---

## Project Structure

```
my-plugin/
├── engine/                  # Rust audio engine
│   ├── Cargo.toml           # Dependencies on wavecraft-* crates (git tags)
│   └── src/
│       └── lib.rs           # Plugin entry point (using declarative DSL)
│
├── ui/                      # React UI (TypeScript + Tailwind)
│   ├── package.json         # Dependencies: @wavecraft/core + @wavecraft/components
│   └── src/
│       └── App.tsx          # Your UI layout
│
└── xtask/                   # Build automation
    └── src/main.rs          # xtask commands (bundle, dev, install, etc.)
```

**Note:** The generated project references:
- **Rust crates** via git tags (e.g., `git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0"`)
- **npm packages** from the `@wavecraft` organization (`@wavecraft/core`, `@wavecraft/components`)

---

## Implementing Your DSP

The CLI-generated plugin uses the **declarative DSL** for quick setup. Your plugin is defined in `engine/src/lib.rs`:

```rust
use wavecraft_core::prelude::*;

// Define the processor chain (using built-in Gain processor)
wavecraft_processor!(MyPluginGain => Gain);

// Generate the complete plugin
wavecraft_plugin! {
    name: "My Plugin",
    vendor: "My Company",
    url: "https://example.com",
    email: "info@example.com",
    signal: MyPluginGain,
}
```

### Adding Custom DSP

To implement custom processing, create your own processor struct and implement the `Processor` trait:

```rust
use wavecraft_core::prelude::*;

pub struct MyProcessor {
    sample_rate: f32,
    gain_smoother: f32,
}

impl MyProcessor {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0,
            gain_smoother: 0.0,
        }
    }
}

impl Processor for MyProcessor {
    fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(&mut self, _transport: &Transport, buffer: &mut Buffer) {
        // Get parameter values (smoothed)
        let target_gain = /* get from params */;
        
        // Process each sample
        for channel in buffer.iter_mut() {
            for sample in channel.iter_mut() {
                // Simple gain with smoothing
                self.gain_smoother += 0.001 * (target_gain - self.gain_smoother);
                *sample *= self.gain_smoother;
            }
        }
    }

    fn reset(&mut self) {
        self.gain_smoother = 0.0;
    }
}
```

### Key Concepts

1. **`prepare()`** — Called when audio starts. Use for initialization.
2. **`process()`** — Called for each audio buffer. Must be real-time safe!
3. **`reset()`** — Called when playback stops. Clear delay lines, etc.
4. **`Transport`** — Contains tempo, time signature, playhead position.

### Real-Time Safety Rules

The `process()` method runs on the audio thread. You **must not**:
- Allocate memory (`Box::new`, `Vec::push`, `String::from`)
- Lock mutexes or use blocking operations
- Make system calls (file I/O, network)
- Log or print

---

## Defining Parameters

Use the `wavecraft_params!` macro to declare your parameters:

```rust
use wavecraft_core::prelude::*;

wavecraft_params! {
    Gain: { id: 0, name: "Gain", range: -24.0..=24.0, default: 0.0, unit: "dB" },
    Mix: { id: 1, name: "Mix", range: 0.0..=1.0, default: 1.0, unit: "%" },
    Frequency: { id: 2, name: "Frequency", range: 20.0..=20000.0, default: 1000.0, unit: "Hz" },
}
```

### Parameter Fields

| Field | Description |
|-------|-------------|
| `id` | Unique numeric identifier (for host automation) |
| `name` | Display name shown in UI and host |
| `range` | Value range (inclusive) |
| `default` | Initial value |
| `unit` | Unit string (dB, %, Hz, ms, etc.) |

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
      {groups.length > 0 ? (
        groups.map(group => <ParameterGroup key={group.name} group={group} />)
      ) : (
        params?.map(p => <ParameterSlider key={p.id} id={p.id} />)
      )}
    </div>
  );
}
```

### Custom Components with Hooks

```tsx
import { useParameter } from '@wavecraft/core';

function MyKnob({ id }: { id: string }) {
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
      <span>{value?.toFixed(2)} {info?.unit}</span>
    </div>
  );
}
```

### Available Hooks

All hooks are exported from `@wavecraft/core`:

| Hook | Purpose |
|------|---------|
| `useParameter(id)` | Read/write a single parameter |
| `useAllParameters()` | Fetch all plugin parameters (automatic discovery) |
| `useParameterGroups(params)` | Group parameters by their `group` attribute |
| `useMeterFrame()` | Access real-time meter data |
| `useConnectionStatus()` | WebSocket connection status (dev mode) |

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

| Command | Description |
|---------|-------------|
| `cargo xtask dev` | Start dev servers (WebSocket + Vite) |
| `cargo xtask bundle` | Build VST3/CLAP bundles |
| `cargo xtask bundle --release` | Build optimized release |
| `cargo xtask test` | Run all tests |
| `cargo xtask lint` | Run linters |
| `cargo xtask install` | Install plugins to system directories |
| `cargo xtask sign` | Sign plugins for macOS |
| `cargo xtask clean` | Clean build artifacts |

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

### Audio glitches/dropouts

1. Check for allocations in `process()` — use `#[deny(clippy::all)]`
2. Reduce buffer size in DAW to test
3. Profile with Instruments (macOS) or perf (Linux)

---

## Next Steps

- **[High-Level Design](../architecture/high-level-design.md)** — Understand the architecture
- **[Coding Standards](../architecture/coding-standards.md)** — Follow project conventions
- **[macOS Signing Guide](./macos-signing.md)** — Prepare for distribution

---

## Getting Help

- **Issues:** File bugs or feature requests on GitHub
- **Discussions:** Ask questions in GitHub Discussions
- **Examples:** See the `examples/` folder in the SDK repository
