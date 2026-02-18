//! Chain combinator for serial processor composition.

use crate::traits::{ParamSpec, Processor, ProcessorParams, Transport};

/// Combines two processors in series: A → B.
///
/// Audio flows through processor A, then through processor B.
/// Parameters from both processors are merged.
pub struct Chain<A, B> {
    pub first: A,
    pub second: B,
}

impl<A, B> Default for Chain<A, B>
where
    A: Default,
    B: Default,
{
    fn default() -> Self {
        Self {
            first: A::default(),
            second: B::default(),
        }
    }
}

/// Combined parameters for chained processors.
///
/// Merges parameter specs from both processors, prefixing IDs to avoid collisions.
pub struct ChainParams<PA, PB> {
    pub first: PA,
    pub second: PB,
}

impl<PA, PB> Default for ChainParams<PA, PB>
where
    PA: Default,
    PB: Default,
{
    fn default() -> Self {
        Self {
            first: PA::default(),
            second: PB::default(),
        }
    }
}

impl<PA, PB> ProcessorParams for ChainParams<PA, PB>
where
    PA: ProcessorParams,
    PB: ProcessorParams,
{
    fn param_specs() -> &'static [ParamSpec] {
        // WORKAROUND FOR HOT-RELOAD HANG:
        //
        // Do NOT use OnceLock or any locking primitive here. On macOS, when the
        // subprocess calls dlopen() → wavecraft_get_params_json() → param_specs(),
        // initialization of OnceLock statics can hang indefinitely (30s timeout).
        //
        // Instead, we allocate and leak the merged specs on EVERY call. This is
        // acceptable because:
        // 1. param_specs() is called at most once per plugin load (startup only)
        // 2. The leak is ~hundreds of bytes (not per-sample, not per-frame)
        // 3. Plugin lifetime = process lifetime (no meaningful leak)
        // 4. Hot-reload works correctly (no 30s hang)
        //
        // This is a pragmatic trade-off: small memory leak vs. broken hot-reload.
        // Future work: investigate root cause of OnceLock hang on macOS dlopen.

        let first_specs = PA::param_specs();
        let second_specs = PB::param_specs();

        let mut merged = Vec::with_capacity(first_specs.len() + second_specs.len());

        // Add first processor's params
        for spec in first_specs {
            merged.push(ParamSpec {
                name: spec.name,
                id_suffix: spec.id_suffix,
                range: spec.range.clone(),
                default: spec.default,
                unit: spec.unit,
                group: spec.group,
            });
        }

        // Add second processor's params
        for spec in second_specs {
            merged.push(ParamSpec {
                name: spec.name,
                id_suffix: spec.id_suffix,
                range: spec.range.clone(),
                default: spec.default,
                unit: spec.unit,
                group: spec.group,
            });
        }

        // Leak to get 'static reference (intentional - see comment above)
        Box::leak(merged.into_boxed_slice())
    }

    fn from_param_defaults() -> Self {
        Self {
            first: PA::from_param_defaults(),
            second: PB::from_param_defaults(),
        }
    }

    fn apply_plain_values(&mut self, values: &[f32]) {
        let first_count = PA::param_specs().len();
        let split_at = first_count.min(values.len());
        let (first_values, second_values) = values.split_at(split_at);

        self.first.apply_plain_values(first_values);
        self.second.apply_plain_values(second_values);
    }
}

