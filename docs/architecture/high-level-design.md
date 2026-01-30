# High-Level Design — React UI for a Rust VST Plugin

⸻

## Assumptions (explicit)
	•	Target hosts: major DAWs (Ableton required). Targets: Windows, macOS, Linux.
	•	Audio engine / DSP will be implemented in Rust.
	•	UI must be built in React (single-page app, built with Vite/webpack/etc) and embedded inside the plugin UI window (not a separate app).
	•	Plugin formats to support: at minimum VST3 (and ideally CLAP/AU where applicable).
	•	No Electron or Tauri; prefer a lightweight embedded webview or an equivalent approach.
	•	You want a production-ready, cross-platform approach (not just a quick POC).

⸻

## Executive summary (one paragraph)

Build the audio/DSP core and host/plugin API surface in Rust (use a modern Rust plugin framework such as nih-plug), expose a minimal parameter and event API, and embed a React frontend by bundling the built static assets into an embedded WebView runtime per platform (WebView2 on Windows, WKWebView on macOS, WebKitGTK or similar on Linux). Communicate via a well-defined IPC (JSON-RPC style) and strictly separate real-time audio thread concerns (lock-free param state and ring buffers) from UI work (runs on main/UI thread). This gives you idiomatic Rust DSP code, cross-platform UI parity, and a maintainable React codebase.  ￼

⸻

## Architecture overview (block diagram, logical)

+---------------------------------------------------------------+
| Plugin binary (single cross-platform project)                 |
|                                                               |
|  +-----------------+    +----------------------+              |
|  | Audio / DSP     |<-->| Plugin API layer     |              |
|  | (Rust, real-    |    | (VST3/CLAP glue)     |              |
|  |  time thread)   |    +----------------------+              |
|  |                 |               ^                       |
|  |  - param atoms  |               | parameter/automation |
|  |  - process()    |               | events                |
|  +-----------------+               v                       |
|                               +----------------+            |
|                               | UI Bridge /    |            |
|  (lock-free queue) <--------> | IPC / messaging | <---+      |
|                               +----------------+     |      |
|                                                      |      |
|  +---------------------------+   +------------------------------+
|  | Embedded WebView (wry /   |   | React SPA (built static)     |
|  | platform-specific)        |   | (HTML/CSS/JS) bundled inside | 
|  | - WKWebView (macOS)       |   |   plugin binary or resources |
|  | - WebView2 (Windows)      |   | - uses host messaging API   |
|  | - WebKitGTK / WRY (Linux) |   +------------------------------+
|  +---------------------------+
+---------------------------------------------------------------+

Key: the audio path never blocks on UI; the UI never directly runs audio code.

⸻

## Main components (concrete)
	1.	Plugin core (Rust)
	•	Use nih-plug (Rust plugin framework) to handle VST3/CLAP exports and common plumbing.  ￼
	•	Implement process() on audio thread; maintain parameter state in atomic types (float atomics) for host automation.
	2.	Plugin API layer
	•	VST3 is supported via the VST SDK; ensure build tooling for VST3 and (if needed) AU/CLAP. Follow Steinberg VST3 dev docs for GUI/parameter mapping.  ￼
	3.	UI (React)
	•	SPA built with Vite (or your preferred bundler). Produce static assets (index.html, bundle.js, CSS).
	•	Use a small runtime footprint approach: tree-shake, code-split, avoid large libraries unless necessary.
	4.	Embedded WebView layer
	•	Use a cross-platform Rust webview binding such as wry (used by Tauri) which wraps native webview engines (WebView2, WKWebView, WebKitGTK). This avoids shipping a full Chromium and keeps the binary smaller than Electron.  ￼
	•	On Windows: WebView2 (Edge/Chromium); macOS: WKWebView; Linux: WebKitGTK or an appropriate system webview.
	5.	IPC / Bridge
	•	Lightweight JSON-RPC or custom message format over the webview host-bridge (postMessage / host object). Expose a minimal API:
	•	setParameter(id, value)
	•	getParameter(id)
	•	subscribeParamChanges(ids)
	•	sendUICommand(name, payload) (non-critical)
	•	requestOfflineProcessing (if you need offline render)
	•	Keep messages small and rate-limited.
	6.	Realtime-safe comms
	•	Use a single-producer single-consumer lock-free ring buffer (SPSC) or atomic double buffer for data from audio → UI (metering, waveform snapshots). Use crates such as rtrb or other proven SPSC ring buffer crates to avoid allocations and locks on the audio thread.  ￼
	7.	Build & Packaging
	•	Rust build (Cargo) for core; CMake or a small shim for packaging VST3 (SDK). Bundle the React build output as plugin resources (embed as bytes or serve them via an in-process file server).
	•	Code signing and notarization steps for macOS; installer options for Windows (MSI) and Linux (DEB/Flatpak/AppImage).

