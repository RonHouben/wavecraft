//! Reusable processor implementations for Wavecraft plugins.

mod oscillator;
mod oscilloscope;

// Oscillator processor and parameter surface.
pub use oscillator::{Oscillator, OscillatorParams, Waveform, generate_waveform_sample};

// Oscilloscope tap, channel, and frame data surface.
pub use oscilloscope::{
    OSCILLOSCOPE_FRAME_POINTS, OscilloscopeFrameConsumer, OscilloscopeFrameProducer,
    OscilloscopeFrameSnapshot, OscilloscopeTap, create_oscilloscope_channel,
};
