// Full variant compile test for wavecraft_plugin! macro
// This file exercises the macro with the same fields but references the crate's params

struct FullProcessor;
impl FullProcessor { fn new() -> Self { Self } fn set_sample_rate(&mut self, _sr: f32) {} }
impl Default for FullProcessor { fn default() -> Self { Self::new() } }

// Define a local Params type to ensure macro works with any Params type
#[derive(Default)]
struct LocalParams;

wavecraft_plugin! {
    ident: FullPlugin,
    name: "Full Plugin",
    vendor: "ExampleCo",
    url: "https://exampleco.example",
    email: "support@exampleco.example",
    version: env!("CARGO_PKG_VERSION"),
    audio: { inputs: 2, outputs: 2 },
    params: [LocalParams],
    processor: FullProcessor,
}

fn main() {
    let _ = FullPlugin::default();
}
