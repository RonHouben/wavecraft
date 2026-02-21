//! FFI processor wrapper and dev audio processor trait.
//!
//! This module bridges the C-ABI `DevProcessorVTable` (loaded from the user's
//! cdylib) to a safe Rust trait that the audio server can drive.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use wavecraft_protocol::DevProcessorVTable;

/// Simplified audio processor trait for dev mode.
///
/// Unlike the full `wavecraft_dsp::Processor` trait, this has no associated
/// types and works with both direct Rust implementations and FFI-loaded
/// processors via type erasure.
pub trait DevAudioProcessor: Send + 'static {
    /// Process deinterleaved audio in-place.
    fn process(&mut self, channels: &mut [&mut [f32]]);

    /// Apply plain parameter values in canonical generation order.
    fn apply_plain_values(&mut self, values: &[f32]);

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
    supports_plain_values: bool,
    unsupported_channel_count: AtomicU32,
    unsupported_channel_flag: AtomicBool,
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
            supports_plain_values: vtable.version >= 2,
            unsupported_channel_count: AtomicU32::new(0),
            unsupported_channel_flag: AtomicBool::new(false),
        })
    }

    fn process_dimensions(channels: &[&mut [f32]]) -> Option<(u32, u32)> {
        let num_channels = channels.len() as u32;
        if num_channels == 0 || channels[0].is_empty() {
            return None;
        }

        Some((num_channels, channels[0].len() as u32))
    }

    fn prepare_channel_ptrs(&self, channels: &mut [&mut [f32]]) -> Option<[*mut f32; 2]> {
        // Real-time safety: use a stack-allocated array instead of Vec.
        // Wavecraft targets stereo (2 channels). Guard against unexpected
        // multi-channel input to avoid out-of-bounds access.
        if channels.len() > 2 {
            // Real-time safe reporting: set a one-shot flag and count events.
            // A non-RT path can poll and report via `take_unsupported_channel_count`
            // and `take_unsupported_channel_flag` if needed.
            self.unsupported_channel_count
                .fetch_add(1, Ordering::Relaxed);
            self.unsupported_channel_flag.store(true, Ordering::Relaxed);
            return None;
        }

        // Build fixed-size array of channel pointers for the C-ABI call.
        // No heap allocation — this lives on the stack.
        let mut ptrs: [*mut f32; 2] = [std::ptr::null_mut(); 2];
        for (index, channel) in channels.iter_mut().enumerate() {
            ptrs[index] = channel.as_mut_ptr();
        }

        Some(ptrs)
    }

    /// Non-RT diagnostic hook: returns and resets the count of callback
    /// invocations that were skipped due to receiving more than 2 channels.
    pub fn take_unsupported_channel_count(&self) -> u32 {
        self.unsupported_channel_count.swap(0, Ordering::Relaxed)
    }

    /// Non-RT diagnostic hook: returns whether any unsupported channel event
    /// occurred since the last call, then clears the flag.
    pub fn take_unsupported_channel_flag(&self) -> bool {
        self.unsupported_channel_flag.swap(false, Ordering::Relaxed)
    }
}

impl DevAudioProcessor for FfiProcessor {
    fn process(&mut self, channels: &mut [&mut [f32]]) {
        let Some((num_channels, num_samples)) = Self::process_dimensions(channels) else {
            return;
        };

        debug_assert!(
            !self.instance.is_null(),
            "FFI processor instance should be valid"
        );
        debug_assert!(
            channels
                .iter()
                .all(|channel| channel.len() == num_samples as usize),
            "FFI processor expects channel slices with equal lengths"
        );

        let Some(mut ptrs) = self.prepare_channel_ptrs(channels) else {
            return;
        };

        (self.vtable.process)(self.instance, ptrs.as_mut_ptr(), num_channels, num_samples);
    }

