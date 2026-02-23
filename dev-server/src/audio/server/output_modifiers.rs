use wavecraft_protocol::db_to_linear;

use super::super::atomic_params::AtomicParameterBridge;
use std::f32::consts::PI;

const GAIN_MULTIPLIER_MIN: f32 = 0.0;
const GAIN_MULTIPLIER_MAX: f32 = 2.0;

const TONE_FILTER_MODE_PARAM_ID: &str = "tone_filter_mode";
const TONE_FILTER_CUTOFF_PARAM_ID: &str = "tone_filter_cutoff_hz";
const TONE_FILTER_RESONANCE_Q_PARAM_ID: &str = "tone_filter_resonance_q";
const TONE_FILTER_BYPASS_PARAM_ID: &str = "tone_filter_bypass";

const TONE_FILTER_MIN_CUTOFF_HZ: f32 = 20.0;
const TONE_FILTER_DEFAULT_CUTOFF_HZ: f32 = 1_000.0;
const TONE_FILTER_MIN_Q: f32 = 0.1;
const TONE_FILTER_MAX_Q: f32 = 10.0;
const TONE_FILTER_DEFAULT_Q: f32 = 0.707;

const SOFT_CLIP_BYPASS_PARAM_ID: &str = "soft_clip_bypass";
const SOFT_CLIP_DRIVE_PARAM_ID: &str = "soft_clip_drive_db";
const SOFT_CLIP_OUTPUT_TRIM_PARAM_ID: &str = "soft_clip_output_trim_db";
const SOFT_CLIP_MIN_GAIN_DB: f32 = -24.0;
const SOFT_CLIP_MAX_GAIN_DB: f32 = 24.0;

// Canonical IDs for gain controls; input trim keeps a temporary legacy fallback
// during the InputGain -> InputTrim migration for hot-reload compatibility.
const INPUT_TRIM_PARAM_ID: &str = "input_trim_level";
const INPUT_TRIM_BYPASS_PARAM_ID: &str = "input_trim_bypass";
const LEGACY_INPUT_GAIN_PARAM_ID: &str = "input_gain_level";
const OUTPUT_GAIN_PARAM_ID: &str = "output_gain_level";
const TEST_TONE_FREQUENCY_MIN_HZ: f32 = 20.0;
const TEST_TONE_FREQUENCY_MAX_HZ: f32 = 20_000.0;
const TEST_TONE_FREQUENCY_FALLBACK_HZ: f32 = 440.0;
const TEST_TONE_LEVEL_MIN: f32 = 0.0;
const TEST_TONE_LEVEL_MAX: f32 = 1.0;
const TEST_TONE_LEVEL_FALLBACK: f32 = 0.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ToneFilterMode {
    #[default]
    LowPass,
    HighPass,
    BandPass,
}

