use wavecraft_processors::{Waveform, generate_waveform_sample};

use super::super::atomic_params::AtomicParameterBridge;

const GAIN_MULTIPLIER_MIN: f32 = 0.0;
const GAIN_MULTIPLIER_MAX: f32 = 2.0;

// Strict runtime policy: canonical IDs only (no alias/legacy fallbacks).
const INPUT_GAIN_PARAM_ID: &str = "input_gain_level";
const OUTPUT_GAIN_PARAM_ID: &str = "output_gain_level";
const OSCILLATOR_WAVEFORM_PARAM_ID: &str = "oscillator_waveform";

const OSCILLATOR_FREQUENCY_MIN_HZ: f32 = 20.0;
const OSCILLATOR_FREQUENCY_MAX_HZ: f32 = 5_000.0;
const OSCILLATOR_FREQUENCY_FALLBACK_HZ: f32 = 440.0;
const OSCILLATOR_LEVEL_MIN: f32 = 0.0;
const OSCILLATOR_LEVEL_MAX: f32 = 1.0;
const OSCILLATOR_LEVEL_FALLBACK: f32 = 0.0;

pub(super) fn apply_output_modifiers(
    left: &mut [f32],
    right: &mut [f32],
    param_bridge: &AtomicParameterBridge,
    oscillator_phase: &mut f32,
    sample_rate: f32,
) {
    let input_gain = read_gain_multiplier(param_bridge, INPUT_GAIN_PARAM_ID);
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

fn apply_gain(left: &mut [f32], right: &mut [f32], gain: f32) {
    if (gain - 1.0).abs() <= f32::EPSILON {
        return;
    }

    for (left_sample, right_sample) in left.iter_mut().zip(right.iter_mut()) {
        *left_sample *= gain;
        *right_sample *= gain;
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
    use super::apply_output_modifiers;
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
        input_gain_level: f32,
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
                id: "input_gain_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: input_gain_level,
                default: input_gain_level,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("InputGain".to_string()),
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
        let bridge = oscillator_bridge(880.0, 0.75, 0.0, 1.0, 1.0, 1.0);
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
        let bridge = oscillator_bridge(440.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let mut left = [0.1_f32; 64];
        let mut right = [0.1_f32; 64];
        let mut phase = 0.0;

        apply_output_modifiers(&mut left, &mut right, &bridge, &mut phase, 48_000.0);

        assert!(left.iter().all(|s| s.abs() <= f32::EPSILON));
        assert!(right.iter().all(|s| s.abs() <= f32::EPSILON));
    }

    #[test]
    fn output_modifiers_frequency_change_changes_waveform() {
        let low_freq_bridge = oscillator_bridge(220.0, 0.5, 0.0, 1.0, 1.0, 1.0);
        let high_freq_bridge = oscillator_bridge(1760.0, 0.5, 0.0, 1.0, 1.0, 1.0);

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
    fn output_modifiers_apply_input_and_output_gain_levels() {
        let unity_bridge = oscillator_bridge(880.0, 0.5, 0.0, 1.0, 1.0, 1.0);
        let boosted_bridge = oscillator_bridge(880.0, 0.5, 0.0, 1.0, 1.5, 2.0);

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
        let sine_bridge = oscillator_bridge(440.0, 0.5, 0.0, 1.0, 1.0, 1.0);
        let saw_bridge = oscillator_bridge(440.0, 0.5, 2.0, 1.0, 1.0, 1.0);

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
                id: "input_gain_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: 1.5,
                default: 1.5,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("InputGain".to_string()),
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
                group: Some("InputGain".to_string()),
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
                group: Some("InputGain".to_string()),
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
                id: "input_gain_level".to_string(),
                name: "Level".to_string(),
                param_type: ParameterType::Float,
                value: 1.6,
                default: 1.6,
                unit: Some("x".to_string()),
                min: 0.0,
                max: 2.0,
                group: Some("InputGain".to_string()),
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
                group: Some("InputGain".to_string()),
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
}