⸻

## Data flows and timing constraints
	•	Control path (host automation): Host ↔ plugin parameter interface (VST parameters). DSP reads parameters atomically on audio thread. UI updates parameters via IPC → plugin parameter setter (which marshals to host param API). Host automation remains authoritative.
	•	Telemetry / metering path: Audio thread writes meter samples into SPSC ring buffer at low frequency (e.g., 30–60 Hz aggregated frames), UI reads from ring buffer on UI thread and renders. Never allocate or block in audio thread.
	•	Large data (waveforms, FFTs): Compute on a worker thread (non-audio) and pass snapshots via ring buffer or shared memory. UI must never request expensive DSP on audio thread.

⸻

## Implementation recommendations (practical steps)
	1.	Prototype (week 0–2)
	•	Small Rust plugin (nih-plug) that exports VST3 and shows a native placeholder GUI. Verify Ableton compatibility.  ￼
	2.	WebView POC (week 2–4)
	•	Create tiny React app and embed it via wry in a minimal Rust app (desktop window) to test IPC and packaging of static assets.
	•	Then replace desktop window with plugin window integration (see next).
	3.	Plugin UI integration (week 4–8)
	•	Integrate webview into plugin GUI code. On macOS use WKWebView, Windows use WebView2 — wry abstracts this.
	•	Implement the IPC bridge; test message roundtrip latencies.
	4.	DSP/UI synchronization
	•	Implement SPSC ring buffer for audio→UI. Precompute and downsample meter data.  ￼
	5.	Polish & Packaging
	•	Bundle static assets, sign binaries, run host compatibility tests (Ableton, Logic, Reaper, etc.).

⸻

## Trade-offs and alternatives
	•	WebView (React) pros
	•	Fast UI iteration, familiar dev tooling, powerful layout/UX.
	•	Cross-platform parity with a single codebase.
	•	WebView cons
	•	Larger binary footprint if you ship a bundled engine; differences between platform web engines can cause rendering/behavioral discrepancies. Tauri/wry uses system webviews to reduce size but inherits engine variation.  ￼
	•	Audio in WebView can be tricky (host audio routing and priorities). There are reported quirks with WebView audio behavior (e.g., how it appears in host). Test carefully.  ￼
	•	Alternative: native GUI in Rust
	•	Using GUI libs (egui/iced/VSTGUI/JUCE) gives tighter host integration and smaller runtime surprises, but you lose React developer ergonomics and need to reimplement complex UIs natively.
	•	Alternative: separate process UI (socket)
	•	Keep plugin small and spawn external UI process (Electron/Tauri). This simplifies UI development but breaks host expectations (single window), and many DAWs do not accept a plugin that spawns a separate UI process nicely; it’s generally discouraged.

⸻

## Real-time safety checklist (musts)
	•	Audio thread: no allocations, no locks, no syscalls that can block.
	•	All audio→UI data goes through preallocated lock-free structures (SPSC ring buffers).
	•	UI→audio control messages update atomic parameter state; heavy work occurs on worker threads.
	•	Limit IPC message rate (coalesce frequent UI control changes to prevent thrashing).
	•	Profiling and testing under high CPU and low buffer sizes (32/64 samples) in Ableton.

References about real-time constraints and ring buffers: best practices and Rust crates (rtrb/direct ring buffer).  ￼

⸻

