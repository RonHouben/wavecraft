// Oscillator — a simple sine-wave generator.
//
// This file demonstrates how to implement a custom Processor with parameters.
// It generates a pure sine tone whose frequency and level are controllable
// from the DAW or the browser UI.
//
// Feel free to modify this file, or copy it to create new processors.
// Add new processors to `processors/mod.rs` so the rest of the crate can see them.

use wavecraft::prelude::*;

// Import the derive macro for parameter definitions.
// The prelude provides the ProcessorParams *trait*; this brings the *derive macro*.
use wavecraft::ProcessorParams;

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------
// The `#[derive(ProcessorParams)]` macro generates the `param_specs()` method
// that tells the framework (and the UI) about each knob/slider.

#[derive(ProcessorParams, Default, Clone)]
#[allow(dead_code)] // Unused in default signal chain (oscillator is commented out)
pub struct OscillatorParams {
    /// Enable/disable oscillator output (0 = off, 1 = on).
    #[param(range = "0.0..=1.0", default = 1.0, unit = "%")]
    pub enabled: f32,

    /// Frequency in Hz.  `factor = 2.5` gives a logarithmic feel in the UI.
    #[param(range = "20.0..=5000.0", default = 440.0, unit = "Hz", factor = 2.5)]
    pub frequency: f32,

    /// Output level (0 % – 100 %).
    #[param(range = "0.0..=1.0", default = 0.5, unit = "%")]
    pub level: f32,
}

// ---------------------------------------------------------------------------
// Processor
// ---------------------------------------------------------------------------

/// A minimal oscillator that produces a sine wave.
///
/// **Key concepts demonstrated here:**
/// - Phase accumulation (how to keep track of where we are in the waveform)
/// - Sample-rate awareness (frequency → phase increment)
/// - State management (`reset()` clears the phase)
#[derive(Default)]
#[allow(dead_code)] // Unused in default signal chain (oscillator is commented out)
pub struct Oscillator {
    /// Current sample rate provided by the host.
    sample_rate: f32,
    /// Phase position within one cycle (0.0 – 1.0).
    phase: f32,
}

impl Processor for Oscillator {
    type Params = OscillatorParams;

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        buffer: &mut [&mut [f32]],
        _transport: &Transport,
        params: &Self::Params,
    ) {
        if params.enabled < 0.5 {
            for channel in buffer.iter_mut() {
                channel.fill(0.0);
            }
            return;
        }

        // Guard: if set_sample_rate() hasn't been called yet, leave buffer unchanged.
        if self.sample_rate == 0.0 {
            return;
        }

        // How far the phase advances per sample:
        //   phase_delta = frequency / sample_rate
        // e.g. 440 Hz at 44 100 S/s → 0.00998 per sample
        let phase_delta = params.frequency / self.sample_rate;

        // Save the starting phase so every channel receives the same waveform.
        // Without this, the right channel would be phase-shifted relative to left.
        let start_phase = self.phase;

        for channel in buffer.iter_mut() {
            self.phase = start_phase;
            for sample in channel.iter_mut() {
                // Generate a sine wave and scale by level
                *sample = (self.phase * std::f32::consts::TAU).sin() * params.level;

                // Advance phase, wrapping at 1.0 to avoid floating-point drift
                self.phase += phase_delta;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
            }
        }
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_params(enabled: f32) -> OscillatorParams {
        OscillatorParams {
            enabled,
            frequency: 440.0,
            level: 0.5,
        }
    }

    #[test]
    fn oscillator_outputs_silence_when_disabled() {
        let mut osc = Oscillator::default();
        osc.set_sample_rate(48_000.0);

        let mut left = [1.0_f32; 64];
        let mut right = [1.0_f32; 64];
        let mut buffer = [&mut left[..], &mut right[..]];

        osc.process(&mut buffer, &Transport::default(), &test_params(0.0));

        assert!(left.iter().all(|s| s.abs() <= f32::EPSILON));
        assert!(right.iter().all(|s| s.abs() <= f32::EPSILON));
    }

    #[test]
    fn oscillator_outputs_signal_when_enabled() {
        let mut osc = Oscillator::default();
        osc.set_sample_rate(48_000.0);

        let mut left = [0.0_f32; 128];
        let mut right = [0.0_f32; 128];
        let mut buffer = [&mut left[..], &mut right[..]];

        osc.process(&mut buffer, &Transport::default(), &test_params(1.0));

        let peak_left = left
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
        let peak_right = right
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        assert!(
            peak_left > 0.01,
            "expected audible oscillator output on left"
        );
        assert!(
            peak_right > 0.01,
            "expected audible oscillator output on right"
        );
    }
}