impl ToneFilterMode {
    fn from_index(index: i32) -> Self {
        match index {
            1 => Self::HighPass,
            2 => Self::BandPass,
            _ => Self::LowPass,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

#[derive(Debug, Clone, Copy)]
struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl BiquadState {
    #[inline]
    fn process_sample(&mut self, input: f32, coeffs: BiquadCoefficients) -> f32 {
        let output = coeffs.b0 * input + coeffs.b1 * self.x1 + coeffs.b2 * self.x2
            - coeffs.a1 * self.y1
            - coeffs.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }
}

#[derive(Debug, Default)]
pub(super) struct StereoToneFilterState {
    left: BiquadState,
    right: BiquadState,
}

pub(super) fn apply_output_modifiers(
    left: &mut [f32],
    right: &mut [f32],
    param_bridge: &AtomicParameterBridge,
    test_tone_phase: &mut f32,
    sample_rate: f32,
) {
    let mut tone_filter_state = StereoToneFilterState::default();
    apply_output_modifiers_with_state(
        left,
        right,
        param_bridge,
        test_tone_phase,
        sample_rate,
        &mut tone_filter_state,
    );
}

pub(super) fn apply_output_modifiers_with_state(
    left: &mut [f32],
    right: &mut [f32],
    param_bridge: &AtomicParameterBridge,
    test_tone_phase: &mut f32,
    sample_rate: f32,
    tone_filter_state: &mut StereoToneFilterState,
) {
    let input_trim_bypassed = read_bypass_state(param_bridge, INPUT_TRIM_BYPASS_PARAM_ID);
    let input_gain = if input_trim_bypassed {
        1.0
    } else {
        read_gain_multiplier_with_fallback(
            param_bridge,
            INPUT_TRIM_PARAM_ID,
            LEGACY_INPUT_GAIN_PARAM_ID,
        )
    };
    let output_gain = read_gain_multiplier(param_bridge, OUTPUT_GAIN_PARAM_ID);
    let combined_gain = input_gain * output_gain;

    // Temporary dedicated control for sdk-template test tone source.
    // 1.0 = on, 0.0 = off.
    if let Some(enabled) = param_bridge.read("test_tone_enabled")
        && enabled < 0.5
    {
        left.fill(0.0);
        right.fill(0.0);
        apply_gain(left, right, combined_gain);
        return;
    }

    // Focused dev-mode bridge for sdk-template test tone parameters while
    // full generic FFI parameter injection is still being implemented.
    let test_tone_frequency = param_bridge.read("test_tone_frequency");
    let test_tone_level = param_bridge.read("test_tone_level");

    if let (Some(frequency), Some(level)) = (test_tone_frequency, test_tone_level) {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            apply_gain(left, right, combined_gain);
            return;
        }

        let clamped_frequency = normalize_test_tone_frequency(frequency);
        let clamped_level = normalize_test_tone_level(level);

        let phase_delta = clamped_frequency / sample_rate;
        let mut phase = normalize_phase(*test_tone_phase);

        for (left_sample, right_sample) in left.iter_mut().zip(right.iter_mut()) {
            let sample = (phase * std::f32::consts::TAU).sin() * clamped_level;
            *left_sample = sample;
            *right_sample = sample;

            advance_phase(&mut phase, phase_delta);
        }

        *test_tone_phase = phase;
    }

    apply_tone_filter(left, right, param_bridge, sample_rate, tone_filter_state);
    apply_soft_clip(left, right, param_bridge);

    apply_gain(left, right, combined_gain);
}

fn read_gain_multiplier(param_bridge: &AtomicParameterBridge, id: &str) -> f32 {
    if let Some(value) = param_bridge.read(id)
        && value.is_finite()
    {
        return value.clamp(GAIN_MULTIPLIER_MIN, GAIN_MULTIPLIER_MAX);
    }

    1.0
}

fn read_gain_multiplier_with_fallback(
    param_bridge: &AtomicParameterBridge,
    primary_id: &str,
    fallback_id: &str,
) -> f32 {
    if let Some(value) = param_bridge.read(primary_id)
        && value.is_finite()
    {
        return value.clamp(GAIN_MULTIPLIER_MIN, GAIN_MULTIPLIER_MAX);
    }

    if let Some(value) = param_bridge.read(fallback_id)
        && value.is_finite()
    {
        return value.clamp(GAIN_MULTIPLIER_MIN, GAIN_MULTIPLIER_MAX);
    }

    1.0
}

fn read_bypass_state(param_bridge: &AtomicParameterBridge, id: &str) -> bool {
    param_bridge
        .read(id)
        .is_some_and(|value| value.is_finite() && value >= 0.5)
}

fn apply_gain(left: &mut [f32], right: &mut [f32], gain: f32) {
    if (gain - 1.0).abs() <= f32::EPSILON {
        return;
    }

    for (left_sample, right_sample) in left.iter_mut().zip(right.iter_mut()) {
        *left_sample *= gain;
        *right_sample *= gain;
    }
}

fn apply_tone_filter(
    left: &mut [f32],
    right: &mut [f32],
    param_bridge: &AtomicParameterBridge,
    sample_rate_hz: f32,
    state: &mut StereoToneFilterState,
) {
    if !has_tone_filter_controls(param_bridge) {
        return;
    }

    if read_bypass_state(param_bridge, TONE_FILTER_BYPASS_PARAM_ID) {
        return;
    }

    let mode = read_tone_filter_mode(param_bridge);
    let cutoff_hz = read_tone_filter_cutoff_hz(param_bridge);
    let resonance_q = read_tone_filter_resonance_q(param_bridge);
    let coeffs = compute_tone_filter_coefficients(sample_rate_hz, cutoff_hz, resonance_q, mode);

    for sample in left.iter_mut() {
        *sample = state.left.process_sample(*sample, coeffs);
    }

    for sample in right.iter_mut() {
        *sample = state.right.process_sample(*sample, coeffs);
    }
}

fn apply_soft_clip(left: &mut [f32], right: &mut [f32], param_bridge: &AtomicParameterBridge) {
    if !has_soft_clip_controls(param_bridge) {
        return;
    }

    if read_bypass_state(param_bridge, SOFT_CLIP_BYPASS_PARAM_ID) {
        return;
    }

    let drive_db = read_soft_clip_db(
        param_bridge,
        SOFT_CLIP_DRIVE_PARAM_ID,
        0.0,
        SOFT_CLIP_MIN_GAIN_DB,
        SOFT_CLIP_MAX_GAIN_DB,
    );
    let output_trim_db = read_soft_clip_db(
        param_bridge,
        SOFT_CLIP_OUTPUT_TRIM_PARAM_ID,
        0.0,
        SOFT_CLIP_MIN_GAIN_DB,
        SOFT_CLIP_MAX_GAIN_DB,
    );

    let drive = db_to_linear(drive_db);
    let output_trim = db_to_linear(output_trim_db);

    for (left_sample, right_sample) in left.iter_mut().zip(right.iter_mut()) {
        let left_driven = *left_sample * drive;
        let right_driven = *right_sample * drive;
        *left_sample = soft_clip(left_driven) * output_trim;
        *right_sample = soft_clip(right_driven) * output_trim;
    }
}

fn has_soft_clip_controls(param_bridge: &AtomicParameterBridge) -> bool {
    param_bridge.read(SOFT_CLIP_BYPASS_PARAM_ID).is_some()
        || param_bridge.read(SOFT_CLIP_DRIVE_PARAM_ID).is_some()
        || param_bridge.read(SOFT_CLIP_OUTPUT_TRIM_PARAM_ID).is_some()
}

fn read_soft_clip_db(
    param_bridge: &AtomicParameterBridge,
    id: &str,
    default: f32,
    min: f32,
    max: f32,
) -> f32 {
    param_bridge
        .read(id)
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(min, max))
        .unwrap_or(default)
}

#[inline]
fn soft_clip(input: f32) -> f32 {
    input / (1.0 + input.abs())
}

fn has_tone_filter_controls(param_bridge: &AtomicParameterBridge) -> bool {
    param_bridge.read(TONE_FILTER_MODE_PARAM_ID).is_some()
        || param_bridge.read(TONE_FILTER_CUTOFF_PARAM_ID).is_some()
        || param_bridge
            .read(TONE_FILTER_RESONANCE_Q_PARAM_ID)
            .is_some()
        || param_bridge.read(TONE_FILTER_BYPASS_PARAM_ID).is_some()
}

fn read_tone_filter_mode(param_bridge: &AtomicParameterBridge) -> ToneFilterMode {
    let mode_index = param_bridge
        .read(TONE_FILTER_MODE_PARAM_ID)
        .filter(|value| value.is_finite())
        .map_or(0, |value| value.round() as i32);

    ToneFilterMode::from_index(mode_index)
}

fn read_tone_filter_cutoff_hz(param_bridge: &AtomicParameterBridge) -> f32 {
    param_bridge
        .read(TONE_FILTER_CUTOFF_PARAM_ID)
        .filter(|value| value.is_finite())
        .unwrap_or(TONE_FILTER_DEFAULT_CUTOFF_HZ)
}

fn read_tone_filter_resonance_q(param_bridge: &AtomicParameterBridge) -> f32 {
    param_bridge
        .read(TONE_FILTER_RESONANCE_Q_PARAM_ID)
        .filter(|value| value.is_finite())
        .unwrap_or(TONE_FILTER_DEFAULT_Q)
}

fn compute_tone_filter_coefficients(
    sample_rate_hz: f32,
    cutoff_hz: f32,
    resonance_q: f32,
    mode: ToneFilterMode,
) -> BiquadCoefficients {
    let sample_rate_hz = sample_rate_hz.max(1.0);
    let nyquist_hz = (sample_rate_hz * 0.5).max(TONE_FILTER_MIN_CUTOFF_HZ + 1.0);
    let cutoff_hz = cutoff_hz.clamp(TONE_FILTER_MIN_CUTOFF_HZ, nyquist_hz - 1.0);
    let q = resonance_q.clamp(TONE_FILTER_MIN_Q, TONE_FILTER_MAX_Q);

    let omega = 2.0 * PI * cutoff_hz / sample_rate_hz;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * q);

