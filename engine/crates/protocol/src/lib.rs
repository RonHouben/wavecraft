//! Protocol crate - shared parameter definitions and contracts.
//!
//! This crate contains the canonical definitions for all parameters,
//! ensuring consistency between DSP, plugin, and UI layers.

pub mod params;

pub use params::{db_to_linear, ParamId, ParamSpec, PARAM_SPECS};
