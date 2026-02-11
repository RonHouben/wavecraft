//! FFI processor wrapper and dev audio processor trait.
//!
//! This module bridges the C-ABI `DevProcessorVTable` (loaded from the user's
//! cdylib) to a safe Rust trait that the audio server can drive.

use std::ffi::c_void;
use wavecraft_protocol::DevProcessorVTable;

/// Simplified audio processor trait for dev mode.
///
/// Unlike the full `wavecraft_dsp::Processor` trait, this has no associated
/// types and works with both direct Rust implementations and FFI-loaded
/// processors via type erasure.
pub trait DevAudioProcessor: Send + 'static {
    /// Process deinterleaved audio in-place.
    fn process(&mut self, channels: &mut [&mut [f32]]);

    /// Update the processor's sample rate.
    fn set_sample_rate(&mut self, sample_rate: f32);

    /// Reset processor state.
    fn reset(&mut self);
}

/// Wraps a `DevProcessorVTable` into a safe `DevAudioProcessor`.
///
/// Owns the opaque processor instance and dispatches through vtable
/// function pointers. All allocation and deallocation happens inside
/// the dylib via the vtable — no cross-allocator issues.
pub struct FfiProcessor {
    instance: *mut c_void,
    vtable: DevProcessorVTable,
}

// SAFETY: The processor instance is only accessed from the cpal audio
// callback thread (single-threaded access). The `Send` bound allows
// transferring it from the main thread (where it's created) to the
// audio thread. `FfiProcessor` is NOT `Sync` — no concurrent access.
unsafe impl Send for FfiProcessor {}

impl FfiProcessor {
    /// Create a new FFI processor from a loaded vtable.
    ///
    /// Calls the vtable's `create` function to allocate the processor
    /// inside the dylib. Returns `None` if `create` returns null
    /// (indicating a panic or allocation failure inside the dylib).
    pub fn new(vtable: &DevProcessorVTable) -> Option<Self> {
        let instance = (vtable.create)();
        if instance.is_null() {
            return None;
        }
        Some(Self {
            instance,
            vtable: *vtable,
        })
    }
}

impl DevAudioProcessor for FfiProcessor {
    fn process(&mut self, channels: &mut [&mut [f32]]) {
        let num_channels = channels.len() as u32;
        if num_channels == 0 || channels[0].is_empty() {
            return;
        }
        let num_samples = channels[0].len() as u32;

        // Real-time safety: use a stack-allocated array instead of Vec.
        // Wavecraft targets stereo (2 channels). Guard against unexpected
        // multi-channel input to avoid out-of-bounds access.
        if channels.len() > 2 {
            tracing::error!(
                num_channels = channels.len(),
                "FfiProcessor::process() received more than 2 channels; skipping"
            );
            return;
        }

        // Build fixed-size array of channel pointers for the C-ABI call.
        // No heap allocation — this lives on the stack.
        let mut ptrs: [*mut f32; 2] = [std::ptr::null_mut(); 2];
        for (i, ch) in channels.iter_mut().enumerate() {
            ptrs[i] = ch.as_mut_ptr();
        }

        (self.vtable.process)(self.instance, ptrs.as_mut_ptr(), num_channels, num_samples);
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        (self.vtable.set_sample_rate)(self.instance, sample_rate);
    }

    fn reset(&mut self) {
        (self.vtable.reset)(self.instance);
    }
}

