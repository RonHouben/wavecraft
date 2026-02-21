use wavecraft_processors::{Waveform, generate_waveform_sample};
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
const OSCILLATOR_WAVEFORM_PARAM_ID: &str = "oscillator_waveform";

const OSCILLATOR_FREQUENCY_MIN_HZ: f32 = 20.0;
const OSCILLATOR_FREQUENCY_MAX_HZ: f32 = 5_000.0;
const OSCILLATOR_FREQUENCY_FALLBACK_HZ: f32 = 440.0;
const OSCILLATOR_LEVEL_MIN: f32 = 0.0;
const OSCILLATOR_LEVEL_MAX: f32 = 1.0;
const OSCILLATOR_LEVEL_FALLBACK: f32 = 0.0;

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
    oscillator_phase: &mut f32,
    sample_rate: f32,
) {
    let mut tone_filter_state = StereoToneFilterState::default();
    apply_output_modifiers_with_state(
        left,
        right,
        param_bridge,
        oscillator_phase,
        sample_rate,
        &mut tone_filter_state,
    );
}

pub(super) fn apply_output_modifiers_with_state(
    left: &mut [f32],
    right: &mut [f32],
    param_bridge: &AtomicParameterBridge,
    oscillator_phase: &mut f32,
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

    // Temporary dedicated control for sdk-template oscillator source.
    // 1.0 = on, 0.0 = off.
    if let Some(enabled) = param_bridge.read("oscillator_enabled")
        && enabled < 0.5
    {
        left.fill(0.0);
        right.fill(0.0);
        apply_gain(left, right, combined_gain);
        return;
    }

    // Focused dev-mode bridge for sdk-template oscillator parameters while
    // full generic FFI parameter injection is still being implemented.
    let oscillator_frequency = param_bridge.read("oscillator_frequency");
    let oscillator_level = param_bridge.read("oscillator_level");
    let oscillator_waveform = param_bridge
        .read(OSCILLATOR_WAVEFORM_PARAM_ID)
        .unwrap_or(0.0);

    if let (Some(frequency), Some(level)) = (oscillator_frequency, oscillator_level) {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            apply_gain(left, right, combined_gain);
            return;
        }

        let clamped_frequency = normalize_oscillator_frequency(frequency);
        let clamped_level = normalize_oscillator_level(level);

        let phase_delta = clamped_frequency / sample_rate;
        let mut phase = normalize_phase(*oscillator_phase);
        let waveform = Waveform::from_index(oscillator_waveform);

        for (left_sample, right_sample) in left.iter_mut().zip(right.iter_mut()) {
            let sample = generate_waveform_sample(waveform, phase) * clamped_level;
            *left_sample = sample;
            *right_sample = sample;

            advance_phase(&mut phase, phase_delta);
        }

        *oscillator_phase = phase;
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

fn normalize_oscillator_frequency(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(OSCILLATOR_FREQUENCY_MIN_HZ, OSCILLATOR_FREQUENCY_MAX_HZ)
    } else {
        OSCILLATOR_FREQUENCY_FALLBACK_HZ
    }
}

