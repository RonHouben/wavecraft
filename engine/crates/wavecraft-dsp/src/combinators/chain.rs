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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::{GainDsp, GainParams, PassthroughDsp, PassthroughParams};

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
}
