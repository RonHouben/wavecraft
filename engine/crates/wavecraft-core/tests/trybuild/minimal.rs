// Minimal compile test for wavecraft_plugin! macro

// Use crate paths explicitly so trybuild compiles this as a separate crate
use wavecraft_core::params::WavecraftParams;

struct MinimalProcessor;
impl MinimalProcessor {
    fn new() -> Self { Self }
    fn set_sample_rate(&mut self, _sr: f32) {}
}
impl Default for MinimalProcessor { fn default() -> Self { Self::new() } }

wavecraft_plugin! {
    ident: MinimalPlugin,
    name: "Minimal Plugin",
    vendor: "Example",
    url: "https://example.com",
    email: "contact@example.com",
    version: env!("CARGO_PKG_VERSION"),
    audio: { inputs: 2, outputs: 2 },
    params: [wavecraft_core::params::WavecraftParams],
    processor: MinimalProcessor,
}

fn main() {
    let _ = MinimalPlugin::default();
}
