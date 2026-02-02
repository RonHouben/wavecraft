use nih_plug::prelude::*;
use vstkit_core::vstkit_plugin;

struct MinimalProcessor;
impl MinimalProcessor {
    fn new() -> Self { Self }
    fn set_sample_rate(&mut self, _sr: f32) {}
}
impl Default for MinimalProcessor { fn default() -> Self { Self::new() } }

#[derive(Params)]
struct LocalParams {
    #[id = "p"]
    p: FloatParam,
}

impl Default for LocalParams {
    fn default() -> Self {
        Self {
            p: FloatParam::new("P", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
        }
    }
}

vstkit_plugin! {
    ident: MinimalPlugin,
    name: "Minimal Plugin",
    vendor: "Example",
    url: "https://example.com",
    email: "contact@example.com",
    version: env!("CARGO_PKG_VERSION"),
    audio: { inputs: 2, outputs: 2 },
    params: [LocalParams],
    processor: MinimalProcessor,
}

fn main() {
    let _ = MinimalPlugin::default();
}
