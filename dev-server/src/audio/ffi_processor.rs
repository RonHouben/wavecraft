//! FFI processor wrapper and dev audio processor trait.
//!
//! This module bridges the C-ABI `DevProcessorVTable` (loaded from the user's
//! cdylib) to a safe Rust trait that the audio server can drive.

use std::ffi::c_void;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use wavecraft_protocol::{DevProcessorVTable, OscilloscopeFrame, SignalChainSlot};

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

    /// Apply a signal-chain slot order on the control thread.
    fn set_signal_chain_order(&mut self, slots: &[SignalChainSlot]) -> bool;

    /// Drain the latest runtime-owned oscilloscope frame if one is available.
    fn take_latest_oscilloscope_frame(&mut self) -> Option<OscilloscopeFrame>;
}

/// Thread-safe control handle for a live FFI processor instance.
#[derive(Clone)]
pub struct FfiRuntimeControl {
    inner: Arc<SharedFfiProcessor>,
}

impl FfiRuntimeControl {
    pub fn set_signal_chain_order(&self, slots: &[SignalChainSlot]) -> bool {
        self.inner.set_signal_chain_order(slots)
    }

    pub fn take_latest_oscilloscope_frame(&self) -> Option<OscilloscopeFrame> {
        self.inner.take_latest_oscilloscope_frame()
    }
}

struct SharedFfiProcessor {
    instance: *mut c_void,
    vtable: DevProcessorVTable,
    supports_plain_values: bool,
    supports_runtime_parity_hooks: bool,
}

impl SharedFfiProcessor {
    fn set_signal_chain_order(&self, slots: &[SignalChainSlot]) -> bool {
        if !self.supports_runtime_parity_hooks {
            return false;
        }

        let Ok(json) = serde_json::to_string(slots) else {
            return false;
        };
        let Ok(c_string) = std::ffi::CString::new(json) else {
            return false;
        };

        // SAFETY: `self.instance` originates from the dylib vtable `create` function.
        // The v3 control hook is limited to thread-safe order application on the
        // generated runtime's pending-order state.
        unsafe { (self.vtable.set_signal_chain_order_json)(self.instance, c_string.as_ptr()) }
    }

    fn take_latest_oscilloscope_frame(&self) -> Option<OscilloscopeFrame> {
        if !self.supports_runtime_parity_hooks {
            return None;
        }

        let json_ptr = (self.vtable.take_latest_oscilloscope_frame_json)(self.instance);
        if json_ptr.is_null() {
            return None;
        }

        struct OwnedFfiJson {
            ptr: *mut std::os::raw::c_char,
        }

        impl Drop for OwnedFfiJson {
            fn drop(&mut self) {
                if !self.ptr.is_null() {
                    // SAFETY: pointer was allocated by CString::into_raw inside the dylib vtable
                    // implementation and is freed exactly once here by reconstructing the CString.
                    unsafe {
                        let _ = std::ffi::CString::from_raw(self.ptr);
                    }
                }
            }
        }

        let owned = OwnedFfiJson { ptr: json_ptr };
        // SAFETY: `owned.ptr` is checked non-null above and must point to a valid
        // NUL-terminated string returned by the dylib.
        let c_str = unsafe { std::ffi::CStr::from_ptr(owned.ptr) };
        let Ok(json) = c_str.to_str() else {
            return None;
        };
        serde_json::from_str::<Option<OscilloscopeFrame>>(json)
            .ok()
            .flatten()
    }
}

impl Drop for SharedFfiProcessor {
    fn drop(&mut self) {
        if !self.instance.is_null() {
            (self.vtable.drop)(self.instance);
            self.instance = std::ptr::null_mut();
        }
    }
}

// SAFETY: the shared processor instance is created on one thread, processed on the
// audio callback thread, and accessed from the control thread only through the v3
// runtime-parity hooks. Those hooks are narrowly scoped to thread-safe generated
// state (pending-order atomics + oscilloscope consumer mutex/ring buffer).
unsafe impl Send for SharedFfiProcessor {}
unsafe impl Sync for SharedFfiProcessor {}

