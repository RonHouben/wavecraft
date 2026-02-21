//! Processor combinators for chaining DSP operations.

mod chain;

pub use chain::Chain;

/// Combines processors into a serial signal chain.
///
/// Preferred macro for building DSP chains in Wavecraft.
/// Use `SignalChain!` for consistency with the `wavecraft_plugin!` DSL.
///
/// # Single Processor (Zero Overhead)
///
/// ```rust,no_run
/// use wavecraft_dsp::{Processor, Transport};
/// use wavecraft_dsp::SignalChain;
///
/// #[derive(Default)]
/// struct Noop;
///
/// impl Processor for Noop {
///     type Params = ();
///
///     fn process(&mut self, _buffer: &mut [&mut [f32]], _transport: &Transport, _params: &Self::Params) {}
/// }
///
/// type Single = SignalChain![Noop]; // Compiles to just `Noop`, no wrapper
/// ```
///
/// # Multiple Processors
///
/// ```rust,no_run
/// use wavecraft_dsp::{Processor, SignalChain, Transport};
///
/// #[derive(Default)]
/// struct A;
/// #[derive(Default)]
/// struct B;
///
/// impl Processor for A {
///     type Params = ();
///
///     fn process(&mut self, _buffer: &mut [&mut [f32]], _transport: &Transport, _params: &Self::Params) {}
/// }
///
/// impl Processor for B {
///     type Params = ();
///
///     fn process(&mut self, _buffer: &mut [&mut [f32]], _transport: &Transport, _params: &Self::Params) {}
/// }
///
/// type Multiple = SignalChain![A, B];
/// ```
#[macro_export]
macro_rules! SignalChain {
    // Single processor: no wrapping, zero overhead
    ($single:ty) => {
        $single
    };
    // Multiple: nest into Chain<A, Chain<B, ...>>
    ($first:ty, $($rest:ty),+ $(,)?) => {
        $crate::combinators::Chain<$first, $crate::SignalChain![$($rest),+]>
    };
}

/// Deprecated compatibility alias for `SignalChain!`.
///
/// Prefer `SignalChain!` for all new code and migrations.
/// This alias remains available for backward compatibility and is
/// scheduled for removal in `0.10.0`.
///
/// # Migration
///
/// ```rust,no_run
/// use wavecraft_dsp::SignalChain;
///
/// #[derive(Default)]
/// struct Noop;
///
/// // Old (deprecated):
/// // type MyChain = Chain![Noop];
///
/// // New (recommended):
/// type MyChain = SignalChain![Noop];
/// ```
#[deprecated(since = "0.9.0", note = "use `SignalChain!` instead")]
#[macro_export]
macro_rules! Chain {
    ($($tt:tt)*) => {
        $crate::SignalChain![$($tt)*]
    };
}
