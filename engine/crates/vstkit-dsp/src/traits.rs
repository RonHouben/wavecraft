//! Core DSP traits for user-implemented audio processors.
//!
//! This module defines the primary extension points for users building plugins
//! with VstKit. The `Processor` trait is the main interface for custom DSP code.

/// Transport information for timing-aware DSP.
///
/// Provides context about playback state, tempo, and position.
#[derive(Debug, Clone, Copy)]
pub struct Transport {
    /// Current tempo in BPM (beats per minute).
    pub tempo: Option<f64>,
    
    /// Current playback position in samples.
    pub pos_samples: i64,
    
    /// True if the host is playing.
    pub playing: bool,
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            tempo: None,
            pos_samples: 0,
            playing: false,
        }
    }
}

/// Trait for user-implemented DSP processors.
///
/// Implement this trait to define custom audio processing logic.
/// All methods must be real-time safe (no allocations, locks, or syscalls).
///
/// # Example
///
/// ```rust
/// use vstkit_dsp::{Processor, Transport};
///
/// struct MyGain {
///     gain: f32,
/// }
///
/// impl Processor for MyGain {
///     fn process(&mut self, buffer: &mut [&mut [f32]], _transport: &Transport) {
///         for channel in buffer.iter_mut() {
///             for sample in channel.iter_mut() {
///                 *sample *= self.gain;
///             }
///         }
///     }
/// }
/// ```
pub trait Processor: Send + 'static {
    /// Process a buffer of audio samples.
    ///
    /// The buffer is provided as a slice of mutable slices, one per channel.
    /// Modify samples in-place to apply your DSP effect.
    ///
    /// # Arguments
    /// * `buffer` - Audio channels as `[L, R, ...]` where each channel is `[samples]`
    /// * `transport` - Playback timing information
    ///
    /// # Real-Time Safety
    /// This method is called on the audio thread. It MUST be real-time safe:
    /// - No allocations (`Vec::push`, `String`, `Box::new`)
    /// - No locks (`Mutex`, `RwLock`)
    /// - No syscalls (file I/O, logging, network)
    /// - No panics (use `debug_assert!` only)
    fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport);

    /// Called when the sample rate changes.
    ///
    /// Use this to update internal state that depends on sample rate
    /// (e.g., filter coefficients, delay line sizes).
    ///
    /// # Arguments
    /// * `sample_rate` - New sample rate in Hz (e.g., 44100.0, 48000.0)
    ///
    /// # Default
    /// No-op by default. Override if your processor needs sample rate.
    fn set_sample_rate(&mut self, _sample_rate: f32) {}

    /// Reset processor state.
    ///
    /// Called when the host stops playback or when the user resets the plugin.
    /// Use this to clear delay lines, reset filters, etc.
    ///
    /// # Default
    /// No-op by default. Override if your processor maintains state.
    fn reset(&mut self) {}
}