fn normalize_oscillator_level(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(OSCILLATOR_LEVEL_MIN, OSCILLATOR_LEVEL_MAX)
    } else {
        OSCILLATOR_LEVEL_FALLBACK
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
    use super::{StereoToneFilterState, apply_output_modifiers, apply_output_modifiers_with_state};
    use crate::audio::atomic_params::AtomicParameterBridge;
    use wavecraft_protocol::{ParameterInfo, ParameterType};

    fn bridge_with_enabled(default_value: f32) -> AtomicParameterBridge {
        AtomicParameterBridge::new(&[ParameterInfo {
            id: "oscillator_enabled".to_string(),
            name: "Enabled".to_string(),
            param_type: ParameterType::Float,
            value: default_value,
            default: default_value,
            unit: Some("%".to_string()),
            min: 0.0,
            max: 1.0,
            group: Some("Oscillator".to_string()),
            variants: None,
        }])
    }

    #[test]
    fn output_modifiers_mute_when_oscillator_disabled() {
        let bridge = bridge_with_enabled(1.0);
        bridge.write("oscillator_enabled", 0.0);

        let mut left = [0.25_f32, -0.5, 0.75];
        let mut right = [0.2_f32, -0.4, 0.6];
        let mut phase = 0.0;
        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        assert!(left.iter().all(|s| s.abs() <= f32::EPSILON));
        assert!(right.iter().all(|s| s.abs() <= f32::EPSILON));
    }

    #[test]
    fn output_modifiers_keep_signal_when_oscillator_enabled() {
        let bridge = bridge_with_enabled(1.0);

        let mut left = [0.25_f32, -0.5, 0.75];
        let mut right = [0.2_f32, -0.4, 0.6];
        let mut phase = 0.0;
        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        assert_eq!(left, [0.25, -0.5, 0.75]);
        assert_eq!(right, [0.2, -0.4, 0.6]);
    }

    fn oscillator_bridge(
        frequency: f32,
        level: f32,
        waveform: f32,
        enabled: f32,
        input_trim_level: f32,
        input_trim_bypass: f32,
        output_gain_level: f32,
    ) -> AtomicParameterBridge {
        AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "oscillator_enabled".to_string(),
                name: "Enabled".to_string(),
                param_type: ParameterType::Float,
                value: enabled,
                default: enabled,
                unit: Some("%".to_string()),
                min: 0.0,
                max: 1.0,
                group: Some("Oscillator".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "oscillator_frequency".to_string(),
                name: "Frequency".to_string(),
                param_type: ParameterType::Float,
                value: frequency,
                default: frequency,
                min: 20.0,
                max: 5_000.0,
                unit: Some("Hz".to_string()),
                group: Some("Oscillator".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "oscillator_waveform".to_string(),
                name: "Waveform".to_string(),
                param_type: ParameterType::Enum,
                value: waveform,
                default: waveform,
                min: 0.0,
                max: 3.0,
                unit: None,
                group: Some("Oscillator".to_string()),
                variants: Some(vec![
                    "Sine".to_string(),
                    "Square".to_string(),
                    "Saw".to_string(),
                    "Triangle".to_string(),
                ]),
            },
            ParameterInfo {
                id: "oscillator_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: level,
                default: level,
                unit: Some("%".to_string()),
                min: 0.0,
                max: 1.0,
                group: Some("Oscillator".to_string()),
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
    fn output_modifiers_generate_runtime_oscillator_from_frequency_and_level() {
        let bridge = oscillator_bridge(880.0, 0.75, 0.0, 1.0, 1.0, 0.0, 1.0);
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

        assert!(peak_left > 0.2, "expected audible generated oscillator");
        assert!(peak_right > 0.2, "expected audible generated oscillator");
        assert_eq!(left, right, "expected in-phase stereo oscillator output");
        assert!(phase > 0.0, "phase should advance after generation");
    }

    #[test]
    fn output_modifiers_level_zero_produces_silence() {
        let bridge = oscillator_bridge(440.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0);
        let mut left = [0.1_f32; 64];
        let mut right = [0.1_f32; 64];
        let mut phase = 0.0;

        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        assert!(left.iter().all(|s| s.abs() <= f32::EPSILON));
        assert!(right.iter().all(|s| s.abs() <= f32::EPSILON));
    }

    #[test]
    fn output_modifiers_frequency_change_changes_waveform() {
        let low_freq_bridge = oscillator_bridge(220.0, 0.5, 0.0, 1.0, 1.0, 0.0, 1.0);
        let high_freq_bridge = oscillator_bridge(1760.0, 0.5, 0.0, 1.0, 1.0, 0.0, 1.0);

        let mut low_left = [0.0_f32; 256];
        let mut low_right = [0.0_f32; 256];
        let mut high_left = [0.0_f32; 256];
        let mut high_right = [0.0_f32; 256];

        let mut low_phase = 0.0;
        let mut high_phase = 0.0;

        apply_output_modifiers(
            &mut low_left,
            &mut low_right,
            &low_freq_bridge,
            &mut low_phase,
            48_000.0,
        );
        apply_output_modifiers(
            &mut high_left,
            &mut high_right,
            &high_freq_bridge,
            &mut high_phase,
            48_000.0,
        );

        assert_ne!(
            low_left, high_left,
            "frequency updates should alter waveform"
        );
        assert_eq!(low_left, low_right);
        assert_eq!(high_left, high_right);
    }

    #[test]
    fn output_modifiers_apply_input_trim_and_output_gain_levels() {
        let unity_bridge = oscillator_bridge(880.0, 0.5, 0.0, 1.0, 1.0, 0.0, 1.0);
        let boosted_bridge = oscillator_bridge(880.0, 0.5, 0.0, 1.0, 1.5, 0.0, 2.0);

        let mut unity_left = [0.0_f32; 256];
        let mut unity_right = [0.0_f32; 256];
        let mut boosted_left = [0.0_f32; 256];
        let mut boosted_right = [0.0_f32; 256];

        let mut unity_phase = 0.0;
        let mut boosted_phase = 0.0;

        apply_output_modifiers(
            &mut unity_left,
            &mut unity_right,
            &unity_bridge,
            &mut unity_phase,
            48_000.0,
        );
        apply_output_modifiers(
            &mut boosted_left,
            &mut boosted_right,
            &boosted_bridge,
            &mut boosted_phase,
            48_000.0,
        );

        let unity_peak = unity_left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
        let boosted_peak = boosted_left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        assert!(boosted_peak > unity_peak * 2.5);
        assert_eq!(boosted_left, boosted_right);
        assert_eq!(unity_left, unity_right);
    }

    #[test]
    fn output_modifiers_waveform_change_changes_shape() {
        let sine_bridge = oscillator_bridge(440.0, 0.5, 0.0, 1.0, 1.0, 0.0, 1.0);
        let saw_bridge = oscillator_bridge(440.0, 0.5, 2.0, 1.0, 1.0, 0.0, 1.0);

        let mut sine_left = [0.0_f32; 256];
        let mut sine_right = [0.0_f32; 256];
        let mut saw_left = [0.0_f32; 256];
        let mut saw_right = [0.0_f32; 256];

        let mut sine_phase = 0.0;
        let mut saw_phase = 0.0;

        apply_output_modifiers(
            &mut sine_left,
            &mut sine_right,
            &sine_bridge,
            &mut sine_phase,
            48_000.0,
        );
        apply_output_modifiers(
            &mut saw_left,
            &mut saw_right,
            &saw_bridge,
            &mut saw_phase,
            48_000.0,
        );

        assert_ne!(
            sine_left, saw_left,
            "waveform selection should change output shape"
        );
        assert_eq!(sine_left, sine_right);
        assert_eq!(saw_left, saw_right);
    }

    #[test]
    fn output_modifiers_apply_gain_without_oscillator_params() {
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

    #[test]
    fn output_modifiers_ignore_compact_legacy_gain_ids() {
        let bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "inputgain_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: 0.2,
                default: 0.2,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("InputTrim".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "outputgain_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: 0.2,
                default: 0.2,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("OutputGain".to_string()),
                variants: None,
            },
        ]);

        let mut left = [0.5_f32; 16];
        let mut right = [0.5_f32; 16];
        let mut phase = 0.0;

        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        // Legacy compact IDs are intentionally unsupported.
        let expected = 0.5;
        assert!(left.iter().all(|sample| (*sample - expected).abs() < 1e-6));
        assert!(right.iter().all(|sample| (*sample - expected).abs() < 1e-6));
    }

    #[test]
    fn output_modifiers_ignore_legacy_snake_case_gain_suffix_ids() {
        let bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "input_gain_gain".to_string(),
                name: "Gain".to_string(),
                param_type: ParameterType::Float,
                value: 0.2,
                default: 0.2,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("InputTrim".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "output_gain_gain".to_string(),
                name: "Gain".to_string(),
                param_type: ParameterType::Float,
                value: 0.2,
                default: 0.2,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("OutputGain".to_string()),
                variants: None,
            },
        ]);

        let mut left = [0.5_f32; 16];
        let mut right = [0.5_f32; 16];
        let mut phase = 0.0;

        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        // Legacy "*_gain" aliases are intentionally unsupported.
        let expected = 0.5;
        assert!(left.iter().all(|sample| (*sample - expected).abs() < 1e-6));
        assert!(right.iter().all(|sample| (*sample - expected).abs() < 1e-6));
    }

    #[test]
    fn output_modifiers_use_canonical_ids_even_when_legacy_variants_exist() {
        let bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "input_trim_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: 1.6,
                default: 1.6,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("InputTrim".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "inputgain_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: 0.4,
                default: 0.4,
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
                value: 1.0,
                default: 1.0,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("OutputGain".to_string()),
                variants: None,
            },
        ]);

        let mut left = [0.5_f32; 8];
        let mut right = [0.5_f32; 8];
        let mut phase = 0.0;

        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        // Strict canonical-only policy: legacy variants are ignored when present.
        let expected = 0.5 * 1.6;
        assert!(left.iter().all(|sample| (*sample - expected).abs() < 1e-6));
        assert!(right.iter().all(|sample| (*sample - expected).abs() < 1e-6));
    }

    #[test]
    fn output_modifiers_use_legacy_input_gain_when_canonical_missing() {
        let bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "input_gain_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: 1.8,
                default: 1.8,
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
                value: 1.0,
                default: 1.0,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("OutputGain".to_string()),
                variants: None,
            },
        ]);

        let mut left = [0.5_f32; 8];
        let mut right = [0.5_f32; 8];
        let mut phase = 0.0;

        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        let expected = 0.5 * 1.8;
        assert!(left.iter().all(|sample| (*sample - expected).abs() < 1e-6));
        assert!(right.iter().all(|sample| (*sample - expected).abs() < 1e-6));
    }

    #[test]
    fn output_modifiers_bypass_skips_input_trim_gain() {
        let enabled_trim = oscillator_bridge(880.0, 0.5, 0.0, 1.0, 1.7, 0.0, 1.0);
        let bypassed_trim = oscillator_bridge(880.0, 0.5, 0.0, 1.0, 1.7, 1.0, 1.0);

        let mut enabled_left = [0.0_f32; 256];
        let mut enabled_right = [0.0_f32; 256];
        let mut bypassed_left = [0.0_f32; 256];
        let mut bypassed_right = [0.0_f32; 256];

        let mut enabled_phase = 0.0;
        let mut bypassed_phase = 0.0;

        apply_output_modifiers(
            &mut enabled_left,
            &mut enabled_right,
            &enabled_trim,
            &mut enabled_phase,
            48_000.0,
        );
        apply_output_modifiers(
            &mut bypassed_left,
            &mut bypassed_right,
            &bypassed_trim,
            &mut bypassed_phase,
            48_000.0,
        );

        let enabled_peak = enabled_left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
        let bypassed_peak = bypassed_left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        assert!(enabled_peak > bypassed_peak * 1.5);
        assert_eq!(enabled_left, enabled_right);
        assert_eq!(bypassed_left, bypassed_right);
    }

    #[test]
    fn output_modifiers_tone_filter_mode_cutoff_and_q_affect_signal() {
        let highpass_bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "tone_filter_mode".to_string(),
                name: "Tone Filter Mode".to_string(),
                param_type: ParameterType::Enum,
                value: 1.0,
                default: 1.0,
                min: 0.0,
                max: 2.0,
                unit: None,
                group: Some("ToneFilter".to_string()),
                variants: Some(vec![
                    "LowPass".to_string(),
                    "HighPass".to_string(),
                    "BandPass".to_string(),
                ]),
            },
            ParameterInfo {
                id: "tone_filter_cutoff_hz".to_string(),
                name: "Tone Filter Cutoff".to_string(),
                param_type: ParameterType::Float,
                value: 1_000.0,
                default: 1_000.0,
                min: 20.0,
                max: 20_000.0,
                unit: Some("Hz".to_string()),
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "tone_filter_resonance_q".to_string(),
                name: "Tone Filter Resonance".to_string(),
                param_type: ParameterType::Float,
                value: 0.707,
                default: 0.707,
                min: 0.1,
                max: 10.0,
                unit: Some("Q".to_string()),
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "tone_filter_bypass".to_string(),
                name: "Tone Filter Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 1.0,
                unit: None,
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
        ]);

        let lowpass_bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "tone_filter_mode".to_string(),
                name: "Tone Filter Mode".to_string(),
                param_type: ParameterType::Enum,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 2.0,
                unit: None,
                group: Some("ToneFilter".to_string()),
                variants: Some(vec![
                    "LowPass".to_string(),
                    "HighPass".to_string(),
                    "BandPass".to_string(),
                ]),
            },
            ParameterInfo {
                id: "tone_filter_cutoff_hz".to_string(),
                name: "Tone Filter Cutoff".to_string(),
                param_type: ParameterType::Float,
                value: 1_000.0,
                default: 1_000.0,
                min: 20.0,
                max: 20_000.0,
                unit: Some("Hz".to_string()),
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "tone_filter_resonance_q".to_string(),
                name: "Tone Filter Resonance".to_string(),
                param_type: ParameterType::Float,
                value: 2.5,
                default: 2.5,
                min: 0.1,
                max: 10.0,
                unit: Some("Q".to_string()),
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "tone_filter_bypass".to_string(),
                name: "Tone Filter Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 1.0,
                unit: None,
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
        ]);

        let mut highpass_left = [1.0_f32; 256];
        let mut highpass_right = [1.0_f32; 256];
        let mut lowpass_left = [1.0_f32; 256];
        let mut lowpass_right = [1.0_f32; 256];
        let mut highpass_phase = 0.0;
        let mut lowpass_phase = 0.0;
        let mut highpass_filter_state = StereoToneFilterState::default();
        let mut lowpass_filter_state = StereoToneFilterState::default();

        apply_output_modifiers_with_state(
            &mut highpass_left,
            &mut highpass_right,
            &highpass_bridge,
            &mut highpass_phase,
            48_000.0,
            &mut highpass_filter_state,
        );
        apply_output_modifiers_with_state(
            &mut lowpass_left,
            &mut lowpass_right,
            &lowpass_bridge,
            &mut lowpass_phase,
            48_000.0,
            &mut lowpass_filter_state,
        );

        let highpass_tail = highpass_left.last().copied().unwrap_or_default().abs();
        let lowpass_tail = lowpass_left.last().copied().unwrap_or_default().abs();

        assert!(
            lowpass_tail > highpass_tail,
            "low-pass should preserve more DC than high-pass"
        );
        assert_ne!(
            lowpass_left, highpass_left,
            "mode/cutoff/q should alter filter response"
        );
        assert_eq!(highpass_left, highpass_right);
        assert_eq!(lowpass_left, lowpass_right);
    }

    #[test]
    fn output_modifiers_tone_filter_bypass_disables_filter_processing() {
        let filtered_bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "tone_filter_mode".to_string(),
                name: "Tone Filter Mode".to_string(),
                param_type: ParameterType::Enum,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 2.0,
                unit: None,
                group: Some("ToneFilter".to_string()),
                variants: Some(vec![
                    "LowPass".to_string(),
                    "HighPass".to_string(),
                    "BandPass".to_string(),
                ]),
            },
            ParameterInfo {
                id: "tone_filter_cutoff_hz".to_string(),
                name: "Tone Filter Cutoff".to_string(),
                param_type: ParameterType::Float,
                value: 500.0,
                default: 500.0,
                min: 20.0,
                max: 20_000.0,
                unit: Some("Hz".to_string()),
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "tone_filter_resonance_q".to_string(),
                name: "Tone Filter Resonance".to_string(),
                param_type: ParameterType::Float,
                value: 0.707,
                default: 0.707,
                min: 0.1,
                max: 10.0,
                unit: Some("Q".to_string()),
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "tone_filter_bypass".to_string(),
                name: "Tone Filter Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 1.0,
                unit: None,
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
        ]);

        let bypassed_bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "tone_filter_mode".to_string(),
                name: "Tone Filter Mode".to_string(),
                param_type: ParameterType::Enum,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 2.0,
                unit: None,
                group: Some("ToneFilter".to_string()),
                variants: Some(vec![
                    "LowPass".to_string(),
                    "HighPass".to_string(),
                    "BandPass".to_string(),
                ]),
            },
            ParameterInfo {
                id: "tone_filter_cutoff_hz".to_string(),
                name: "Tone Filter Cutoff".to_string(),
                param_type: ParameterType::Float,
                value: 500.0,
                default: 500.0,
                min: 20.0,
                max: 20_000.0,
                unit: Some("Hz".to_string()),
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "tone_filter_resonance_q".to_string(),
                name: "Tone Filter Resonance".to_string(),
                param_type: ParameterType::Float,
                value: 0.707,
                default: 0.707,
                min: 0.1,
                max: 10.0,
                unit: Some("Q".to_string()),
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "tone_filter_bypass".to_string(),
                name: "Tone Filter Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: 1.0,
                default: 1.0,
                min: 0.0,
                max: 1.0,
                unit: None,
                group: Some("ToneFilter".to_string()),
                variants: None,
            },
        ]);

        let mut filtered_left = [0.0_f32; 256];
        let mut filtered_right = [0.0_f32; 256];
        let mut bypassed_left = [0.0_f32; 256];
        let mut bypassed_right = [0.0_f32; 256];
        for (index, sample) in filtered_left.iter_mut().enumerate() {
            *sample = if index % 2 == 0 { 1.0 } else { -1.0 };
        }
        filtered_right.copy_from_slice(&filtered_left);
        bypassed_left.copy_from_slice(&filtered_left);
        bypassed_right.copy_from_slice(&filtered_left);

        let original_bypassed_left = bypassed_left;
        let original_bypassed_right = bypassed_right;

        let mut filtered_phase = 0.0;
        let mut bypassed_phase = 0.0;
        let mut filtered_filter_state = StereoToneFilterState::default();
        let mut bypassed_filter_state = StereoToneFilterState::default();

        apply_output_modifiers_with_state(
            &mut filtered_left,
            &mut filtered_right,
            &filtered_bridge,
            &mut filtered_phase,
            48_000.0,
            &mut filtered_filter_state,
        );
        apply_output_modifiers_with_state(
            &mut bypassed_left,
            &mut bypassed_right,
            &bypassed_bridge,
            &mut bypassed_phase,
            48_000.0,
            &mut bypassed_filter_state,
        );

        assert_eq!(bypassed_left, original_bypassed_left);
        assert_eq!(bypassed_right, original_bypassed_right);
        assert_ne!(
            filtered_left, bypassed_left,
            "filtered output should differ from bypassed output"
        );
    }

    #[test]
    fn output_modifiers_soft_clip_drive_and_output_trim_affect_signal() {
        let neutral_bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "soft_clip_bypass".to_string(),
                name: "Soft Clip Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 1.0,
                unit: None,
                group: Some("SoftClip".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_drive_db".to_string(),
                name: "Drive".to_string(),
                param_type: ParameterType::Float,
                value: 0.0,
                default: 0.0,
                min: -24.0,
                max: 24.0,
                unit: Some("dB".to_string()),
                group: Some("SoftClip".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_output_trim_db".to_string(),
                name: "Output Trim".to_string(),
                param_type: ParameterType::Float,
                value: 0.0,
                default: 0.0,
                min: -24.0,
                max: 24.0,
                unit: Some("dB".to_string()),
                group: Some("SoftClip".to_string()),
                variants: None,
            },
        ]);

        let heavy_drive_bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "soft_clip_bypass".to_string(),
                name: "Soft Clip Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 1.0,
                unit: None,
                group: Some("SoftClip".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_drive_db".to_string(),
                name: "Drive".to_string(),
                param_type: ParameterType::Float,
                value: 18.0,
                default: 18.0,
                min: -24.0,
                max: 24.0,
                unit: Some("dB".to_string()),
                group: Some("SoftClip".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_output_trim_db".to_string(),
                name: "Output Trim".to_string(),
                param_type: ParameterType::Float,
                value: -12.0,
                default: -12.0,
                min: -24.0,
                max: 24.0,
                unit: Some("dB".to_string()),
                group: Some("SoftClip".to_string()),
                variants: None,
            },
        ]);

        let mut neutral_left = [0.8_f32, -0.8, 0.6, -0.6];
        let mut neutral_right = neutral_left;
        let mut heavy_left = [0.8_f32, -0.8, 0.6, -0.6];
        let mut heavy_right = heavy_left;
        let mut neutral_phase = 0.0;
        let mut heavy_phase = 0.0;

        apply_output_modifiers(
            &mut neutral_left,
            &mut neutral_right,
            &neutral_bridge,
            &mut neutral_phase,
            48_000.0,
        );
        apply_output_modifiers(
            &mut heavy_left,
            &mut heavy_right,
            &heavy_drive_bridge,
            &mut heavy_phase,
            48_000.0,
        );

        // Neutral soft clip at 0 dB/0 dB still applies gentle non-linearity,
        // but heavy drive + negative trim should clearly reduce output level.
        let neutral_peak = neutral_left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
        let heavy_peak = heavy_left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        assert!(heavy_peak < neutral_peak);
        assert_eq!(neutral_left, neutral_right);
        assert_eq!(heavy_left, heavy_right);
    }

    #[test]
    fn output_modifiers_soft_clip_bypass_skips_processing() {
        let bypassed_bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "soft_clip_bypass".to_string(),
                name: "Soft Clip Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: 1.0,
                default: 1.0,
                min: 0.0,
                max: 1.0,
                unit: None,
                group: Some("SoftClip".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_drive_db".to_string(),
                name: "Drive".to_string(),
                param_type: ParameterType::Float,
                value: 24.0,
                default: 24.0,
                min: -24.0,
                max: 24.0,
                unit: Some("dB".to_string()),
                group: Some("SoftClip".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_output_trim_db".to_string(),
                name: "Output Trim".to_string(),
                param_type: ParameterType::Float,
                value: -24.0,
                default: -24.0,
                min: -24.0,
                max: 24.0,
                unit: Some("dB".to_string()),
                group: Some("SoftClip".to_string()),
                variants: None,
            },
        ]);

        let enabled_bridge = AtomicParameterBridge::new(&[
            ParameterInfo {
                id: "soft_clip_bypass".to_string(),
                name: "Soft Clip Bypass".to_string(),
                param_type: ParameterType::Bool,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 1.0,
                unit: None,
                group: Some("SoftClip".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_drive_db".to_string(),
                name: "Drive".to_string(),
                param_type: ParameterType::Float,
                value: 24.0,
                default: 24.0,
                min: -24.0,
                max: 24.0,
                unit: Some("dB".to_string()),
                group: Some("SoftClip".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "soft_clip_output_trim_db".to_string(),
                name: "Output Trim".to_string(),
                param_type: ParameterType::Float,
                value: -24.0,
                default: -24.0,
                min: -24.0,
                max: 24.0,
                unit: Some("dB".to_string()),
                group: Some("SoftClip".to_string()),
                variants: None,
            },
        ]);

        let mut bypassed_left = [0.5_f32, -0.5, 0.25, -0.25];
        let mut bypassed_right = bypassed_left;
        let mut enabled_left = [0.5_f32, -0.5, 0.25, -0.25];
        let mut enabled_right = enabled_left;
        let expected_bypassed_left = bypassed_left;
        let expected_bypassed_right = bypassed_right;
        let mut bypassed_phase = 0.0;
        let mut enabled_phase = 0.0;

        apply_output_modifiers(
            &mut bypassed_left,
            &mut bypassed_right,
            &bypassed_bridge,
            &mut bypassed_phase,
            48_000.0,
        );
        apply_output_modifiers(
            &mut enabled_left,
            &mut enabled_right,
            &enabled_bridge,
            &mut enabled_phase,
            48_000.0,
        );

        assert_eq!(bypassed_left, expected_bypassed_left);
        assert_eq!(bypassed_right, expected_bypassed_right);
        assert_ne!(enabled_left, bypassed_left);
    }
}
