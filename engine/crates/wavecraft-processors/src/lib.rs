//! Reusable processor implementations for Wavecraft plugins.

mod gain;
mod oscillator;
mod oscilloscope;
mod passthrough;
mod saturator;
mod unified_filter;

// Built-in processors and parameter surface.
pub use gain::{GainDsp, GainParams};
pub use passthrough::{PassthroughDsp, PassthroughParams};
pub use saturator::{SaturatorDsp, SaturatorParams};
pub use unified_filter::{UnifiedFilterDsp, UnifiedFilterMode, UnifiedFilterParams};

// Oscillator processor and parameter surface.
pub use oscillator::{Oscillator, OscillatorParams, Waveform, generate_waveform_sample};

// Oscilloscope tap, channel, and frame data surface.
pub use oscilloscope::{
    OSCILLOSCOPE_FRAME_POINTS, OscilloscopeFrameConsumer, OscilloscopeFrameProducer,
    OscilloscopeFrameSnapshot, OscilloscopeTap, create_oscilloscope_channel,
};