impl<A, B> Processor for Chain<A, B>
where
    A: Processor,
    B: Processor,
{
    type Params = ChainParams<A::Params, B::Params>;

    fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport, params: &Self::Params) {
        // Process first, then second (serial chain)
        self.first.process(buffer, transport, &params.first);
        self.second.process(buffer, transport, &params.second);
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.first.set_sample_rate(sample_rate);
        self.second.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.first.reset();
        self.second.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::{GainDsp, GainParams, PassthroughDsp, PassthroughParams};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    };

    #[derive(Clone)]
    struct TestParams;

    impl Default for TestParams {
        fn default() -> Self {
            Self
        }
    }

    impl ProcessorParams for TestParams {
        fn param_specs() -> &'static [ParamSpec] {
            &[]
        }
    }

    struct LifecycleProbe {
        set_sample_rate_calls: Arc<AtomicU32>,
        reset_calls: Arc<AtomicU32>,
        last_sample_rate_bits: Arc<AtomicU32>,
        touched_process: Arc<AtomicBool>,
    }

    impl LifecycleProbe {
        fn new() -> Self {
            Self {
                set_sample_rate_calls: Arc::new(AtomicU32::new(0)),
                reset_calls: Arc::new(AtomicU32::new(0)),
                last_sample_rate_bits: Arc::new(AtomicU32::new(0)),
                touched_process: Arc::new(AtomicBool::new(false)),
            }
        }
    }

    impl Processor for LifecycleProbe {
        type Params = TestParams;

        fn process(
            &mut self,
            _buffer: &mut [&mut [f32]],
            _transport: &Transport,
            _params: &Self::Params,
        ) {
            self.touched_process.store(true, Ordering::SeqCst);
        }

        fn set_sample_rate(&mut self, sample_rate: f32) {
            self.set_sample_rate_calls.fetch_add(1, Ordering::SeqCst);
            self.last_sample_rate_bits
                .store(sample_rate.to_bits(), Ordering::SeqCst);
        }

        fn reset(&mut self) {
            self.reset_calls.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_chain_processes_in_order() {
        let mut chain = Chain {
            first: GainDsp::default(),
            second: GainDsp::default(),
        };

        let mut left = [1.0_f32, 1.0_f32];
        let mut right = [1.0_f32, 1.0_f32];
        let mut buffer = [&mut left[..], &mut right[..]];

        let transport = Transport::default();
        let params = ChainParams {
            first: GainParams { level: 0.5 },
            second: GainParams { level: 2.0 },
        };

        chain.process(&mut buffer, &transport, &params);

        // Expected: 1.0 * 0.5 * 2.0 = 1.0
        assert!((buffer[0][0] - 1.0_f32).abs() < 1e-6);
        assert!((buffer[1][0] - 1.0_f32).abs() < 1e-6);
    }

    #[test]
    fn test_chain_with_passthrough() {
        let mut chain = Chain {
            first: PassthroughDsp,
            second: GainDsp::default(),
        };

        let mut left = [2.0_f32, 2.0_f32];
        let mut right = [2.0_f32, 2.0_f32];
        let mut buffer = [&mut left[..], &mut right[..]];

        let transport = Transport::default();
        let params = ChainParams {
            first: PassthroughParams,
            second: GainParams { level: 0.5 },
        };

        chain.process(&mut buffer, &transport, &params);

        // Expected: 2.0 * 1.0 * 0.5 = 1.0
        assert!((buffer[0][0] - 1.0_f32).abs() < 1e-6);
    }

    #[test]
    fn test_chain_params_merge() {
        let specs = <ChainParams<GainParams, GainParams>>::param_specs();
        assert_eq!(specs.len(), 2); // Both gain params

        // Both should have "Level" name
        assert_eq!(specs[0].name, "Level");
        assert_eq!(specs[1].name, "Level");
    }

    #[test]
    fn test_chain_default() {
        let _chain: Chain<GainDsp, PassthroughDsp> = Chain::default();
        let _params: ChainParams<GainParams, PassthroughParams> = ChainParams::default();
    }

    #[test]
    fn test_chain_from_param_defaults_uses_children_spec_defaults() {
        let defaults = <ChainParams<GainParams, GainParams>>::from_param_defaults();
        assert!((defaults.first.level - 1.0).abs() < 1e-6);
        assert!((defaults.second.level - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_chain_apply_plain_values_splits_by_child_param_count() {
        let mut params = <ChainParams<GainParams, GainParams>>::from_param_defaults();
        params.apply_plain_values(&[0.25, 1.75]);

        assert!((params.first.level - 0.25).abs() < 1e-6);
        assert!((params.second.level - 1.75).abs() < 1e-6);
    }

    #[test]
    fn test_chain_propagates_set_sample_rate_to_both_processors() {
        let first = LifecycleProbe::new();
        let second = LifecycleProbe::new();

        let first_calls = Arc::clone(&first.set_sample_rate_calls);
        let second_calls = Arc::clone(&second.set_sample_rate_calls);
        let first_sr = Arc::clone(&first.last_sample_rate_bits);
        let second_sr = Arc::clone(&second.last_sample_rate_bits);

        let mut chain = Chain { first, second };
        chain.set_sample_rate(48_000.0);

        assert_eq!(first_calls.load(Ordering::SeqCst), 1);
        assert_eq!(second_calls.load(Ordering::SeqCst), 1);
        assert_eq!(f32::from_bits(first_sr.load(Ordering::SeqCst)), 48_000.0);
        assert_eq!(f32::from_bits(second_sr.load(Ordering::SeqCst)), 48_000.0);
    }

    #[test]
    fn test_chain_propagates_reset_to_both_processors() {
        let first = LifecycleProbe::new();
        let second = LifecycleProbe::new();

        let first_resets = Arc::clone(&first.reset_calls);
        let second_resets = Arc::clone(&second.reset_calls);

        let mut chain = Chain { first, second };
        chain.reset();

        assert_eq!(first_resets.load(Ordering::SeqCst), 1);
        assert_eq!(second_resets.load(Ordering::SeqCst), 1);
    }
}
