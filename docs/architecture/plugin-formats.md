# Plugin Formats — VST3, CLAP & AU

Wavecraft targets three plugin formats: VST3 (primary), CLAP (secondary), and AU (optional, macOS only). This document covers format-specific architecture, behavioral differences, and the AU wrapper approach.

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview, component design, data flows
- [Development Workflows](./development-workflows.md) — Build commands, signing, notarization
- [Versioning and Distribution](./versioning-and-distribution.md) — Version management and packaging

---

## Plugin Format Overview

| Aspect | VST3 | CLAP | AU (via clap-wrapper) |
|--------|------|------|----------------------|
| nih-plug support | ✅ Native | ✅ Native | ⚠️ Via clap-wrapper |
| Primary use | Ableton Live | Cross-platform | Logic Pro, GarageBand |
| Bundle extension | `.vst3` | `.clap` | `.component` |
| Platform | macOS, Windows, Linux | macOS, Windows, Linux | macOS only |

---

## Audio Unit (AU) Architecture

### Overview

Audio Units are Apple's native plugin format, required for Logic Pro and GarageBand compatibility. AU plugins are macOS-only and have distinct architectural requirements from VST3.

> **Important:** nih-plug does **NOT** support AU export directly. The project author has stated that AU support is very low priority due to AUv2's limitations and Apple's uncertain future for the format. See [nih-plug issue #63](https://github.com/robbert-vdh/nih-plug/issues/63) for details.

### AU Support via clap-wrapper

The recommended approach for AU support is to use **[clap-wrapper](https://github.com/free-audio/clap-wrapper/)**, which converts CLAP plugins to AUv2. This is the community-endorsed solution for nih-plug projects needing AU compatibility.

**How it works:**
1. nih-plug exports a `.clap` plugin (fully supported)
2. clap-wrapper (CMake-based) wraps the CLAP binary into an AUv2 `.component` bundle
3. The resulting AU plugin runs in Logic Pro, GarageBand, and other AU hosts

### AU-Specific Requirements

1. **Bundle Structure**
   - AU plugins are packaged as `.component` bundles
   - Install location: `/Library/Audio/Plug-Ins/Components/` (system) or `~/Library/Audio/Plug-Ins/Components/` (user)
   - Bundle must contain `Info.plist` with AU-specific keys (manufacturer code, subtype, type)

2. **Four-Character Codes (4CC)**
   - AU uses 4-character codes for identification:
     - **Manufacturer code**: Unique 4-char identifier (e.g., `'VstK'`)
     - **Subtype code**: Plugin-specific identifier (e.g., `'vsk1'`)
     - **Type code**: `'aufx'` for effects, `'aumu'` for instruments, `'aumf'` for MIDI effects
   - These must be registered with Apple for commercial distribution

3. **Component Registration**
   - macOS caches AU plugins; use `auval` to validate and `killall -9 AudioComponentRegistrar` to refresh cache during development
   - Plugin must pass `auval -v aufx <subtype> <manufacturer>` validation

### clap-wrapper Integration

To convert a nih-plug CLAP plugin to AUv2 using clap-wrapper:

**1. Prerequisites:**
- CMake 3.15+
- Xcode command-line tools (macOS)
- The `.clap` plugin built from nih-plug

**2. CMakeLists.txt Configuration:**

```cmake
cmake_minimum_required(VERSION 3.15)
project(wavecraft) # <-- your project name

# This CMakeLists.txt converts a .clap file into an AUv2 plugin (.component)

# Path to the CLAP plugin built by nih-plug
set(CLAP_PLUGIN_PATH "${CMAKE_CURRENT_SOURCE_DIR}/Wavecraft.clap") # <-- adjust path

# Required for AU SDK on macOS
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_OSX_DEPLOYMENT_TARGET 11.0) # or your minimum target
set(CMAKE_OSX_ARCHITECTURES "arm64;x86_64") # universal binary
enable_language(OBJC)
enable_language(OBJCXX)

# clap-wrapper as submodule (https://github.com/free-audio/clap-wrapper/)
set(CLAP_WRAPPER_DOWNLOAD_DEPENDENCIES ON)
add_subdirectory(clap-wrapper)

# Create AUv2 target
set(AUV2_TARGET ${PROJECT_NAME}_auv2)
add_library(${AUV2_TARGET} MODULE)

# Populate the AUv2 target using clap-wrapper
target_add_auv2_wrapper(
    TARGET ${AUV2_TARGET}
    MACOS_EMBEDDED_CLAP_LOCATION ${CLAP_PLUGIN_PATH}
    
    # AU metadata (must match your plugin)
    OUTPUT_NAME "Wavecraft"
    BUNDLE_IDENTIFIER "com.yourcompany.wavecraft"
    BUNDLE_VERSION "1.0"
    MANUFACTURER_NAME "Your Company"
    MANUFACTURER_CODE "VstK"  # 4-char code
    SUBTYPE_CODE "vsk1"       # 4-char code
    INSTRUMENT_TYPE "aufx"    # aufx=effect, aumu=instrument
)
```

**3. Build Commands:**

```bash
# Build everything (tests → bundle → AU → install)
cd engine
cargo xtask all

# Or step-by-step:
cargo xtask bundle --release    # Build VST3/CLAP
cargo xtask au                  # Build AU wrapper (macOS)
cargo xtask install             # Install to system directories

# Preview what would happen without executing
cargo xtask all --dry-run
```

**4. Important Notes:**
- The `.clap` bundle is embedded inside the `.component` bundle
- Both the inner CLAP and outer AU must be signed for notarization
- Use `cmake --build build --clean-first` if `auval` doesn't detect updates
- Restart the host or run `killall -9 AudioComponentRegistrar` after rebuilds

### Why Not Native AU in nih-plug?

The nih-plug maintainer has explained the rationale ([issue #63](https://github.com/robbert-vdh/nih-plug/issues/63)):

1. **Limited benefit**: AU is only needed for Logic Pro (other DAWs support VST3/CLAP)
2. **API limitations**: AUv2 is older and less flexible than CLAP/VST3
3. **Parameter restrictions**: AU requires linear parameter enumeration with manual tombstones
4. **Uncertain future**: Apple previously deprecated then un-deprecated AUv2
5. **AUv3 issues**: AUv3 has no easy C API and runs in separate processes

The clap-wrapper approach provides AU compatibility without these architectural compromises.

### AU vs VST3 vs CLAP Behavioral Differences

| Aspect | VST3 | AU (via clap-wrapper) | CLAP (native) |
|--------|------|----------------------|---------------|
| Parameter IDs | 32-bit integers | 32-bit integers (AudioUnitParameterID) | String-based IDs |
| Parameter ranges | Arbitrary float | Arbitrary float | Arbitrary float |
| Preset format | `.vstpreset` | `.aupreset` (property list) | Host-dependent |
| State persistence | Binary blob via `IEditController` | Property list via `kAudioUnitProperty_ClassInfo` | Binary blob |
| UI hosting | `IPlugView` interface | `AudioUnitCocoaView` protocol | `clap_plugin_gui` |
| Sidechain | Explicit bus configuration | `kAudioUnitProperty_SupportedChannelLayoutTags` | Audio ports |
| Latency reporting | `IComponent::getLatencySamples()` | `kAudioUnitProperty_Latency` | `clap_plugin_latency` |
| Tail time | `IAudioProcessor::getTailSamples()` | `kAudioUnitProperty_TailTime` | `clap_plugin_tail` |
| nih-plug support | ✅ Native | ⚠️ Via clap-wrapper | ✅ Native |

### AU-Specific Constraints

1. **Threading Model**
   - AU hosts may call render from any thread (not guaranteed to be the same thread)
   - Render callback must be fully reentrant
   - UI updates must dispatch to main thread via GCD

2. **Real-Time Thread Priority**
   - AU render threads run at real-time priority
   - Same real-time safety rules apply: no allocations, no locks, no syscalls

3. **View Lifecycle**
   - Cocoa views must handle `viewDidMoveToWindow` for cleanup
   - WebView embedding requires careful management of `NSView` lifecycle
   - Views may be created/destroyed multiple times during plugin lifetime

### Logic Pro Specific Notes

- Logic Pro has stricter AU validation than other hosts
- Always test with `auval` before Logic Pro testing
- Logic Pro 10.5+ requires notarized plugins on macOS 10.15+
- Logic Pro caches plugin state aggressively; restart Logic after plugin updates

---

## Testing Matrix (focused on macOS + Ableton)

> **Primary target:** macOS + Ableton Live. Other hosts and platforms are deprioritized.

### Primary (Required)
	•	**Ableton Live (macOS, VST3)** — primary host, must work flawlessly
	•	Buffer/CPU tests: low buffer sizes (32/64) and high CPU stress to detect audio dropouts
	•	Automation tests: host automation read/write roundtrip verified
	•	UI tests: verify parameter updates from host appear in UI and UI changes are streamed back to host automation

### Secondary (Nice-to-Have)
	•	Logic Pro (macOS, AU) — requires AU via clap-wrapper
	•	GarageBand (macOS, AU)
	•	AU validation: `auval -v aufx <subtype> <manufacturer>` must pass with no errors
	•	AU-specific tests: load in AU Lab, state save/restore, bypass state

### Deprioritized (Future Consideration)
	•	Ableton Live (Windows)
	•	Reaper (all platforms)
	•	Cubase, FL Studio
	•	Linux hosts
	•	Platform checklists for Windows (WebView2) and Linux (WebKitGTK)
