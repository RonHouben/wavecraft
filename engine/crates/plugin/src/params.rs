//! nih-plug parameter wrapper.
//!
//! Bridges the canonical parameter definitions from the protocol crate
//! to nih-plug's parameter system.

use nih_plug::prelude::*;
use protocol::PARAM_SPECS;

/// Plugin parameters wrapped for nih-plug.
#[derive(Params)]
pub struct VstKitParams {
    /// Main gain control in decibels.
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for VstKitParams {
    fn default() -> Self {
        // Get the gain spec from protocol
        let gain_spec = &PARAM_SPECS[0];

        Self {
            gain: FloatParam::new(
                gain_spec.name,
                gain_spec.default,
                FloatRange::Linear {
                    min: gain_spec.min,
                    max: gain_spec.max,
                },
            )
            .with_unit(&format!(" {}", gain_spec.unit))
            .with_step_size(gain_spec.step)
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(1)),
        }
    }
}