/// Wraps a `DevProcessorVTable` into a safe `DevAudioProcessor`.
///
/// Owns the opaque processor instance and dispatches through vtable
/// function pointers. All allocation and deallocation happens inside
/// the dylib via the vtable — no cross-allocator issues.
pub struct FfiProcessor {
    inner: Arc<SharedFfiProcessor>,
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
            inner: Arc::new(SharedFfiProcessor {
                instance,
                vtable: *vtable,
                supports_plain_values: vtable.version >= 2,
                supports_runtime_parity_hooks: vtable.version >= 3,
            }),
            unsupported_channel_count: AtomicU32::new(0),
            unsupported_channel_flag: AtomicBool::new(false),
        })
    }

    pub fn runtime_control(&self) -> FfiRuntimeControl {
        FfiRuntimeControl {
            inner: Arc::clone(&self.inner),
        }
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
            !self.inner.instance.is_null(),
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

        (self.inner.vtable.process)(
            self.inner.instance,
            ptrs.as_mut_ptr(),
            num_channels,
            num_samples,
        );
    }

    fn apply_plain_values(&mut self, values: &[f32]) {
        if !self.inner.supports_plain_values {
            return;
        }

        // SAFETY: `self.instance` originates from the loaded vtable `create` function,
        // `values.as_ptr()` is valid for `values.len()` elements for this call, and
        // the plugin owns interpretation of plain-value order.
        unsafe {
            (self.inner.vtable.apply_plain_values)(
                self.inner.instance,
                values.as_ptr(),
                values.len(),
            );
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        (self.inner.vtable.set_sample_rate)(self.inner.instance, sample_rate);
    }

    fn reset(&mut self) {
        (self.inner.vtable.reset)(self.inner.instance);
    }

    fn set_signal_chain_order(&mut self, slots: &[SignalChainSlot]) -> bool {
        self.inner.set_signal_chain_order(slots)
    }

    fn take_latest_oscilloscope_frame(&mut self) -> Option<OscilloscopeFrame> {
        self.inner.take_latest_oscilloscope_frame()
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
    static SET_SIGNAL_CHAIN_ORDER_CALLED: AtomicBool = AtomicBool::new(false);
    static TAKE_OSCILLOSCOPE_FRAME_CALLED: AtomicBool = AtomicBool::new(false);

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
        SET_SIGNAL_CHAIN_ORDER_CALLED.store(false, Ordering::SeqCst);
        TAKE_OSCILLOSCOPE_FRAME_CALLED.store(false, Ordering::SeqCst);
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

    unsafe extern "C" fn mock_set_signal_chain_order_json(
        _instance: *mut c_void,
        _json_ptr: *const std::os::raw::c_char,
    ) -> bool {
        SET_SIGNAL_CHAIN_ORDER_CALLED.store(true, Ordering::SeqCst);
        true
    }

    extern "C" fn mock_take_latest_oscilloscope_frame_json(
        _instance: *mut c_void,
    ) -> *mut std::os::raw::c_char {
        TAKE_OSCILLOSCOPE_FRAME_CALLED.store(true, Ordering::SeqCst);
        std::ffi::CString::new(
            r#"{"points_l":[0.1],"points_r":[0.2],"sample_rate":48000.0,"timestamp":1,"no_signal":false,"trigger_mode":"risingZeroCrossing"}"#,
        )
        .unwrap()
        .into_raw()
    }

    fn mock_vtable() -> DevProcessorVTable {
        DevProcessorVTable {
            version: wavecraft_protocol::DEV_PROCESSOR_VTABLE_VERSION,
            create: mock_create,
            process: mock_process,
            apply_plain_values: mock_apply_plain_values,
            set_signal_chain_order_json: mock_set_signal_chain_order_json,
            take_latest_oscilloscope_frame_json: mock_take_latest_oscilloscope_frame_json,
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
    fn test_ffi_processor_runtime_parity_hooks() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_flags();
        let vtable = mock_vtable();

        let mut processor = FfiProcessor::new(&vtable).expect("create should succeed");
        let applied = processor.set_signal_chain_order(&[SignalChainSlot {
            id: "soft_clip".to_string(),
            slot_type: wavecraft_protocol::SlotType::Processor,
        }]);
        assert!(applied);
        assert!(SET_SIGNAL_CHAIN_ORDER_CALLED.load(Ordering::SeqCst));

        let frame = processor.take_latest_oscilloscope_frame();
        assert!(TAKE_OSCILLOSCOPE_FRAME_CALLED.load(Ordering::SeqCst));
        assert!(frame.is_some());
        assert_eq!(frame.unwrap().sample_rate, 48_000.0);
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
