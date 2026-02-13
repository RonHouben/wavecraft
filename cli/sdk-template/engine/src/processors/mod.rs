// Processor modules — one file per processor.
//
// To add a new processor:
//   1. Create a file in this folder   (e.g. `filter.rs`)
//   2. Declare the module here         (e.g. `pub mod filter;`)
//   3. Re-export the processor type    (e.g. `pub use filter::Filter;`)
//   4. Wire it into the signal chain in `lib.rs`

pub mod oscillator;

pub use oscillator::Oscillator;
