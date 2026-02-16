//! Core DSP traits for user-implemented audio processors.
//!
//! This module defines the primary extension points for users building plugins
//! with Wavecraft. The `Processor` trait is the main interface for custom DSP code.

/// Transport information for timing-aware DSP.
///
/// Provides context about playback state, tempo, and position.
#[derive(Debug, Clone, Copy, Default)]
pub struct Transport {
    /// Current tempo in BPM (beats per minute).
    pub tempo: Option<f64>,

    /// Current playback position in samples.
    pub pos_samples: i64,

    /// True if the host is playing.
    pub playing: bool,
}

/// Trait for defining processor parameters.
///
/// This trait provides metadata about a processor's parameters,
/// enabling automatic UI generation and nih-plug integration.
///
/// Typically implemented via `#[derive(ProcessorParams)]` rather than manually.
pub trait ProcessorParams: Default + Send + Sync + 'static {
    /// Returns the parameter specifications for this processor.
    fn param_specs() -> &'static [ParamSpec];

    /// Builds parameter values initialized from each [`ParamSpec::default`].
    ///
    /// By default this falls back to `Self::default()`. Implementations should
    /// override this when struct field defaults differ from declared
    /// `param_specs()` defaults (common with `#[derive(Default)]` on numeric
    /// fields, which initializes them to zero).
    fn from_param_defaults() -> Self {
        Self::default()
    }
}

/// Specification for a single parameter.
#[derive(Debug, Clone)]
pub struct ParamSpec {
    /// Display name of the parameter (e.g., "Frequency").
    pub name: &'static str,

    /// ID suffix for this parameter (e.g., "frequency").
    /// Full ID will be prefixed with processor name: "my_filter_frequency"
    pub id_suffix: &'static str,

    /// Value range for this parameter.
    pub range: ParamRange,

    /// Default value.
    pub default: f64,

    /// Unit string (e.g., "dB", "Hz", "%").
    pub unit: &'static str,

    /// Optional group name for UI organization (e.g., "Input", "Processing", "Output").
    pub group: Option<&'static str>,
}

/// Parameter value range definition.
#[derive(Debug, Clone)]
pub enum ParamRange {
    /// Linear range from min to max.
    Linear { min: f64, max: f64 },

    /// Skewed range with exponential/logarithmic scaling.
    /// Factor > 1.0 = logarithmic, factor < 1.0 = exponential.
    Skewed { min: f64, max: f64, factor: f64 },

    /// Integer stepped range (for enums, switches).
    Stepped { min: i32, max: i32 },

    /// Enumerated parameter with named variants.
    ///
    /// Index 0 corresponds to the first variant, 1 to the second, etc.
    Enum {
        variants: &'static [&'static str],
    },
}

/// Unit type has no parameters.
impl ProcessorParams for () {
    fn param_specs() -> &'static [ParamSpec] {
        &[]
    }
}

/// Trait for user-implemented DSP processors.
///
/// Implement this trait to define custom audio processing logic.
/// All methods must be real-time safe (no allocations, locks, or syscalls).
///
/// # Example
///
/// ```rust,no_run
/// use wavecraft_dsp::{ParamRange, ParamSpec, Processor, ProcessorParams, Transport};
///
/// #[derive(Default)]
/// struct MyGainParams {
///     level: f32,
/// }
///
/// impl ProcessorParams for MyGainParams {
///     fn param_specs() -> &'static [ParamSpec] {
///         &[ParamSpec {
///             name: "Level",
///             id_suffix: "level",
///             range: ParamRange::Linear { min: 0.0, max: 2.0 },
///             default: 1.0,
///             unit: "x",
///             group: None,
///         }]
///     }
/// }
///
/// struct MyGain {
///     sample_rate: f32,
/// }
///
/// impl Processor for MyGain {
///     type Params = MyGainParams;
///
///     fn process(
///         &mut self,
///         buffer: &mut [&mut [f32]],
///         _transport: &Transport,
///         params: &Self::Params,
///     ) {
///         for channel in buffer.iter_mut() {
///             for sample in channel.iter_mut() {
///                 *sample *= params.level;
///             }
///         }
///     }
/// }
/// ```
pub trait Processor: Send + 'static {
    /// Associated parameter type for this processor.
    ///
    /// Use `()` for processors with no parameters, or define a struct
    /// with `#[derive(ProcessorParams)]`.
    type Params: ProcessorParams + Default + Send + Sync + 'static;

    /// Process a buffer of audio samples.
    ///
    /// The buffer is provided as a slice of mutable slices, one per channel.
    /// Modify samples in-place to apply your DSP effect.
    ///
    /// # Arguments
    /// * `buffer` - Audio channels as `[L, R, ...]` where each channel is `[samples]`
    /// * `transport` - Playback timing information
    /// * `params` - Current parameter values
    ///
    /// # Real-Time Safety
    /// This method is called on the audio thread. It MUST be real-time safe:
    /// - No allocations (`Vec::push`, `String`, `Box::new`)
    /// - No locks (`Mutex`, `RwLock`)
    /// - No syscalls (file I/O, logging, network)
    /// - No panics (use `debug_assert!` only)
    fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport, params: &Self::Params);

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