    let (b0, b1, b2, a0, a1, a2) = match mode {
        ToneFilterMode::LowPass => (
            (1.0 - cos_omega) * 0.5,
            1.0 - cos_omega,
            (1.0 - cos_omega) * 0.5,
            1.0 + alpha,
            -2.0 * cos_omega,
            1.0 - alpha,
        ),
        ToneFilterMode::HighPass => (
            (1.0 + cos_omega) * 0.5,
            -(1.0 + cos_omega),
            (1.0 + cos_omega) * 0.5,
            1.0 + alpha,
            -2.0 * cos_omega,
            1.0 - alpha,
        ),
        ToneFilterMode::BandPass => (
            alpha,
            0.0,
            -alpha,
            1.0 + alpha,
            -2.0 * cos_omega,
            1.0 - alpha,
        ),
    };

    BiquadCoefficients {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
    }
}

fn normalize_test_tone_frequency(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(TEST_TONE_FREQUENCY_MIN_HZ, TEST_TONE_FREQUENCY_MAX_HZ)
    } else {
        TEST_TONE_FREQUENCY_FALLBACK_HZ
    }
}

fn normalize_test_tone_level(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(TEST_TONE_LEVEL_MIN, TEST_TONE_LEVEL_MAX)
    } else {
        TEST_TONE_LEVEL_FALLBACK
    }
}

