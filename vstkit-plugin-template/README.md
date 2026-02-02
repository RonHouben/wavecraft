# My Plugin

A VST3/CLAP audio plugin built with the [VstKit SDK](https://github.com/vstkit/vstkit).

## Quick Start

### Prerequisites

- **Rust** 1.70 or later ([install](https://rustup.rs/))
- **Node.js** 18 or later ([install](https://nodejs.org/))
- **macOS** 10.13+ (Windows and Linux support coming soon)

### Get Started in 3 Steps

```bash
# 1. Install dependencies
cd ui && npm install && cd ..

# 2. Build the UI
cd ui && npm run build && cd ..

# 3. Bundle the plugin
cd engine && cargo xtask bundle --release
```

The plugin bundle is now in `engine/target/bundled/`. Copy it to your plugin folder to use in your DAW!

## Project Structure

```
my-plugin/
├── engine/           # Rust audio engine
│   ├── src/
│   │   └── lib.rs   # Main plugin code
│   ├── xtask/       # Build automation
│   └── Cargo.toml
├── ui/              # React user interface
│   ├── src/
│   │   ├── App.tsx
│   │   ├── components/
│   │   └── lib/vstkit-ipc/  # VstKit IPC client
│   ├── package.json
│   └── vite.config.ts
└── README.md
```

### Key Files

| File | Purpose |
|------|---------|
| `engine/src/lib.rs` | Main plugin implementation (DSP, parameters, nih-plug integration) |
| `ui/src/App.tsx` | User interface layout and components |
| `ui/src/lib/vstkit-ipc/` | IPC client for UI ↔ Engine communication |
| `engine/Cargo.toml` | Rust dependencies (VstKit SDK from GitHub) |

## Development Workflow

### 1. Modify Parameters

Edit `engine/src/lib.rs`:

```rust
#[derive(Params)]
pub struct MyPluginParams {
    #[id = "gain"]
    pub gain: FloatParam,
    
    // Add more parameters here
    #[id = "drive"]
    pub drive: FloatParam,
}
```

### 2. Implement DSP Logic

Update the `Processor` implementation:

```rust
impl Processor for GainProcessor {
    fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport) {
        // Your DSP code here
        for channel in buffer.iter_mut() {
            for sample in channel.iter_mut() {
                *sample *= 2.0; // Example: double the signal
            }
        }
    }
}
```

### 3. Update UI

Modify `ui/src/App.tsx` to add controls:

```tsx
<ParameterSlider id="drive" />
```

### 4. Rebuild and Test

```bash
# Rebuild UI (if you changed React code)
cd ui && npm run build && cd ..

# Rebuild plugin
cd engine && cargo build --release && cd ..

# Bundle the plugin
cd engine && cargo xtask bundle --release && cd ..
```

## Plugin Configuration

### Metadata

Edit these constants in `engine/src/lib.rs`:

```rust
impl Plugin for MyPlugin {
    const NAME: &'static str = "My Plugin";
    const VENDOR: &'static str = "My Company";
    const URL: &'static str = "https://example.com";
    const EMAIL: &'static str = "contact@example.com";
    // ...
}
```

### Audio I/O

Change the audio routing:

```rust
const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
    AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),  // Stereo in
        main_output_channels: NonZeroU32::new(2), // Stereo out
        ..AudioIOLayout::const_default()
    },
];
```

### Plugin IDs

**IMPORTANT:** Change these to unique values for your plugin:

```rust
impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "com.yourcompany.yourplugin";
    // ...
}

impl Vst3Plugin for MyPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"YourUniqueID0000"; // 16 characters
    // ...
}
```

## VstKit SDK Overview

Your plugin uses these VstKit crates (via GitHub):

| Crate | Purpose |
|-------|---------|
| `vstkit-core` | Main plugin framework, nih-plug integration, editor |
| `vstkit-protocol` | Parameter definitions and IPC contracts |
| `vstkit-dsp` | DSP traits and utilities |
| `vstkit-bridge` | IPC handler (UI ↔ Audio communication) |
| `vstkit-metering` | Real-time safe metering (SPSC ring buffer) |

### Key Traits

**`Processor` trait** (from `vstkit-dsp`):
```rust
pub trait Processor {
    fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport);
    fn set_sample_rate(&mut self, sample_rate: f32) {}
    fn reset(&mut self) {}
}
```

Implement this trait to define your DSP logic.

## Building for Distribution

```bash
cd engine && cargo xtask bundle
```

This creates:
- `engine/target/bundled/my-plugin.vst3`
- `engine/target/bundled/my-plugin.clap`

Copy these bundles to share your plugin.

### Install Locations

| Platform | VST3 | CLAP |
|----------|------|------|
| macOS | `~/Library/Audio/Plug-Ins/VST3/` | `~/Library/Audio/Plug-Ins/CLAP/` |
| Windows | `C:\Program Files\Common Files\VST3\` | `C:\Program Files\Common Files\CLAP\` |
| Linux | `~/.vst3/` | `~/.clap/` |

## Troubleshooting

### Plugin doesn't show up in DAW

1. Check bundle was created: `ls engine/target/bundled/`
2. Verify installation: `ls ~/Library/Audio/Plug-Ins/VST3/`
3. Rescan plugins in your DAW
4. Check DAW logs for errors

### UI doesn't load

1. Rebuild UI: `cd ui && npm run build`
2. Check `ui/dist/` exists and has files
3. Rebuild plugin: `cd engine && cargo build --release`

### Build errors

1. Update Rust: `rustup update`
2. Clean build: `cd engine && cargo clean`
3. Check Rust version: `rustc --version` (need 1.70+)
4. Check Node version: `node --version` (need 18+)

### Hot reload not working

The template doesn't include hot reload by default. For development with hot reload, see the [VstKit standalone mode documentation](https://github.com/vstkit/vstkit#development-mode).

## Next Steps

- Read the [VstKit SDK documentation](https://github.com/vstkit/vstkit)
- Explore the [example plugins](https://github.com/vstkit/vstkit/tree/main/examples)
- Join the [Discord community](https://discord.gg/vstkit) (if available)

## License

MIT OR Apache-2.0 (match your preference)

---

**Built with [VstKit](https://github.com/vstkit/vstkit)** — Modern audio plugin framework for Rust + React