    fn apply_plain_values(&mut self, values: &[f32]) {
        if !self.supports_plain_values {
            return;
        }

        // SAFETY: `self.instance` originates from the loaded vtable `create` function,
        // `values.as_ptr()` is valid for `values.len()` elements for this call, and
        // the plugin owns interpretation of plain-value order.
        unsafe {
            (self.vtable.apply_plain_values)(self.instance, values.as_ptr(), values.len());
        }
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
    use std::sync::Mutex;

    // Mutex to serialize tests that share static mock flags.
    // This prevents race conditions when tests run in parallel.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    // Static flags for mock vtable functions
    static CREATE_CALLED: AtomicBool = AtomicBool::new(false);
    static PROCESS_CALLED: AtomicBool = AtomicBool::new(false);
    static SET_SAMPLE_RATE_CALLED: AtomicBool = AtomicBool::new(false);
    static RESET_CALLED: AtomicBool = AtomicBool::new(false);
    static DROP_CALLED: AtomicBool = AtomicBool::new(false);
    static APPLY_PLAIN_VALUES_CALLED: AtomicBool = AtomicBool::new(false);
    static APPLY_PLAIN_VALUES_LEN: AtomicU32 = AtomicU32::new(0);
    static PROCESS_CHANNELS: AtomicU32 = AtomicU32::new(0);
    static PROCESS_SAMPLES: AtomicU32 = AtomicU32::new(0);

    fn reset_flags() {
        CREATE_CALLED.store(false, Ordering::SeqCst);
        PROCESS_CALLED.store(false, Ordering::SeqCst);
        SET_SAMPLE_RATE_CALLED.store(false, Ordering::SeqCst);
        RESET_CALLED.store(false, Ordering::SeqCst);
        DROP_CALLED.store(false, Ordering::SeqCst);
        APPLY_PLAIN_VALUES_CALLED.store(false, Ordering::SeqCst);
        APPLY_PLAIN_VALUES_LEN.store(0, Ordering::SeqCst);
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

    unsafe extern "C" fn mock_apply_plain_values(
        _instance: *mut c_void,
        _values_ptr: *const f32,
        len: usize,
    ) {
        APPLY_PLAIN_VALUES_CALLED.store(true, Ordering::SeqCst);
        APPLY_PLAIN_VALUES_LEN.store(len as u32, Ordering::SeqCst);
    }

    fn mock_vtable() -> DevProcessorVTable {
        DevProcessorVTable {
            version: wavecraft_protocol::DEV_PROCESSOR_VTABLE_VERSION,
            create: mock_create,
            process: mock_process,
            apply_plain_values: mock_apply_plain_values,
            set_sample_rate: mock_set_sample_rate,
            reset: mock_reset,
            drop: mock_drop,
        }
    }

    #[test]
    fn test_ffi_processor_lifecycle() {
        let _guard = TEST_LOCK.lock().unwrap();
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
        let _guard = TEST_LOCK.lock().unwrap();
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
    fn test_ffi_processor_apply_plain_values() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_flags();
        let vtable = mock_vtable();

        let mut processor = FfiProcessor::new(&vtable).expect("create should succeed");
        processor.apply_plain_values(&[0.1, 0.2, 0.3]);

        assert!(APPLY_PLAIN_VALUES_CALLED.load(Ordering::SeqCst));
        assert_eq!(APPLY_PLAIN_VALUES_LEN.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_ffi_processor_null_create_returns_none() {
        let _guard = TEST_LOCK.lock().unwrap();
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
        let _guard = TEST_LOCK.lock().unwrap();
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

    #[test]
    fn test_ffi_processor_multichannel_records_rt_safe_diagnostic() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_flags();
        let vtable = mock_vtable();
        let mut processor = FfiProcessor::new(&vtable).expect("create should succeed");

        // More than 2 channels should skip processing and record diagnostics.
        PROCESS_CALLED.store(false, Ordering::SeqCst);
        let mut ch1 = vec![0.0f32; 16];
        let mut ch2 = vec![0.0f32; 16];
        let mut ch3 = vec![0.0f32; 16];
        let mut channels: Vec<&mut [f32]> = vec![&mut ch1, &mut ch2, &mut ch3];

        processor.process(&mut channels);

        assert!(
            !PROCESS_CALLED.load(Ordering::SeqCst),
            "Should not call vtable.process when channel count > 2"
        );
        assert!(processor.take_unsupported_channel_flag());
        assert_eq!(processor.take_unsupported_channel_count(), 1);

        // Hooks are one-shot/resetting.
        assert!(!processor.take_unsupported_channel_flag());
        assert_eq!(processor.take_unsupported_channel_count(), 0);

        drop(processor);
    }
}
