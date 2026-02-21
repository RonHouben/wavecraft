//! Declarative macros for Wavecraft DSP processors.
//!
//! This module provides the `wavecraft_processor!` macro for creating
//! named wrappers around built-in DSP processors.
//!
//! For the `wavecraft_plugin!` macro that generates nih-plug plugins,
//! see the `wavecraft-nih_plug` crate.

/// `wavecraft_processor!` — creates a named wrapper around a built-in DSP processor.
///
/// This macro generates a newtype struct that wraps a built-in processor type and
/// delegates the `Processor` trait implementation to the inner type.
///
/// # Syntax
///
/// ```text
/// wavecraft_processor!(MyGain => Gain);
/// ```
///
/// # Generated Code
///
/// ```text
/// pub struct MyGain(wavecraft_processors::GainDsp);
///
/// impl Default for MyGain {
///     fn default() -> Self {
///         Self(wavecraft_processors::GainDsp::default())
///     }
/// }
///
/// impl wavecraft_dsp::Processor for MyGain {
///     type Params = <wavecraft_processors::GainDsp as wavecraft_dsp::Processor>::Params;
///
///     fn process(&mut self, buffer: &mut [&mut [f32]], transport: &wavecraft_dsp::Transport, params: &Self::Params) {
///         self.0.process(buffer, transport, params)
///     }
/// }
/// ```
///
/// # Built-in Processor Types
///
/// - `Gain` → `wavecraft_processors::GainDsp`
/// - `Passthrough` → `wavecraft_processors::PassthroughDsp`
/// - `Filter` → `wavecraft_processors::UnifiedFilterDsp`
/// - `Saturator` → `wavecraft_processors::SaturatorDsp`
///
/// # Example
///
/// ```rust,no_run
/// use wavecraft_core::wavecraft_processor;
/// use wavecraft_dsp::{Processor, Transport};
///
/// wavecraft_processor!(InputGain => Gain);
/// wavecraft_processor!(OutputGain => Gain);
///
/// let mut input = InputGain::default();
/// let mut output = OutputGain::default();
/// ```
#[macro_export]
macro_rules! wavecraft_processor {
    ($name:ident => Gain) => {
        $crate::wavecraft_processor!($name => $crate::wavecraft_processors::GainDsp);
    };

    ($name:ident => Passthrough) => {
        $crate::wavecraft_processor!($name => $crate::wavecraft_processors::PassthroughDsp);
    };

    ($name:ident => Filter) => {
        $crate::wavecraft_processor!($name => $crate::wavecraft_processors::UnifiedFilterDsp);
    };

    ($name:ident => Saturator) => {
        $crate::wavecraft_processor!($name => $crate::wavecraft_processors::SaturatorDsp);
    };

    ($name:ident => $inner:path) => {
        #[derive(Default)]
        pub struct $name($inner);

        impl $crate::wavecraft_dsp::Processor for $name {
            type Params = <$inner as $crate::wavecraft_dsp::Processor>::Params;

            fn process(
                &mut self,
                buffer: &mut [&mut [f32]],
                transport: &$crate::wavecraft_dsp::Transport,
                params: &Self::Params,
            ) {
                self.0.process(buffer, transport, params)
            }

            fn set_sample_rate(&mut self, sample_rate: f32) {
                self.0.set_sample_rate(sample_rate)
            }

            fn reset(&mut self) {
                self.0.reset()
            }
        }
    };
}
