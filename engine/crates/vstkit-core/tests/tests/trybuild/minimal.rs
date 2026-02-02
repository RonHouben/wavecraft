// Minimal compile test for vstkit_plugin! macro

// Use crate paths explicitly so trybuild compiles this as a separate crate
use vstkit_core::params::VstKitParams;

struct MinimalProcessor;
impl MinimalProcessor {
    fn new() -> Self { Self }
    fn set_sample_rate(&mut self, _sr: f32) {}
}
impl Default for MinimalProcessor { fn default() -> Self { Self::new() } }

vstkit_plugin! {
    ident: MinimalPlugin,
    name: "Minimal Plugin",
    vendor: "Example",
    url: "https://example.com",
    email: "contact@example.com",
    version: env!("CARGO_PKG_VERSION"),
    audio: { inputs: 2, outputs: 2 },
    params: [vstkit_core::params::VstKitParams],
    processor: MinimalProcessor,
}

fn main() {
    let _ = MinimalPlugin::default();
}
