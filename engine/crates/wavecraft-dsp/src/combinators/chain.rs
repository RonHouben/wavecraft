//! Chain combinator for serial processor composition.

use crate::traits::{ParamSpec, Processor, ProcessorParams, Transport};

/// Processor wrapper that adds a standard per-instance bypass parameter.
pub struct Bypassed<P> {
    pub processor: P,
    source_bypassed: bool,
    target_bypassed: bool,
    transition_phase: BypassTransitionPhase,
    transition_samples: u32,
}

#[derive(Debug, Clone, Copy)]
enum BypassTransitionPhase {
    Stable,
    FadeOut { remaining: u32 },
    FadeIn { remaining: u32 },
}

const DEFAULT_BYPASS_TRANSITION_SAMPLES: u32 = 64;
const MIN_BYPASS_TRANSITION_SAMPLES: u32 = 16;
const MAX_BYPASS_TRANSITION_SAMPLES: u32 = 256;
const BYPASS_TRANSITION_SECONDS: f32 = 0.002;

impl<P> Bypassed<P> {
    /// Creates a bypass wrapper around a processor instance.
    pub fn new(processor: P) -> Self {
        Self {
            processor,
            source_bypassed: false,
            target_bypassed: false,
            transition_phase: BypassTransitionPhase::Stable,
            transition_samples: DEFAULT_BYPASS_TRANSITION_SAMPLES,
        }
    }

    #[inline]
    fn transition_samples_for_rate(sample_rate: f32) -> u32 {
        if sample_rate <= 0.0 {
            return DEFAULT_BYPASS_TRANSITION_SAMPLES;
        }

        ((sample_rate * BYPASS_TRANSITION_SECONDS).round() as u32)
            .clamp(MIN_BYPASS_TRANSITION_SAMPLES, MAX_BYPASS_TRANSITION_SAMPLES)
    }

    #[inline]
    fn apply_gain_ramp(buffer: &mut [&mut [f32]], start_gain: f32, end_gain: f32) {
        let samples = buffer
            .iter()
            .map(|channel| channel.len())
            .min()
            .unwrap_or(0);
        if samples == 0 {
            return;
        }

        let denominator = samples.saturating_sub(1) as f32;

        for channel in buffer.iter_mut() {
            for (idx, sample) in channel.iter_mut().take(samples).enumerate() {
                let t = if denominator <= 0.0 {
                    1.0
                } else {
                    idx as f32 / denominator
                };
                let gain = start_gain + (end_gain - start_gain) * t;
                *sample *= gain;
            }
        }
    }
}

impl<P> Default for Bypassed<P>
where
    P: Default,
{
    fn default() -> Self {
        Self::new(P::default())
    }
}

impl<P> From<P> for Bypassed<P> {
    fn from(processor: P) -> Self {
        Self::new(processor)
    }
}

/// Parameters for [`Bypassed`].
///
/// Includes wrapped processor parameters plus one boolean bypass flag.
pub struct BypassedParams<PP> {
    pub inner: PP,
    pub bypassed: bool,
}

impl<PP> Default for BypassedParams<PP>
where
    PP: Default,
{
    fn default() -> Self {
        Self {
            inner: PP::default(),
            bypassed: false,
        }
    }
}

