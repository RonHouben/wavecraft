//! Processor combinators for chaining DSP operations.

mod chain;

pub use chain::Chain;

/// Combines processors into a serial chain.
///
/// Single processor optimization:
/// ```ignore
/// Chain![MyGain]  // Compiles to just `MyGain`, no wrapper overhead
/// ```
///
/// Multiple processors:
/// ```ignore
/// Chain![Gain, Filter, Limiter]  // Nests as Chain<Gain, Chain<Filter, Limiter>>
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
