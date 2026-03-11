use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use wavecraft_protocol::InputSourceKind;

const ORDERING: Ordering = Ordering::SeqCst;

#[derive(Clone)]
pub struct SharedInputSourceSelection {
    inner: Arc<AtomicU8>,
}

impl Default for SharedInputSourceSelection {
    fn default() -> Self {
        Self::new(InputSourceKind::HardwareInput)
    }
}

impl SharedInputSourceSelection {
    pub fn new(initial: InputSourceKind) -> Self {
        Self {
            inner: Arc::new(AtomicU8::new(encode_input_source(initial))),
        }
    }

    pub fn load(&self) -> InputSourceKind {
        decode_input_source(self.inner.load(ORDERING))
    }

    pub fn store(&self, source: InputSourceKind) {
        self.inner.store(encode_input_source(source), ORDERING);
    }
}

fn encode_input_source(source: InputSourceKind) -> u8 {
    match source {
        InputSourceKind::HardwareInput => 0,
        InputSourceKind::TestTone => 1,
    }
}

fn decode_input_source(raw: u8) -> InputSourceKind {
    match raw {
        1 => InputSourceKind::TestTone,
        _ => InputSourceKind::HardwareInput,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_input_source_selection_roundtrips() {
        let selection = SharedInputSourceSelection::default();
        assert_eq!(selection.load(), InputSourceKind::HardwareInput);

        selection.store(InputSourceKind::TestTone);
        assert_eq!(selection.load(), InputSourceKind::TestTone);
    }
}
