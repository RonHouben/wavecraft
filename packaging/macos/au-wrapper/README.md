# Wavecraft AU (Audio Unit v2) Wrapper

This directory contains the CMake configuration for building the Wavecraft Audio Unit plugin
using [clap-wrapper](https://github.com/free-audio/clap-wrapper/).

## Overview

nih-plug does NOT support AU export directly. This wrapper uses the clap-wrapper project
to convert the Wavecraft CLAP plugin into an AUv2 plugin for use in:
- GarageBand
- Logic Pro
- Other AU-compatible hosts on macOS

**⚠️ Known Limitation:** clap-wrapper generates a generic AU parameter interface and does 
not forward the custom CLAP GUI. The AU plugin will show a standard parameter view, while 
VST3/CLAP formats display the custom React UI. This is expected behavior.

## Prerequisites

- macOS (AU is macOS-only)
- CMake 3.21+
- Xcode command line tools (`xcode-select --install`)
- Wavecraft CLAP plugin built (`cargo xtask bundle plugin --release` in engine/)

## Build Instructions

1. **First, build the CLAP plugin** (if not already built):
   ```bash
   cd ../../../engine
   cargo xtask bundle plugin --release
   ```

2. **Configure the AU wrapper**:
   ```bash
   cd packaging/macos/au-wrapper
   cmake -B build
   ```

3. **Build the AU plugin**:
   ```bash
   cmake --build build
   ```

4. **Install to the system AU folder**:
   ```bash
   cmake --build build --target install-au
   ```
   This copies the plugin to `~/Library/Audio/Plug-Ins/Components/` and refreshes
   the macOS plugin cache.

5. **Validate with auval** (required before testing in DAWs):
   ```bash
   cmake --build build --target validate-au
   ```
   Or manually (note: subtype is auto-generated from CLAP ID):
   ```bash
   auval -v aufx G0CJ VstK
   ```

## AU Metadata

| Property | Value | Notes |
|----------|-------|-------|
| Output Name | Wavecraft | Plugin display name |
| Bundle ID | dev.wavecraft.wavecraft | macOS bundle identifier |
| Manufacturer Code | VstK | 4-char code (should be registered with Apple) |
| Subtype Code | G0CJ | Auto-generated from CLAP ID hash by clap-wrapper |
| Type | aufx | Audio effect (use `aumu` for instruments) |

## Troubleshooting

### Plugin not appearing in DAW
1. Ensure auval validation passes
2. Run `killall -9 AudioComponentRegistrar` to refresh the AU cache
3. Restart the DAW

### auval fails to find plugin
1. Verify the plugin is installed in `~/Library/Audio/Plug-Ins/Components/`
2. Run a clean rebuild: `cmake --build build --clean-first`
3. Check that the 4-char codes match exactly

### Build errors
1. Ensure CLAP plugin exists at the expected path
2. Check CMake version: `cmake --version` (must be 3.21+)
3. Verify Xcode tools: `xcode-select -p`

## Directory Structure

```
au-wrapper/
├── CMakeLists.txt    # Main build configuration
├── README.md         # This file
└── build/            # Build output (created by cmake)
    └── Wavecraft.component/  # The built AU plugin
```

## References

- [clap-wrapper documentation](https://github.com/free-audio/clap-wrapper/wiki)
- [Apple Audio Unit documentation](https://developer.apple.com/documentation/audiounit)
- [auval man page](https://ss64.com/osx/auval.html)
