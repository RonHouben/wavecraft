//! Reusable processor implementations for Wavecraft plugins.

mod gain;
mod oscilloscope;
mod passthrough;
mod saturator;
mod test_tone_processor;
mod unified_filter;

// Built-in processors and parameter surface.
pub use gain::{GainDsp, GainParams};
pub use passthrough::{PassthroughDsp, PassthroughParams};
pub use saturator::{SaturatorDsp, SaturatorParams};
pub use unified_filter::{UnifiedFilterDsp, UnifiedFilterMode, UnifiedFilterParams};

// Test tone processor and parameter surface.
pub use test_tone_processor::{TestToneProcessor, TestToneProcessorParams};

// Oscilloscope tap, channel, and frame data surface.
pub use oscilloscope::{
    OSCILLOSCOPE_FRAME_POINTS, OscilloscopeFrameConsumer, OscilloscopeFrameProducer,
    OscilloscopeFrameSnapshot, OscilloscopeTap, create_oscilloscope_channel,
};
