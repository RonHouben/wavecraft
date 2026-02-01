//! Parameter definitions - the single source of truth for all plugin parameters.
//!
//! This module defines parameter IDs, specifications, conversion functions,
//! and the ParamSet trait for user-defined parameter sets.

/// Trait for user-defined parameter sets.
///
/// Implement this trait to define custom parameters for your plugin.
/// The trait provides type-safe access to parameter specifications.
///
/// # Example
///
/// ```rust
/// use vstkit_protocol::{ParamSet, ParamSpec, ParamId};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// #[repr(u32)]
/// pub enum MyParamId {
///     Volume = 0,
///     Pan = 1,
/// }
///
/// impl From<MyParamId> for ParamId {
///     fn from(id: MyParamId) -> Self {
///         ParamId(id as u32)
///     }
/// }
///
/// pub struct MyParams;
///
/// impl ParamSet for MyParams {
///     type Id = MyParamId;
///     
///     const SPECS: &'static [ParamSpec] = &[
///         ParamSpec {
///             id: ParamId(0),
///             name: "Volume",
///             short_name: "Vol",
///             unit: "dB",
///             default: 0.0,
///             min: -60.0,
///             max: 12.0,
///             step: 0.1,
///         },
///         ParamSpec {
///             id: ParamId(1),
///             name: "Pan",
///             short_name: "Pan",
///             unit: "",
///             default: 0.0,
///             min: -1.0,
///             max: 1.0,
///             step: 0.01,
///         },
///     ];
///     
///     fn spec(id: Self::Id) -> Option<&'static ParamSpec> {
///         Self::SPECS.iter().find(|s| s.id.0 == id as u32)
///     }
///     
///     fn iter() -> impl Iterator<Item = &'static ParamSpec> {
///         Self::SPECS.iter()
///     }
/// }
/// ```
pub trait ParamSet: 'static + Send + Sync {
    /// The parameter ID type (typically an enum).
    type Id: Copy + Into<ParamId>;
    
    /// All parameter specifications for this set.
    const SPECS: &'static [ParamSpec];
    
    /// Get the specification for a parameter by ID.
    ///
    /// # Arguments
    /// * `id` - The parameter ID
    ///
    /// # Returns
    /// The parameter specification, or `None` if the ID is invalid.
    fn spec(id: Self::Id) -> Option<&'static ParamSpec>;
    
    /// Iterate over all parameter specifications.
    fn iter() -> impl Iterator<Item = &'static ParamSpec>;
    
    /// Get the number of parameters in this set.
    fn count() -> usize {
        Self::SPECS.len()
    }
}

/// Unique identifier for each parameter.
///
/// This wraps a u32 ID for maximum flexibility. Plugin-specific parameter
/// enums can convert to this type via `Into<ParamId>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ParamId(pub u32);

impl From<u32> for ParamId {
    fn from(id: u32) -> Self {
        ParamId(id)
    }
}

/// Legacy parameter IDs for the VstKit reference implementation.
///
/// This enum is kept for backward compatibility with the existing plugin code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum VstKitParamId {
    Gain = 0,
}

impl From<VstKitParamId> for ParamId {
    fn from(id: VstKitParamId) -> Self {
        ParamId(id as u32)
    }
}

/// Specification for a single parameter.
#[derive(Debug, Clone)]
pub struct ParamSpec {
    /// Unique parameter ID
    pub id: ParamId,
    /// Full display name (e.g., "Gain")
    pub name: &'static str,
    /// Short name for compact displays (e.g., "Gain")
    pub short_name: &'static str,
    /// Unit suffix (e.g., "dB")
    pub unit: &'static str,
    /// Default value
    pub default: f32,
    /// Minimum value
    pub min: f32,
    /// Maximum value
    pub max: f32,
    /// Step size for UI controls (0.0 = continuous)
    pub step: f32,
}

/// Canonical parameter specifications.
///
/// This is the single source of truth for all parameter metadata.
pub const PARAM_SPECS: &[ParamSpec] = &[ParamSpec {
    id: ParamId(0),
    name: "Gain",
    short_name: "Gain",
    unit: "dB",
    default: 0.0,
    min: -24.0,
    max: 24.0,
    step: 0.1,
}];

/// Default parameter set for VstKit reference implementation.
pub struct VstKitParams;

impl ParamSet for VstKitParams {
    type Id = VstKitParamId;
    
    const SPECS: &'static [ParamSpec] = PARAM_SPECS;
    
    fn spec(id: Self::Id) -> Option<&'static ParamSpec> {
        Self::SPECS.iter().find(|s| s.id.0 == id as u32)
    }
    
    fn iter() -> impl Iterator<Item = &'static ParamSpec> {
        Self::SPECS.iter()
    }
}

/// Convert decibels to linear gain.
///
/// # Arguments
/// * `db` - Gain value in decibels
///
/// # Returns
/// Linear gain multiplier (e.g., 0 dB → 1.0, -6 dB → ~0.5)
///
/// # Performance
/// This function is marked `#[inline]` for use on the audio thread.
#[inline]
pub fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_to_linear_unity() {
        let result = db_to_linear(0.0);
        assert!((result - 1.0).abs() < 1e-6, "0 dB should equal 1.0 linear");
    }

    #[test]
    fn test_db_to_linear_minus_6db() {
        let result = db_to_linear(-6.0);
        // -6 dB ≈ 0.501187
        assert!(
            (result - 0.501187).abs() < 0.001,
            "-6 dB should be approximately 0.5"
        );
    }

    #[test]
    fn test_db_to_linear_plus_6db() {
        let result = db_to_linear(6.0);
        // +6 dB ≈ 1.9953
        assert!(
            (result - 1.9953).abs() < 0.001,
            "+6 dB should be approximately 2.0"
        );
    }

    #[test]
    fn test_db_to_linear_minus_infinity() {
        let result = db_to_linear(-100.0);
        assert!(result < 1e-4, "-100 dB should be nearly silent");
    }
}
