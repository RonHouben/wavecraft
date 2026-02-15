// ExampleProcessor — minimal custom processor template.
//
// This processor applies a fixed 2x gain to all channels/samples.
// Use it as a starting point for your own custom DSP.

use wavecraft::prelude::*;

#[derive(Clone, Default)]
pub struct ExampleProcessorParams;

impl ProcessorParams for ExampleProcessorParams {
    fn param_specs() -> &'static [ParamSpec] {
        &[]
    }

    fn from_param_defaults() -> Self {
        Self
    }
}

#[derive(Default)]
pub struct ExampleProcessor;

impl Processor for ExampleProcessor {
    type Params = ExampleProcessorParams;

    fn process(
        &mut self,
        buffer: &mut [&mut [f32]],
        _transport: &Transport,
        _params: &Self::Params,
    ) {
        for channel in buffer.iter_mut() {
            for sample in channel.iter_mut() {
                *sample *= 2.0;
            }
        }
    }
}
