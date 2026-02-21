//! Passthrough processor that leaves input audio unchanged.

use wavecraft_dsp::{ParamSpec, Processor, ProcessorParams, Transport};

/// Empty parameter struct for passthrough processor.
#[derive(Debug, Default, Clone)]
pub struct PassthroughParams;

impl ProcessorParams for PassthroughParams {
    fn param_specs() -> &'static [ParamSpec] {
        &[] // No parameters
    }
}

/// Passthrough processor that does not modify the audio signal.
///
/// This processor is useful for:
/// - Testing signal chain composition
/// - Placeholder in development
/// - Bypass functionality
#[derive(Debug, Default)]
pub struct PassthroughDsp;

impl Processor for PassthroughDsp {
    type Params = PassthroughParams;

    fn process(
        &mut self,
        _buffer: &mut [&mut [f32]],
        _transport: &Transport,
        _params: &Self::Params,
    ) {
        // No-op: audio passes through unchanged
    }

    fn set_sample_rate(&mut self, _sample_rate: f32) {
        // No state dependent on sample rate
    }

    fn reset(&mut self) {
        // No state to reset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_passthrough_unchanged(mut input: [f32; 4]) {
        let mut processor = PassthroughDsp;
        let original = input;
        let mut buffer = [&mut input[..]];

        let params = PassthroughParams;
        let transport = Transport::default();

        processor.process(&mut buffer, &transport, &params);

        assert_eq!(input, original);
    }

    #[test]
    fn test_passthrough() {
        assert_passthrough_unchanged([0.5, -0.5, 0.25, -0.25]);
    }

    #[test]
    fn test_no_params() {
        let specs = PassthroughParams::param_specs();
        assert_eq!(specs.len(), 0);
    }
}
