use wavecraft::prelude::*;

wavecraft_processor!(InputGain => Gain);
wavecraft_processor!(OutputGain => Gain);
wavecraft_processor!(AnotherGain => Gain);

wavecraft_plugin! {
    name: "Wavecraft Example",
    signal: SignalChain![InputGain, AnotherGain, OutputGain],
}