impl Drop for FfiProcessor {
    fn drop(&mut self) {
        if !self.instance.is_null() {
            (self.vtable.drop)(self.instance);
            self.instance = std::ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    // Static flags for mock vtable functions
    static CREATE_CALLED: AtomicBool = AtomicBool::new(false);
    static PROCESS_CALLED: AtomicBool = AtomicBool::new(false);
    static SET_SAMPLE_RATE_CALLED: AtomicBool = AtomicBool::new(false);
    static RESET_CALLED: AtomicBool = AtomicBool::new(false);
    static DROP_CALLED: AtomicBool = AtomicBool::new(false);
    static PROCESS_CHANNELS: AtomicU32 = AtomicU32::new(0);
    static PROCESS_SAMPLES: AtomicU32 = AtomicU32::new(0);

    fn reset_flags() {
        CREATE_CALLED.store(false, Ordering::SeqCst);
        PROCESS_CALLED.store(false, Ordering::SeqCst);
        SET_SAMPLE_RATE_CALLED.store(false, Ordering::SeqCst);
        RESET_CALLED.store(false, Ordering::SeqCst);
        DROP_CALLED.store(false, Ordering::SeqCst);
        PROCESS_CHANNELS.store(0, Ordering::SeqCst);
        PROCESS_SAMPLES.store(0, Ordering::SeqCst);
    }

    extern "C" fn mock_create() -> *mut c_void {
        CREATE_CALLED.store(true, Ordering::SeqCst);
        // Return a non-null sentinel (we never dereference it in mock)
        std::ptr::dangling_mut::<c_void>()
    }

    extern "C" fn mock_create_null() -> *mut c_void {
        CREATE_CALLED.store(true, Ordering::SeqCst);
        std::ptr::null_mut()
    }

    extern "C" fn mock_process(
        _instance: *mut c_void,
        _channels: *mut *mut f32,
        num_channels: u32,
        num_samples: u32,
    ) {
        PROCESS_CALLED.store(true, Ordering::SeqCst);
        PROCESS_CHANNELS.store(num_channels, Ordering::SeqCst);
        PROCESS_SAMPLES.store(num_samples, Ordering::SeqCst);
    }

    extern "C" fn mock_set_sample_rate(_instance: *mut c_void, _sample_rate: f32) {
        SET_SAMPLE_RATE_CALLED.store(true, Ordering::SeqCst);
    }

    extern "C" fn mock_reset(_instance: *mut c_void) {
        RESET_CALLED.store(true, Ordering::SeqCst);
    }

    extern "C" fn mock_drop(_instance: *mut c_void) {
        DROP_CALLED.store(true, Ordering::SeqCst);
    }

    fn mock_vtable() -> DevProcessorVTable {
        DevProcessorVTable {
            version: wavecraft_protocol::DEV_PROCESSOR_VTABLE_VERSION,
            create: mock_create,
            process: mock_process,
            set_sample_rate: mock_set_sample_rate,
            reset: mock_reset,
            drop: mock_drop,
        }
    }

    #[test]
    fn test_ffi_processor_lifecycle() {
        reset_flags();
        let vtable = mock_vtable();

        let mut processor = FfiProcessor::new(&vtable).expect("create should succeed");
        assert!(CREATE_CALLED.load(Ordering::SeqCst));

        // Process some audio
        let mut left = vec![0.0f32; 128];
        let mut right = vec![0.0f32; 128];
        let mut channels: Vec<&mut [f32]> = vec![&mut left, &mut right];
        processor.process(&mut channels);
        assert!(PROCESS_CALLED.load(Ordering::SeqCst));
        assert_eq!(PROCESS_CHANNELS.load(Ordering::SeqCst), 2);
        assert_eq!(PROCESS_SAMPLES.load(Ordering::SeqCst), 128);

        // Drop should call vtable.drop
        drop(processor);
        assert!(DROP_CALLED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_ffi_processor_set_sample_rate_and_reset() {
        reset_flags();
        let vtable = mock_vtable();

        let mut processor = FfiProcessor::new(&vtable).expect("create should succeed");

        processor.set_sample_rate(48000.0);
        assert!(SET_SAMPLE_RATE_CALLED.load(Ordering::SeqCst));

        processor.reset();
        assert!(RESET_CALLED.load(Ordering::SeqCst));

        drop(processor);
    }

    #[test]
    fn test_ffi_processor_null_create_returns_none() {
        reset_flags();
        let mut vtable = mock_vtable();
        vtable.create = mock_create_null;

        let result = FfiProcessor::new(&vtable);
        assert!(CREATE_CALLED.load(Ordering::SeqCst));
        assert!(
            result.is_none(),
            "Should return None when create returns null"
        );
    }

    #[test]
    fn test_ffi_processor_empty_channels_noop() {
        reset_flags();
        let vtable = mock_vtable();
        let mut processor = FfiProcessor::new(&vtable).expect("create should succeed");

        // Empty channels → should not call process
        PROCESS_CALLED.store(false, Ordering::SeqCst);
        let mut channels: Vec<&mut [f32]> = vec![];
        processor.process(&mut channels);
        assert!(
            !PROCESS_CALLED.load(Ordering::SeqCst),
            "Should not call vtable.process with empty channels"
        );

        drop(processor);
    }
}
