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
/// pub struct MyGain(wavecraft_dsp::builtins::GainDsp);
///
/// impl Default for MyGain {
///     fn default() -> Self {
///         Self(wavecraft_dsp::builtins::GainDsp::default())
///     }
/// }
///
/// impl wavecraft_dsp::Processor for MyGain {
///     type Params = <wavecraft_dsp::builtins::GainDsp as wavecraft_dsp::Processor>::Params;
///
///     fn process(&mut self, buffer: &mut [&mut [f32]], transport: &wavecraft_dsp::Transport, params: &Self::Params) {
///         self.0.process(buffer, transport, params)
///     }
/// }
/// ```
///
/// # Built-in Processor Types
///
/// - `Gain` → `wavecraft_dsp::builtins::GainDsp`
/// - `Passthrough` → `wavecraft_dsp::builtins::PassthroughDsp`
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
        #[derive(Default)]
        pub struct $name($crate::wavecraft_dsp::builtins::GainDsp);

        impl $crate::wavecraft_dsp::Processor for $name {
            type Params =
                <$crate::wavecraft_dsp::builtins::GainDsp as $crate::wavecraft_dsp::Processor>::Params;

            fn process(
                &mut self,
                buffer: &mut [&mut [f32]],
                transport: &$crate::wavecraft_dsp::Transport,
                params: &Self::Params,
            ) {
                self.0.process(buffer, transport, params)
            }
        }
    };

    ($name:ident => Passthrough) => {
        #[derive(Default)]
        pub struct $name($crate::wavecraft_dsp::builtins::PassthroughDsp);

        impl $crate::wavecraft_dsp::Processor for $name {
            type Params =
                <$crate::wavecraft_dsp::builtins::PassthroughDsp as $crate::wavecraft_dsp::Processor>::Params;

            fn process(
                &mut self,
                buffer: &mut [&mut [f32]],
                transport: &$crate::wavecraft_dsp::Transport,
                params: &Self::Params,
            ) {
                self.0.process(buffer, transport, params)
            }
        }
    };
}