impl<PP> ProcessorParams for BypassedParams<PP>
where
    PP: ProcessorParams,
{
    fn param_specs() -> &'static [ParamSpec] {
        fn extend_specs(target: &mut Vec<ParamSpec>, source: &[ParamSpec]) {
            for spec in source {
                target.push(ParamSpec {
                    name: spec.name,
                    id_suffix: spec.id_suffix,
                    range: spec.range.clone(),
                    default: spec.default,
                    unit: spec.unit,
                    group: spec.group,
                });
            }
        }

        let inner_specs = PP::param_specs();
        let mut merged = Vec::with_capacity(inner_specs.len() + 1);

        extend_specs(&mut merged, inner_specs);
        merged.push(ParamSpec {
            name: "Bypass",
            id_suffix: "bypass",
            range: crate::ParamRange::Stepped { min: 0, max: 1 },
            default: 0.0,
            unit: "",
            group: None,
        });

        // See comment in ChainParams::param_specs for rationale.
        Box::leak(merged.into_boxed_slice())
    }

    fn from_param_defaults() -> Self {
        Self {
            inner: PP::from_param_defaults(),
            bypassed: false,
        }
    }

    fn plain_value_count() -> usize {
        PP::plain_value_count() + 1
    }

    fn apply_plain_values(&mut self, values: &[f32]) {
        let inner_count = PP::plain_value_count();
        let split_at = inner_count.min(values.len());
        let (inner_values, bypass_values) = values.split_at(split_at);

        self.inner.apply_plain_values(inner_values);

        if let Some(bypass_value) = bypass_values.first() {
            self.bypassed = *bypass_value >= 0.5;
        }
    }
}

