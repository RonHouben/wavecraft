use proc_macro2::TokenStream;
use quote::quote;

/// Generate all FFI export functions for parameter discovery and dev-audio processing.
pub(super) fn build(
    krate: &syn::Path,
    processor_param_mappings: &[TokenStream],
    processor_info_entries: &[TokenStream],
) -> TokenStream {
    quote! {
        // ================================================================
        // FFI Exports for Parameter Discovery (used by `wavecraft start`)
        // ================================================================

        /// Returns JSON-serialized parameter specifications.
        ///
        /// # Safety
        /// The returned pointer must be freed with `wavecraft_free_string`.
        #[unsafe(no_mangle)]
        pub extern "C" fn wavecraft_get_params_json() -> *mut ::std::ffi::c_char {
            let mut params: ::std::vec::Vec<#krate::__internal::ParameterInfo> =
                ::std::vec::Vec::new();
            #(#processor_param_mappings)*
            let json = #krate::__internal::serde_json::to_string(&params)
                .unwrap_or_else(|_| "[]".to_string());
            ::std::ffi::CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(::std::ptr::null_mut())
        }

        /// Returns JSON-serialized processor metadata.
        ///
        /// # Safety
        /// The returned pointer must be freed with `wavecraft_free_string`.
        #[unsafe(no_mangle)]
        pub extern "C" fn wavecraft_get_processors_json() -> *mut ::std::ffi::c_char {
            let processors: ::std::vec::Vec<#krate::__internal::ProcessorInfo> = vec![
                #(#processor_info_entries),*
            ];
            let json = #krate::__internal::serde_json::to_string(&processors)
                .unwrap_or_else(|_| "[]".to_string());
            ::std::ffi::CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(::std::ptr::null_mut())
        }

        /// Frees a string returned by `wavecraft_get_params_json`.
        ///
        /// # Safety
        /// The pointer must have been returned by `wavecraft_get_params_json`.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn wavecraft_free_string(ptr: *mut ::std::ffi::c_char) {
            if !ptr.is_null() {
                let _ = ::std::ffi::CString::from_raw(ptr);
            }
        }

        // ================================================================
        // FFI Exports for Dev Audio Processing (used by `wavecraft start`)
        // ================================================================

        /// Returns a C-ABI vtable for creating and driving the plugin's audio
        /// processor from the CLI dev server (in-process audio via FFI).
        ///
        /// Each inner function is wrapped in `catch_unwind` to prevent panics
        /// from unwinding across the FFI boundary.
        #[unsafe(no_mangle)]
        pub extern "C" fn wavecraft_dev_create_processor() -> #krate::__internal::DevProcessorVTable {
            use ::std::ffi::c_void;

            type __P = __ProcessorType;
            type __Params = <__P as #krate::Processor>::Params;
            struct __DevProcessorInstance {
                processor: __P,
                params: __Params,
            }

            extern "C" fn create() -> *mut c_void {
                let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    let instance = ::std::boxed::Box::new(__DevProcessorInstance {
                        processor: <__P as ::std::default::Default>::default(),
                        params: <__Params as #krate::ProcessorParams>::from_param_defaults(),
                    });
                    ::std::boxed::Box::into_raw(instance) as *mut c_void
                }));
                match result {
                    Ok(ptr) => ptr,
                    Err(_) => ::std::ptr::null_mut(),
                }
            }

            extern "C" fn process(
                instance: *mut c_void,
                channels: *mut *mut f32,
                num_channels: u32,
                num_samples: u32,
            ) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null()
                        || channels.is_null()
                        || num_channels == 0
                        || num_samples == 0
                    {
                        return;
                    }
                    // SAFETY: `instance` is non-null (checked above), was created by
                    // `Box::into_raw` in the `create` function, and has not been freed.
                    // The cast is sound because the pointer type matches `__DevProcessorInstance`.
                    let instance =
                        unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    let num_ch = num_channels as usize;
                    let num_samp = num_samples as usize;
                    let transport = #krate::Transport::default();

                    match num_ch {
                        1 => {
                            // SAFETY: `channels` is non-null (checked above) and points to
                            // at least `num_channels` (== 1) valid f32 pointers; index 0 is valid.
                            let ch0_ptr = unsafe { *channels.add(0) };
                            if ch0_ptr.is_null() {
                                return;
                            }
                            // SAFETY: `ch0_ptr` is non-null (checked above), points to
                            // `num_samp` f32 values owned by the caller, valid for this call.
                            let ch0 = unsafe {
                                ::std::slice::from_raw_parts_mut(ch0_ptr, num_samp)
                            };
                            let mut channel_slices: [&mut [f32]; 1] = [ch0];
                            #krate::Processor::process(
                                &mut instance.processor,
                                &mut channel_slices,
                                &transport,
                                &instance.params,
                            );
                        }
                        2 => {
                            // SAFETY: `channels` is non-null (checked above) and points to
                            // at least `num_channels` (== 2) valid f32 pointers; indices 0
                            // and 1 are both valid.
                            let ch0_ptr = unsafe { *channels.add(0) };
                            let ch1_ptr = unsafe { *channels.add(1) };
                            if ch0_ptr.is_null() || ch1_ptr.is_null() {
                                return;
                            }
                            // SAFETY: `ch0_ptr` / `ch1_ptr` are non-null (checked above),
                            // point to `num_samp` f32 values owned by the caller, valid for
                            // this call. The two pointers are disjoint by caller contract.
                            let ch0 = unsafe {
                                ::std::slice::from_raw_parts_mut(ch0_ptr, num_samp)
                            };
                            let ch1 = unsafe {
                                ::std::slice::from_raw_parts_mut(ch1_ptr, num_samp)
                            };
                            let mut channel_slices: [&mut [f32]; 2] = [ch0, ch1];
                            #krate::Processor::process(
                                &mut instance.processor,
                                &mut channel_slices,
                                &transport,
                                &instance.params,
                            );
                        }
                        _ => {} // Unsupported channel topology in dev-FFI path
                    }
                }));
            }

            unsafe extern "C" fn apply_plain_values(
                instance: *mut c_void,
                values_ptr: *const f32,
                len: usize,
            ) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() {
                        return;
                    }
                    if values_ptr.is_null() && len != 0 {
                        return;
                    }
                    // SAFETY: `instance` is non-null (checked above), was created by
                    // `Box::into_raw` in `create`, and has not been freed.
                    let instance =
                        unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    let values: &[f32] = if len == 0 {
                        &[]
                    } else {
                        // SAFETY: `values_ptr` is non-null (checked above), points to `len`
                        // valid f32 values provided by the caller, valid for this call.
                        unsafe { ::std::slice::from_raw_parts(values_ptr, len) }
                    };
                    <__Params as #krate::ProcessorParams>::apply_plain_values(
                        &mut instance.params,
                        values,
                    );
                }));
            }

            extern "C" fn set_sample_rate(instance: *mut c_void, sample_rate: f32) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() {
                        return;
                    }
                    // SAFETY: `instance` is non-null (checked above), was created by
                    // `Box::into_raw` in `create`, and has not been freed.
                    let instance =
                        unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    #krate::Processor::set_sample_rate(&mut instance.processor, sample_rate);
                }));
            }

            extern "C" fn reset(instance: *mut c_void) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() {
                        return;
                    }
                    // SAFETY: `instance` is non-null (checked above), was created by
                    // `Box::into_raw` in `create`, and has not been freed.
                    let instance =
                        unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    #krate::Processor::reset(&mut instance.processor);
                }));
            }

            extern "C" fn drop_fn(instance: *mut c_void) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if !instance.is_null() {
                        // SAFETY: `instance` was created by `Box::into_raw` in `create`,
                        // is non-null (checked above), and `drop_fn` is called exactly once
                        // per the vtable contract. Ownership is transferred back to Box.
                        let _ = unsafe {
                            ::std::boxed::Box::from_raw(
                                instance as *mut __DevProcessorInstance,
                            )
                        };
                    }
                }));
            }

            #krate::__internal::DevProcessorVTable {
                version: #krate::__internal::DEV_PROCESSOR_VTABLE_VERSION,
                create,
                process,
                apply_plain_values,
                set_sample_rate,
                reset,
                drop: drop_fn,
            }
        }
    }
}