## Testing matrix (must include Ableton)
	•	Host compatibility: Ableton Live (Windows/macOS), Logic Pro (macOS/AU), Reaper (all), Cubase, FL Studio.
	•	Buffer/CPU tests: low buffer sizes (32/64) and high CPU stress to detect audio dropouts.
	•	Automation tests: host automation read/write roundtrip verified.
	•	UI tests: verify parameter updates from host appear in UI and UI changes are streamed back to host automation.
	•	Platform checklists: WebView engine versions per OS (ensure WebView2 available on Windows; ensure WebKitGTK available on target Linux distros).

⸻

## Packaging & distribution notes
	•	macOS: notarization and signing required; package VST3 and AU; embed React assets into plugin bundle (.vst3 package resources).
	•	Windows: ensure WebView2 runtime installed or include evergreen bootstrap in installer; produce .dll VST3 and installer (MSI).
	•	Linux: many host distros vary; recommend shipping CLAP/VST3 and provide AppImage/Flatpak for GUI testing.

Docs for VST3 build process: Steinberg dev portal.  ￼

⸻

## Risks & mitigations
	1.	WebView audio / host integration quirks
	•	Risk: audio inside WebView might not behave as expected or may be routed differently.
	•	Mitigation: do host tests early; consider disabling in-webview audio entirely (avoiding audio playback inside the UI).
	2.	Cross-engine rendering differences
	•	Risk: CSS/JS behaves slightly differently across WebKit vs Chromium.
	•	Mitigation: constrain to common subset of web APIs; automated visual tests per platform.
	3.	Binary size / dependency issues
	•	Risk: shipping a large engine increases installer size.
	•	Mitigation: use system webviews (wry/tauri approach) to avoid bundling Chromium; selectively polyfill features.
	4.	Real-time safety mistakes
	•	Risk: accidental allocations or locks in process() cause xruns.
	•	Mitigation: code reviews, linters, try to run audio thread with sanitizers and stress tests; prefer proven patterns (preallocated buffers, atomics, SPSC ring buffers).  ￼

⸻

## Recommended libraries & tools (quick list)
	•	Audio / plugin: nih-plug (Rust).  ￼
	•	Platform webview: wry (Rust) / system WebView2 / WKWebView / WebKitGTK.  ￼
	•	Real-time buffers: rtrb or direct_ring_buffer crates (SPSC).  ￼
	•	Build: Cargo + CMake (for VST3 SDK integration).  ￼
	•	React tooling: Vite + TypeScript, bundle to static assets.

⸻

## Minimal interface contract (example)

Define JSON messages exchanged over the webview bridge. Keep it small and versioned.
	•	From UI → Host
```json
{ "type": "setParameter", "paramId": "gain", "value": 0.73 }
```

	•	From Host → UI
```json
{ "type": "paramUpdate", "paramId": "gain", "value": 0.73 }
```

	•	Meter frame (audio → UI, via ring buffer snapshot)
```json
{ "type": "meterFrame", "meters": [{ "id":"outL", "peak":0.7, "rms":0.12 }, ...], "ts": 1680000000 }
```

Version each message payload so UI and plugin can be backward compatible.

⸻

## Roadmap (suggested milestones)
	1.	Week 0–2: Rust plugin skeleton with VST3 exports (nih-plug); native placeholder UI. Confirm Ableton host load.  ￼
	2.	Week 2–4: Desktop POC: React app embedded in a Rust desktop app via wry. Test IPC patterns.  ￼
	3.	Week 4–8: Integrate webview into plugin GUI; implement param bridge and ring buffer metering.
	4.	Week 8–12: Cross-platform hardening, signing, packaging, circular testing in Ableton and other DAWs.
	5.	After: Performance tuning, UX polish, and support for AU/CLAP if needed.

⸻

## Appendix — Key references
	•	nih-plug (Rust plugin framework).  ￼
	•	Steinberg VST3 SDK & developer tutorials.  ￼
	•	Wry / Tauri webview (cross-platform Rust webview).  ￼
	•	WebView2 overview & APIs (Windows).  ￼
	•	Real-time safe ring buffer crates and patterns (rtrb, direct ring buffers).  ￼
