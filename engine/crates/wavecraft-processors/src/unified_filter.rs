//! Unified filter processor (LP/HP/BP) with enum mode selection.

use core::f32::consts::PI;
use wavecraft_dsp::{ParamRange, ParamSpec, Processor, ProcessorParams, Transport};

const MIN_CUTOFF_HZ: f32 = 20.0;
const MIN_Q: f32 = 0.1;
const MAX_Q: f32 = 10.0;
const DEFAULT_CUTOFF_HZ: f32 = 1_000.0;
const DEFAULT_Q: f32 = 0.707;
const MAX_FILTER_CHANNELS: usize = 8;

/// Unified filter mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnifiedFilterMode {
    #[default]
    LowPass,
    HighPass,
    BandPass,
}

impl UnifiedFilterMode {
    fn from_index(index: i32) -> Self {
        match index {
            1 => Self::HighPass,
            2 => Self::BandPass,
            _ => Self::LowPass,
        }
    }
}

/// Parameters for unified filter processor.
#[derive(Debug, Clone)]
pub struct UnifiedFilterParams {
    /// Cutoff frequency in Hz.
    pub cutoff_hz: f32,
    /// Resonance/Q factor.
    pub resonance_q: f32,
    /// Filter mode.
    pub mode: UnifiedFilterMode,
}

impl Default for UnifiedFilterParams {
    fn default() -> Self {
        Self::from_param_defaults()
    }
}

impl ProcessorParams for UnifiedFilterParams {
    fn param_specs() -> &'static [ParamSpec] {
        static MODES: [&str; 3] = ["Low-pass", "High-pass", "Band-pass"];
        static SPECS: [ParamSpec; 3] = [
            ParamSpec {
                name: "Mode",
                id_suffix: "mode",
                range: ParamRange::Enum { variants: &MODES },
                default: 0.0,
                unit: "",
                group: Some("Filter"),
            },
            ParamSpec {
                name: "Cutoff",
                id_suffix: "cutoff_hz",
                range: ParamRange::Skewed {
                    min: MIN_CUTOFF_HZ as f64,
                    max: 20_000.0,
                    factor: 2.5,
                },
                default: DEFAULT_CUTOFF_HZ as f64,
                unit: "Hz",
                group: Some("Filter"),
            },
            ParamSpec {
                name: "Resonance",
                id_suffix: "resonance_q",
                range: ParamRange::Linear {
                    min: MIN_Q as f64,
                    max: MAX_Q as f64,
                },
                default: DEFAULT_Q as f64,
                unit: "Q",
                group: Some("Filter"),
            },
        ];

