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
/// This macro parses a simple DSL and generates all the boilerplate code for
/// a working VST3/CLAP plugin.
///
/// # Syntax
///
/// ```text
/// wavecraft_plugin! {
///     name: "My Plugin",
///     vendor: "My Company",
///     url: "https://example.com",  // optional
///     email: "info@example.com",   // optional
///     signal: Chain![
///         MyGain { level: 0.0 },
///     ],
/// }
/// ```
///
/// # Phase 6 Status
///
/// This is a work-in-progress implementation. Current status:
/// - [x] Step 6.1: Basic input parsing (name, vendor, signal)
/// - [ ] Step 6.2: Generate Plugin struct (partially done)
/// - [ ] Step 6.3: Generate Params struct from processor chain
/// - [ ] Step 6.4: Generate Plugin trait impl (partially done)
/// - [ ] Step 6.5: Generate format impls & exports (done)
/// - [ ] Step 6.6: Add error messages
#[proc_macro]
pub fn wavecraft_plugin(input: TokenStream) -> TokenStream {
    plugin::wavecraft_plugin_impl(input)
}
