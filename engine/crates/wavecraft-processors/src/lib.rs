//! Reusable processors for Wavecraft plugins.

mod oscillator;
mod oscilloscope;

pub use oscillator::{Oscillator, OscillatorParams};
pub use oscilloscope::{
    OSCILLOSCOPE_FRAME_POINTS, OscilloscopeFrameConsumer, OscilloscopeFrameProducer,
    OscilloscopeFrameSnapshot, OscilloscopeTap, create_oscilloscope_channel,
};
