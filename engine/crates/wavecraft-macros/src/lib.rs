//! Wavecraft procedural macros for plugin development.
//!
//! This crate provides derive macros and procedural macros that simplify
//! audio plugin creation by automatically generating boilerplate code.

extern crate proc_macro;

mod processor_params;

use proc_macro::TokenStream;

/// Derive macro for implementing `ProcessorParams` trait.
///
/// This macro automatically generates parameter metadata from struct fields
/// annotated with `#[param(...)]` attributes.
///
/// # Example
///
/// ```rust,ignore
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
