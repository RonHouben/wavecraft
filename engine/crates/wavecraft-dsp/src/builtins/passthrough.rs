//! Passthrough processor - does nothing (useful for testing and placeholders).

use crate::traits::{Processor, ProcessorParams, Transport};

/// Empty parameter struct for passthrough processor.
#[derive(Debug, Default, Clone)]
pub struct PassthroughParams;

impl ProcessorParams for PassthroughParams {
    fn param_specs() -> &'static [crate::traits::ParamSpec] {
        &[] // No parameters
    }
}

/// Passthrough processor - does not modify the audio signal.
///
/// This processor is useful for:
/// - Testing signal chain composition
/// - Placeholder in development
/// - Bypass functionality
#[derive(Debug, Default)]
pub struct PassthroughDsp;

impl Processor for PassthroughDsp {
    type Params = PassthroughParams;
    
    fn process(&mut self, _buffer: &mut [&mut [f32]], _transport: &Transport, _params: &Self::Params) {
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
    
    #[test]
    fn test_passthrough() {
        let mut processor = PassthroughDsp::default();
        let original = [0.5, -0.5, 0.25, -0.25];
        let mut data = original;
        let mut buffer = [&mut data[..]];
        
        let params = PassthroughParams;
        let transport = Transport::default();
        
        processor.process(&mut buffer, &transport, &params);
        
        // Data should be unchanged
        for (i, &sample) in data.iter().enumerate() {
            assert_eq!(sample, original[i]);
        }
    }
    
    #[test]
    fn test_no_params() {
        let specs = PassthroughParams::param_specs();
        assert_eq!(specs.len(), 0);
    }
}