        &SPECS
    }

    fn from_param_defaults() -> Self {
        Self {
            cutoff_hz: DEFAULT_CUTOFF_HZ,
            resonance_q: DEFAULT_Q,
            mode: UnifiedFilterMode::LowPass,
        }
    }

    fn apply_plain_values(&mut self, values: &[f32]) {
        if let Some(mode) = values.first() {
            self.mode = UnifiedFilterMode::from_index(mode.round() as i32);
        }
        if let Some(cutoff_hz) = values.get(1) {
            self.cutoff_hz = *cutoff_hz;
        }
        if let Some(resonance_q) = values.get(2) {
            self.resonance_q = *resonance_q;
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

/// Unified biquad filter DSP processor.
#[derive(Debug)]
pub struct UnifiedFilterDsp {
    sample_rate_hz: f32,
    state: [BiquadState; MAX_FILTER_CHANNELS],
}

impl Default for UnifiedFilterDsp {
    fn default() -> Self {
        Self {
            sample_rate_hz: 44_100.0,
            state: [BiquadState::default(); MAX_FILTER_CHANNELS],
        }
    }
}

impl Processor for UnifiedFilterDsp {
    type Params = UnifiedFilterParams;

    fn process(
        &mut self,
        buffer: &mut [&mut [f32]],
        _transport: &Transport,
        params: &Self::Params,
    ) {
        if buffer.is_empty() {
            return;
        }

        let coeffs = compute_coefficients(
            self.sample_rate_hz,
            params.cutoff_hz,
            params.resonance_q,
            params.mode,
        );

        for (channel_index, channel) in buffer.iter_mut().enumerate() {
            let state_index = channel_index.min(MAX_FILTER_CHANNELS - 1);
            let state = &mut self.state[state_index];

            for sample in channel.iter_mut() {
                *sample = state.process_sample(*sample, coeffs);
            }
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate_hz = sample_rate.max(1.0);
    }

    fn reset(&mut self) {
        self.state = [BiquadState::default(); MAX_FILTER_CHANNELS];
    }
}

fn compute_coefficients(
    sample_rate_hz: f32,
    cutoff_hz: f32,
    resonance_q: f32,
    mode: UnifiedFilterMode,
) -> BiquadCoefficients {
    let sample_rate_hz = sample_rate_hz.max(1.0);
    let nyquist_hz = (sample_rate_hz * 0.5).max(MIN_CUTOFF_HZ + 1.0);
    let cutoff_hz = cutoff_hz.clamp(MIN_CUTOFF_HZ, nyquist_hz - 1.0);
    let q = resonance_q.clamp(MIN_Q, MAX_Q);

    let omega = 2.0 * PI * cutoff_hz / sample_rate_hz;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * q);

    let (b0, b1, b2, a0, a1, a2) = match mode {
        UnifiedFilterMode::LowPass => (
            (1.0 - cos_omega) * 0.5,
            1.0 - cos_omega,
            (1.0 - cos_omega) * 0.5,
            1.0 + alpha,
            -2.0 * cos_omega,
            1.0 - alpha,
        ),
        UnifiedFilterMode::HighPass => (
            (1.0 + cos_omega) * 0.5,
            -(1.0 + cos_omega),
            (1.0 + cos_omega) * 0.5,
            1.0 + alpha,
            -2.0 * cos_omega,
            1.0 - alpha,
        ),
        UnifiedFilterMode::BandPass => (
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

#[cfg(test)]
mod tests {
    use super::*;

    fn process_mono(
        processor: &mut UnifiedFilterDsp,
        mode: UnifiedFilterMode,
        cutoff_hz: f32,
        resonance_q: f32,
        input: &[f32],
    ) -> Vec<f32> {
        let mut channel = input.to_vec();
        let mut buffer = [&mut channel[..]];
        let params = UnifiedFilterParams {
            mode,
            cutoff_hz,
            resonance_q,
        };
        processor.process(&mut buffer, &Transport::default(), &params);
        channel
    }

    #[test]
    fn param_specs_use_expected_suffixes_and_groups() {
        let specs = UnifiedFilterParams::param_specs();
        assert_eq!(specs.len(), 3);
        assert_eq!(specs[0].id_suffix, "mode");
        assert_eq!(specs[1].id_suffix, "cutoff_hz");
        assert_eq!(specs[2].id_suffix, "resonance_q");
        assert_eq!(specs[1].group, Some("Filter"));
        assert_eq!(specs[2].unit, "Q");
    }

    #[test]
    fn apply_plain_values_maps_mode_index() {
        let mut params = UnifiedFilterParams::from_param_defaults();
        params.apply_plain_values(&[2.0, 2000.0, 1.25]);

        assert_eq!(params.mode, UnifiedFilterMode::BandPass);
        assert!((params.cutoff_hz - 2000.0).abs() < f32::EPSILON);
        assert!((params.resonance_q - 1.25).abs() < f32::EPSILON);
    }

    #[test]
    fn lowpass_preserves_dc_more_than_highpass() {
        let input = vec![1.0_f32; 256];

        let mut lowpass = UnifiedFilterDsp::default();
        lowpass.set_sample_rate(48_000.0);
        let low = process_mono(
            &mut lowpass,
            UnifiedFilterMode::LowPass,
            800.0,
            0.707,
            &input,
        );

        let mut highpass = UnifiedFilterDsp::default();
        highpass.set_sample_rate(48_000.0);
        let high = process_mono(
            &mut highpass,
            UnifiedFilterMode::HighPass,
            800.0,
            0.707,
            &input,
        );

        let low_tail = low.last().copied().unwrap_or_default().abs();
        let high_tail = high.last().copied().unwrap_or_default().abs();
        assert!(low_tail > high_tail);
    }

    #[test]
    fn bandpass_rejects_dc() {
        let input = vec![1.0_f32; 256];

        let mut bandpass = UnifiedFilterDsp::default();
        bandpass.set_sample_rate(48_000.0);
        let output = process_mono(
            &mut bandpass,
            UnifiedFilterMode::BandPass,
            1000.0,
            0.707,
            &input,
        );

        let tail = output.last().copied().unwrap_or_default().abs();
        assert!(tail < 0.1);
    }
}