impl<P> Processor for Bypassed<P>
where
    P: Processor,
{
    type Params = BypassedParams<P::Params>;

    fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport, params: &Self::Params) {
        if params.bypassed != self.target_bypassed {
            self.target_bypassed = params.bypassed;
            if self.source_bypassed != self.target_bypassed {
                self.transition_phase = BypassTransitionPhase::FadeOut {
                    remaining: self.transition_samples,
                };
            }
        }

        if !self.source_bypassed {
            self.processor.process(buffer, transport, &params.inner);
        }

        let samples = buffer
            .iter()
            .map(|channel| channel.len())
            .min()
            .unwrap_or(0) as u32;
        if samples == 0 {
            return;
        }

        match self.transition_phase {
            BypassTransitionPhase::Stable => {}
            BypassTransitionPhase::FadeOut { remaining } => {
                let used = remaining.min(samples);
                let total = self.transition_samples.max(1) as f32;
                let start_gain = remaining as f32 / total;
                let end_gain = remaining.saturating_sub(used) as f32 / total;

                Self::apply_gain_ramp(buffer, start_gain, end_gain);

                let new_remaining = remaining.saturating_sub(used);
                if new_remaining == 0 {
                    self.source_bypassed = self.target_bypassed;
                    self.transition_phase = BypassTransitionPhase::FadeIn {
                        remaining: self.transition_samples,
                    };
                } else {
                    self.transition_phase = BypassTransitionPhase::FadeOut {
                        remaining: new_remaining,
                    };
                }
            }
            BypassTransitionPhase::FadeIn { remaining } => {
                let used = remaining.min(samples);
                let total = self.transition_samples.max(1) as f32;
                let start_gain = 1.0 - (remaining as f32 / total);
                let end_gain = 1.0 - (remaining.saturating_sub(used) as f32 / total);

                Self::apply_gain_ramp(buffer, start_gain, end_gain);

                let new_remaining = remaining.saturating_sub(used);
                if new_remaining == 0 {
                    self.transition_phase = BypassTransitionPhase::Stable;
                } else {
                    self.transition_phase = BypassTransitionPhase::FadeIn {
                        remaining: new_remaining,
                    };
                }
            }
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.transition_samples = Self::transition_samples_for_rate(sample_rate);
        self.processor.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.source_bypassed = self.target_bypassed;
        self.transition_phase = BypassTransitionPhase::Stable;
        self.processor.reset();
    }
}

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
/// Merges parameter specs from both processors.
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
        fn extend_specs(target: &mut Vec<ParamSpec>, source: &[ParamSpec]) {
            for spec in source {
                target.push(ParamSpec {
                    name: spec.name,
                    id_suffix: spec.id_suffix,
                    range: spec.range.clone(),
                    default: spec.default,
                    unit: spec.unit,
                    group: spec.group,
                });
            }
        }

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

        extend_specs(&mut merged, first_specs);
        extend_specs(&mut merged, second_specs);

        // Leak to get 'static reference (intentional - see comment above)
        Box::leak(merged.into_boxed_slice())
    }

    fn from_param_defaults() -> Self {
        Self {
            first: PA::from_param_defaults(),
            second: PB::from_param_defaults(),
        }
    }

    fn plain_value_count() -> usize {
        PA::plain_value_count() + PB::plain_value_count()
    }

    fn apply_plain_values(&mut self, values: &[f32]) {
        let first_count = PA::plain_value_count();
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
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    };

    #[derive(Clone)]
    struct TestGainParams {
        level: f32,
    }

    impl Default for TestGainParams {
        fn default() -> Self {
            Self { level: 1.0 }
        }
    }

    impl ProcessorParams for TestGainParams {
        fn param_specs() -> &'static [ParamSpec] {
            static SPECS: [ParamSpec; 1] = [ParamSpec {
                name: "Level",
                id_suffix: "level",
                range: crate::ParamRange::Linear { min: 0.0, max: 2.0 },
                default: 1.0,
                unit: "x",
                group: None,
            }];
            &SPECS
        }

        fn from_param_defaults() -> Self {
            Self { level: 1.0 }
        }

        fn apply_plain_values(&mut self, values: &[f32]) {
            if let Some(level) = values.first() {
                self.level = *level;
            }
        }
    }

    #[derive(Default)]
    struct TestGainDsp;

    impl Processor for TestGainDsp {
        type Params = TestGainParams;

        fn process(
            &mut self,
            buffer: &mut [&mut [f32]],
            _transport: &Transport,
            params: &Self::Params,
        ) {
            for channel in buffer.iter_mut() {
                for sample in channel.iter_mut() {
                    *sample *= params.level;
                }
            }
        }
    }

    #[derive(Clone, Default)]
    struct TestPassthroughParams;

    impl ProcessorParams for TestPassthroughParams {
        fn param_specs() -> &'static [ParamSpec] {
            &[]
        }
    }

    #[derive(Default)]
    struct TestPassthroughDsp;

    impl Processor for TestPassthroughDsp {
        type Params = TestPassthroughParams;

        fn process(
            &mut self,
            _buffer: &mut [&mut [f32]],
            _transport: &Transport,
            _params: &Self::Params,
        ) {
        }
    }

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

    #[derive(Clone, Default)]
    struct PanicParamSpecsParams {
        value: f32,
    }

    impl ProcessorParams for PanicParamSpecsParams {
        fn param_specs() -> &'static [ParamSpec] {
            panic!("param_specs must not be called from runtime split path")
        }

        fn plain_value_count() -> usize {
            1
        }

        fn apply_plain_values(&mut self, values: &[f32]) {
            if let Some(value) = values.first() {
                self.value = *value;
            }
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
            first: TestGainDsp,
            second: TestGainDsp,
        };

        let mut left = [1.0_f32, 1.0_f32];
        let mut right = [1.0_f32, 1.0_f32];
        let mut buffer = [&mut left[..], &mut right[..]];

        let transport = Transport::default();
        let params = ChainParams {
            first: TestGainParams { level: 0.5 },
            second: TestGainParams { level: 2.0 },
        };

        chain.process(&mut buffer, &transport, &params);

        // Expected: 1.0 * 0.5 * 2.0 = 1.0
        assert!((buffer[0][0] - 1.0_f32).abs() < 1e-6);
        assert!((buffer[1][0] - 1.0_f32).abs() < 1e-6);
    }

    #[test]
    fn test_chain_with_passthrough() {
        let mut chain = Chain {
            first: TestPassthroughDsp,
            second: TestGainDsp,
        };

        let mut left = [2.0_f32, 2.0_f32];
        let mut right = [2.0_f32, 2.0_f32];
        let mut buffer = [&mut left[..], &mut right[..]];

        let transport = Transport::default();
        let params = ChainParams {
            first: TestPassthroughParams,
            second: TestGainParams { level: 0.5 },
        };

        chain.process(&mut buffer, &transport, &params);

        // Expected: 2.0 * 1.0 * 0.5 = 1.0
        assert!((buffer[0][0] - 1.0_f32).abs() < 1e-6);
    }

    #[test]
    fn test_chain_params_merge() {
        let specs = <ChainParams<TestGainParams, TestGainParams>>::param_specs();
        assert_eq!(specs.len(), 2); // Both gain params

        // Both should have "Level" name
        assert_eq!(specs[0].name, "Level");
        assert_eq!(specs[1].name, "Level");
    }

    #[test]
    fn test_chain_default() {
        let _chain: Chain<TestGainDsp, TestPassthroughDsp> = Chain::default();
        let _params: ChainParams<TestGainParams, TestPassthroughParams> = ChainParams::default();
    }

    #[test]
    fn test_chain_from_param_defaults_uses_children_spec_defaults() {
        let defaults = <ChainParams<TestGainParams, TestGainParams>>::from_param_defaults();
        assert!((defaults.first.level - 1.0).abs() < 1e-6);
        assert!((defaults.second.level - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_chain_apply_plain_values_splits_by_child_param_count() {
        let mut params = <ChainParams<TestGainParams, TestGainParams>>::from_param_defaults();
        params.apply_plain_values(&[0.25, 1.75]);

        assert!((params.first.level - 0.25).abs() < 1e-6);
        assert!((params.second.level - 1.75).abs() < 1e-6);
    }

    #[test]
    fn test_bypassed_apply_plain_values_uses_plain_value_count_without_param_specs() {
        let mut params = <BypassedParams<PanicParamSpecsParams>>::from_param_defaults();
        params.apply_plain_values(&[0.5, 1.0]);

        assert!((params.inner.value - 0.5).abs() < 1e-6);
        assert!(params.bypassed);
    }

    #[test]
    fn test_chain_apply_plain_values_uses_plain_value_count_without_param_specs() {
        let mut params =
            <ChainParams<PanicParamSpecsParams, PanicParamSpecsParams>>::from_param_defaults();
        params.apply_plain_values(&[0.25, 0.75]);

        assert!((params.first.value - 0.25).abs() < 1e-6);
        assert!((params.second.value - 0.75).abs() < 1e-6);
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

    #[test]
    fn test_bypassed_param_specs_include_bypass_flag() {
        let specs = <BypassedParams<TestGainParams>>::param_specs();
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].id_suffix, "level");
        assert_eq!(specs[1].id_suffix, "bypass");
    }

    #[test]
    fn test_bypassed_process_skips_child_when_bypassed_after_transition() {
        let touched = Arc::new(AtomicBool::new(false));

        struct TouchProbe {
            touched: Arc<AtomicBool>,
        }

        impl Processor for TouchProbe {
            type Params = TestParams;

            fn process(
                &mut self,
                _buffer: &mut [&mut [f32]],
                _transport: &Transport,
                _params: &Self::Params,
            ) {
                self.touched.store(true, Ordering::SeqCst);
            }
        }

        let mut wrapped = Bypassed::new(TouchProbe {
            touched: Arc::clone(&touched),
        });

        let mut left = [1.0_f32, 2.0_f32];
        let mut right = [3.0_f32, 4.0_f32];
        let mut buffer = [&mut left[..], &mut right[..]];

        // First call starts transition, subsequent calls settle bypass.
        for _ in 0..140 {
            wrapped.process(
                &mut buffer,
                &Transport::default(),
                &BypassedParams {
                    inner: TestParams,
                    bypassed: true,
                },
            );
        }

        touched.store(false, Ordering::SeqCst);

        wrapped.process(
            &mut buffer,
            &Transport::default(),
            &BypassedParams {
                inner: TestParams,
                bypassed: true,
            },
        );

        let mut verify_left = [1.0_f32, 2.0_f32];
        let mut verify_right = [3.0_f32, 4.0_f32];
        let original_left = verify_left;
        let original_right = verify_right;
        let mut verify_buffer = [&mut verify_left[..], &mut verify_right[..]];

        wrapped.process(
            &mut verify_buffer,
            &Transport::default(),
            &BypassedParams {
                inner: TestParams,
                bypassed: true,
            },
        );

        assert!(!touched.load(Ordering::SeqCst));
        assert_eq!(verify_left, original_left);
        assert_eq!(verify_right, original_right);
    }

    #[test]
    fn test_bypassed_process_runs_child_when_active() {
        let mut wrapped = Bypassed::new(TestGainDsp);

        let mut left = [1.0_f32, 1.0_f32];
        let mut right = [1.0_f32, 1.0_f32];
        let mut buffer = [&mut left[..], &mut right[..]];

        wrapped.process(
            &mut buffer,
            &Transport::default(),
            &BypassedParams {
                inner: TestGainParams { level: 0.5 },
                bypassed: false,
            },
        );

        assert!((left[0] - 0.5_f32).abs() < 1e-6);
        assert!((right[0] - 0.5_f32).abs() < 1e-6);
    }

    #[derive(Default)]
    struct PolarityFlip;

    impl Processor for PolarityFlip {
        type Params = TestPassthroughParams;

        fn process(
            &mut self,
            buffer: &mut [&mut [f32]],
            _transport: &Transport,
            _params: &Self::Params,
        ) {
            for channel in buffer.iter_mut() {
                for sample in channel.iter_mut() {
                    *sample = -*sample;
                }
            }
        }
    }

    #[test]
    fn test_bypassed_transition_smooths_toggle_edges() {
        let mut wrapped = Bypassed::new(PolarityFlip);

        let active = BypassedParams {
            inner: TestPassthroughParams,
            bypassed: false,
        };
        let bypassed = BypassedParams {
            inner: TestPassthroughParams,
            bypassed: true,
        };

        // Warm up active state: output should be wet (-1.0).
        let mut previous = -1.0_f32;
        for _ in 0..4 {
            let mut sample = [1.0_f32];
            let mut buffer = [&mut sample[..]];
            wrapped.process(&mut buffer, &Transport::default(), &active);
            previous = sample[0];
        }
        assert!(previous < -0.95);

        // Toggle to bypassed and ensure output does not jump directly to +1.0.
        let mut max_step = 0.0_f32;
        for _ in 0..160 {
            let mut sample = [1.0_f32];
            let mut buffer = [&mut sample[..]];
            wrapped.process(&mut buffer, &Transport::default(), &bypassed);

            let step = (sample[0] - previous).abs();
            max_step = max_step.max(step);
            previous = sample[0];
        }

        // Bounded transition: no full-scale discontinuity in one sample.
        assert!(max_step < 1.0);
        // Settles to dry signal after transition.
        assert!(previous > 0.95);
    }

    #[test]
    fn test_bypassed_transition_is_bidirectional() {
        let mut wrapped = Bypassed::new(PolarityFlip);

        let active = BypassedParams {
            inner: TestPassthroughParams,
            bypassed: false,
        };
        let bypassed = BypassedParams {
            inner: TestPassthroughParams,
            bypassed: true,
        };

        // Move into stable bypassed state.
        let mut sample_out = 0.0_f32;
        for _ in 0..160 {
            let mut sample = [1.0_f32];
            let mut buffer = [&mut sample[..]];
            wrapped.process(&mut buffer, &Transport::default(), &bypassed);
            sample_out = sample[0];
        }
        assert!(sample_out > 0.95);

        // Toggle back to active and ensure transition settles to wet (-1.0).
        let mut max_step = 0.0_f32;
        let mut previous = sample_out;
        for _ in 0..160 {
            let mut sample = [1.0_f32];
            let mut buffer = [&mut sample[..]];
            wrapped.process(&mut buffer, &Transport::default(), &active);

            let step = (sample[0] - previous).abs();
            max_step = max_step.max(step);
            previous = sample[0];
        }

        assert!(max_step < 1.0);
        assert!(previous < -0.95);
    }
}
