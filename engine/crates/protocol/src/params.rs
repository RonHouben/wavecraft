//! Parameter definitions - the single source of truth for all plugin parameters.
//!
//! This module defines parameter IDs, specifications, and conversion functions
//! that are shared across all layers of the plugin.

/// Unique identifier for each parameter.
///
/// The `#[repr(u32)]` ensures stable ABI for host compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ParamId {
    Gain = 0,
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
    id: ParamId::Gain,
    name: "Gain",
    short_name: "Gain",
    unit: "dB",
    default: 0.0,
    min: -24.0,
    max: 24.0,
    step: 0.1,
}];

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
