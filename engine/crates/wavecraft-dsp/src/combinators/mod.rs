//! Processor combinators for chaining DSP operations.

mod chain;

pub use chain::Chain;

/// Combines processors into a serial chain.
///
/// Single processor optimization:
/// ```rust,no_run
/// use wavecraft_dsp::builtins::GainDsp;
/// use wavecraft_dsp::Chain;
///
/// type Single = Chain![GainDsp]; // Compiles to just `GainDsp`, no wrapper overhead
/// ```
///
/// Multiple processors:
/// ```rust,no_run
/// use wavecraft_dsp::builtins::{GainDsp, PassthroughDsp};
/// use wavecraft_dsp::Chain;
///
/// type Multiple = Chain![GainDsp, PassthroughDsp];
/// ```
#[macro_export]
macro_rules! Chain {
    // Single processor: no wrapping, zero overhead
    ($single:ty) => {
        $single
    };
    // Multiple: nest into Chain<A, Chain<B, ...>>
    ($first:ty, $($rest:ty),+ $(,)?) => {
        $crate::combinators::Chain<$first, $crate::Chain![$($rest),+]>
    };
}
