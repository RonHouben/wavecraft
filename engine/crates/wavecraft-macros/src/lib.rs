//! Wavecraft procedural macros for plugin development.
//!
//! This crate provides derive macros and procedural macros that simplify
//! audio plugin creation by automatically generating boilerplate code.

extern crate proc_macro;

mod plugin;
mod processor_params;

use proc_macro::TokenStream;

/// Derive macro for implementing `ProcessorParams` trait.
///
/// This macro automatically generates parameter metadata from struct fields
/// annotated with `#[param(...)]` attributes.
///
/// # Example
///
/// ```text
/// use wavecraft_macros::ProcessorParams;
///
/// #[derive(ProcessorParams, Default)]
/// struct GainParams {
///     #[param(range = "0.0..=2.0", default = 1.0, unit = "x")]
///     level: f32,
/// }
/// ```
#[proc_macro_derive(ProcessorParams, attributes(param))]
pub fn derive_processor_params(input: TokenStream) -> TokenStream {
    processor_params::derive(input)
}

/// Procedural macro for generating complete plugin implementations.
///
/// This macro parses a minimal DSL and generates all the boilerplate code for
/// a working VST3/CLAP plugin. Plugin metadata is automatically derived from
/// your `Cargo.toml`.
///
/// # Syntax (0.9.0+)
///
/// ```text
/// use wavecraft::prelude::*;
///
/// // Define your processor wrapper
/// wavecraft_processor!(MyGain => Gain);
///
/// // Generate complete plugin with minimal boilerplate
/// wavecraft_plugin! {
///     name: "My Plugin",
///     signal: SignalChain![MyGain],
/// }
/// ```
///
/// # Metadata Derivation
///
/// The macro automatically derives plugin metadata from `Cargo.toml`:
/// - **Vendor**: First author in `authors` field (or "Unknown")
/// - **URL**: `homepage` field (or `repository` if homepage empty)
/// - **Email**: Extracted from first author's email in `authors` field
/// - **Version**: `version` field (via `CARGO_PKG_VERSION`)
///
/// Add metadata to your `Cargo.toml`:
///
/// ```toml
/// [package]
/// authors = ["Your Name <your.email@example.com>"]
/// homepage = "https://yourproject.com"
/// ```
///
/// # Optional `crate` Property
///
/// If you've renamed the `wavecraft` dependency in your `Cargo.toml`:
///
/// ```toml
/// [dependencies]
/// my_wavecraft = { package = "wavecraft-nih_plug", ... }
/// ```
///
/// Then specify the renamed crate:
///
/// ```text
/// wavecraft_plugin! {
///     name: "My Plugin",
///     signal: SignalChain![MyGain],
///     crate: my_wavecraft,  // Optional
/// }
/// ```
///
/// # Breaking Changes (0.9.0)
///
/// - Removed `vendor`, `url`, `email` properties (now auto-derived)
/// - Bare processors no longer accepted: use `SignalChain![...]` wrapper
/// - VST3 Class IDs now use package name instead of vendor (plugins get new IDs)
/// - Default `crate` path changed from `::wavecraft_nih_plug` to `::wavecraft`
///
/// See `docs/MIGRATION-0.9.md` for migration guide.
///
/// # Known Limitations
///
/// **Parameter Automation**: The DSL-generated `process()` method always receives
/// default parameter values. Host automation and UI work correctly, but the
/// `Processor` cannot read parameter changes.
///
/// **Workaround**: For parameter-driven DSP, implement the `Plugin` trait directly
/// instead of using this macro. See SDK documentation for examples.
///
/// **Tracking**: This limitation is tracked in the SDK roadmap for future releases.
#[proc_macro]
pub fn wavecraft_plugin(input: TokenStream) -> TokenStream {
    plugin::wavecraft_plugin_impl(input)
}