fn normalize_phase(phase: f32) -> f32 {
    if phase.is_finite() { phase } else { 0.0 }
}

fn advance_phase(phase: &mut f32, phase_delta: f32) {
    *phase += phase_delta;
    if *phase >= 1.0 {
        *phase -= phase.floor();
    }
}

#[cfg(test)]
mod tests {
    use super::apply_output_modifiers;
    use crate::audio::atomic_params::AtomicParameterBridge;
    use wavecraft_protocol::{ParameterInfo, ParameterType};

    fn bridge_with_enabled(default_value: f32) -> AtomicParameterBridge {
        AtomicParameterBridge::new(&[ParameterInfo {
            id: "test_tone_enabled".to_string(),
            name: "Enabled".to_string(),
            param_type: ParameterType::Float,
            value: default_value,
            default: default_value,
            unit: Some("%".to_string()),
            min: 0.0,
            max: 1.0,
            group: Some("Test Tone".to_string()),
            variants: None,
        }])
    }

    fn test_tone_bridge(
        frequency: f32,
        level: f32,
        enabled: f32,
        input_trim_level: f32,
        input_trim_bypass: f32,
        output_gain_level: f32,
    ) -> AtomicParameterBridge {
        AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "test_tone_enabled".to_string(),
                name: "Enabled".to_string(),
                param_type: ParameterType::Float,
                value: enabled,
                default: enabled,
                unit: Some("%".to_string()),
                min: 0.0,
                max: 1.0,
                group: Some("Test Tone".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "test_tone_frequency".to_string(),
                name: "Frequency".to_string(),
                param_type: ParameterType::Float,
                value: frequency,
                default: frequency,
                min: 20.0,
                max: 20_000.0,
                unit: Some("Hz".to_string()),
                group: Some("Test Tone".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "test_tone_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: level,
                default: level,
                unit: Some("%".to_string()),
                min: 0.0,
                max: 1.0,
                group: Some("Test Tone".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "input_trim_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: input_trim_level,
                default: input_trim_level,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("InputTrim".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "input_trim_bypass".to_string(),
                name: "Input Trim Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: input_trim_bypass,
                default: input_trim_bypass,
                unit: None,
                min: 0.0,
                max: 1.0,
                group: Some("InputTrim".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "output_gain_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: output_gain_level,
                default: output_gain_level,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("OutputGain".to_string()),
                variants: None,
            },
        ])
    }

    #[test]
    fn output_modifiers_mute_when_test_tone_disabled() {
        let bridge = bridge_with_enabled(1.0);
        bridge.write("test_tone_enabled", 0.0);

        let mut left = [0.25_f32, -0.5, 0.75];
        let mut right = [0.2_f32, -0.4, 0.6];
        let mut phase = 0.0;
        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        assert!(left.iter().all(|s| s.abs() <= f32::EPSILON));
        assert!(right.iter().all(|s| s.abs() <= f32::EPSILON));
    }

    #[test]
    fn output_modifiers_generate_runtime_test_tone_from_frequency_and_level() {
        let bridge = test_tone_bridge(880.0, 0.75, 1.0, 1.0, 0.0, 1.0);
        let mut left = [0.0_f32; 128];
        let mut right = [0.0_f32; 128];
        let mut phase = 0.0;

        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        let peak_left = left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
        let peak_right = right
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        assert!(peak_left > 0.2, "expected audible generated test tone");
        assert!(peak_right > 0.2, "expected audible generated test tone");
        assert_eq!(left, right, "expected in-phase stereo test tone output");
        assert!(phase > 0.0, "phase should advance after generation");
    }

    #[test]
    fn output_modifiers_apply_gain_without_test_tone_params() {
        let bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "input_trim_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: 1.5,
                default: 1.5,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("InputTrim".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "output_gain_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: 1.2,
                default: 1.2,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("OutputGain".to_string()),
                variants: None,
            },
        ]);

        let mut left = [0.25_f32, -0.5, 0.75];
        let mut right = [0.2_f32, -0.4, 0.6];
        let mut phase = 0.0;

        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        let expected_gain = 1.5 * 1.2;
        assert_eq!(
            left,
            [
                0.25 * expected_gain,
                -0.5 * expected_gain,
                0.75 * expected_gain
            ]
        );
        assert_eq!(
            right,
            [
                0.2 * expected_gain,
                -0.4 * expected_gain,
                0.6 * expected_gain
            ]
        );
    }
}
