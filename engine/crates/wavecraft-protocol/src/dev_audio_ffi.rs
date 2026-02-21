//! C-ABI FFI contract for dev-mode audio processing.
//!
//! This module defines the shared interface between the `wavecraft_plugin!` macro
//! (which generates FFI exports in the user's cdylib) and the CLI consumer
//! (`wavecraft start`) that loads and calls them.
//!
//! # Design Principles
//!
//! - **`#[repr(C)]`** struct with `extern "C"` function pointers for ABI stability
//! - **`*mut c_void`** instance pointers for type erasure across the dylib boundary
//! - **Version field** for forward-compatible ABI evolution
//! - All memory alloc/dealloc stays inside the dylib (no cross-allocator issues)

use std::ffi::c_void;

/// C-ABI stable vtable for dev-mode audio processing.
///
/// This struct is returned by the `wavecraft_dev_create_processor` FFI symbol
/// exported from user plugins. It provides function pointers for creating,
/// using, and destroying a processor instance across the dylib boundary.
///
/// # ABI Stability
///
/// This struct uses `#[repr(C)]` and only `extern "C"` function pointers,
/// making it safe across separately compiled Rust binaries. All data passes
/// through primitive types (`f32`, `u32`, `*mut c_void`, `*mut *mut f32`).
///
/// # Versioning
///
/// A `version` field allows the CLI to detect incompatible vtable changes
/// and provide clear upgrade guidance instead of undefined behavior.
///
/// # Memory Ownership
///
/// ```text
/// create()  → Box::into_raw(Box::new(Processor))     [dylib allocates]
/// process() → &mut *(ptr as *mut Processor)           [dylib borrows]
/// drop()    → Box::from_raw(ptr as *mut Processor)    [dylib deallocates]
/// ```
///
/// The CLI never allocates or frees the processor memory; it only passes the
/// opaque pointer back into vtable functions.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DevProcessorVTable {
    /// VTable version. Must equal [`DEV_PROCESSOR_VTABLE_VERSION`].
    pub version: u32,

    /// Create a new processor instance.
    ///
    /// Returns an opaque pointer to a heap-allocated processor.
    /// The caller must eventually pass this pointer to `drop` to free it.
    pub create: extern "C" fn() -> *mut c_void,

    /// Process audio in deinterleaved (per-channel) format.
    ///
    /// # Arguments
    /// - `instance`: Opaque processor pointer from `create`
    /// - `channels`: Pointer to an array of `num_channels` mutable f32 pointers
    /// - `num_channels`: Number of audio channels (typically 2)
    /// - `num_samples`: Number of samples per channel
    ///
    /// # Safety
    /// - `instance` must be a valid pointer from `create`
    /// - `channels[0..num_channels]` must each point to `num_samples` valid f32s
    /// - Must be called from a single thread (not thread-safe)
    pub process: extern "C" fn(
        instance: *mut c_void,
        channels: *mut *mut f32,
        num_channels: u32,
        num_samples: u32,
    ),

    /// Apply a dense plain-value parameter snapshot to the processor instance.
    ///
    /// Values are ordered according to `ProcessorParams::param_specs()` in the
    /// generated plugin wrapper.
    ///
    /// # Safety
    /// - `instance` must be a valid pointer from `create`
    /// - `values_ptr` must reference `len` contiguous `f32` values
    /// - Caller must ensure the value order matches the processor parameter order
    pub apply_plain_values:
        unsafe extern "C" fn(instance: *mut c_void, values_ptr: *const f32, len: usize),

    /// Update the processor's sample rate.
    pub set_sample_rate: extern "C" fn(instance: *mut c_void, sample_rate: f32),

    /// Reset processor state (clear delay lines, filters, etc.).
    pub reset: extern "C" fn(instance: *mut c_void),

    /// Destroy the processor instance and free its memory.
    ///
    /// # Safety
    /// - `instance` must be a valid pointer from `create`
    /// - Must not be called more than once for the same pointer
    /// - No other vtable function may be called after `drop`
    pub drop: extern "C" fn(instance: *mut c_void),
}

/// Current vtable version.
///
/// v2 adds `apply_plain_values` to support block-boundary parameter injection
/// in dev FFI mode.
pub const DEV_PROCESSOR_VTABLE_VERSION: u32 = 2;

/// FFI symbol name exported by `wavecraft_plugin!` macro.
pub const DEV_PROCESSOR_SYMBOL: &[u8] = b"wavecraft_dev_create_processor\0";
