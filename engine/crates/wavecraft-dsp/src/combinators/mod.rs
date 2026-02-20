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
/// use wavecraft_dsp::builtins::GainDsp;
/// use wavecraft_dsp::SignalChain;
///
/// type Single = SignalChain![GainDsp]; // Compiles to just `GainDsp`, no wrapper
/// ```
///
/// # Multiple Processors
///
/// ```rust,no_run
/// use wavecraft_dsp::builtins::{GainDsp, PassthroughDsp};
/// use wavecraft_dsp::SignalChain;
///
/// type Multiple = SignalChain![GainDsp, PassthroughDsp];
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
/// use wavecraft_dsp::builtins::GainDsp;
/// use wavecraft_dsp::SignalChain;
///
/// // Old (deprecated):
/// // type MyChain = Chain![GainDsp];
///
/// // New (recommended):
/// type MyChain = SignalChain![GainDsp];
/// ```
#[deprecated(since = "0.9.0", note = "use `SignalChain!` instead")]
#[macro_export]
macro_rules! Chain {
    ($($tt:tt)*) => {
        $crate::SignalChain![$($tt)*]
    };
}
