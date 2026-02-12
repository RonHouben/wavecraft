//! Example plugin for the Wavecraft SDK.
//!
//! This crate serves as:
//! 1. **Integration example** — Demonstrates minimal plugin structure for SDK contributors
//! 2. **Development target** — Enables `cargo xtask dev` from SDK repository root
//! 3. **Template parity check** — Mirrors the structure in `cli/sdk-templates/new-project/react/`
//!
//! When changes are made to the plugin template, this crate should be updated to match,
//! ensuring the SDK's own development workflow stays consistent with user project structure.

use wavecraft::prelude::*;

wavecraft_processor!(InputGain => Gain);
wavecraft_processor!(OutputGain => Gain);
wavecraft_processor!(AnotherGain => Gain);

wavecraft_plugin! {
    name: "Wavecraft Example",
    signal: SignalChain![InputGain, AnotherGain, OutputGain],
}
