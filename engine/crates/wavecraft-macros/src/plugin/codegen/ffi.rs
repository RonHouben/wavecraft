use proc_macro2::TokenStream;
use quote::quote;

/// Generate all FFI export functions for parameter discovery and dev-audio processing.
pub(super) fn build(
    krate: &syn::Path,
    processor_param_mappings: &[TokenStream],
    processor_info_entries: &[TokenStream],
    s: &super::context::SharedSymbols,
    sc: &super::signal_chain::SignalChainTokens,
) -> TokenStream {
    let proc_idx_usize = &s.proc_idx_usize;
    let n_lit = &s.n_lit;
    let t_lit = &s.t_lit;
    let tap_idx_usize = &s.tap_idx_usize;
    let tap_boundary_slot_idx = &s.tap_boundary_slot_idx;

    let super::signal_chain::SignalChainTokens {
        proc_struct_fields,
        proc_defaults,
        set_sample_rate_calls,
        reset_calls,
        dispatch_arms,
        param_count_exprs,
        tap_struct_fields,
        tap_defaults,
        tap_initialize,
        tap_reset,
        tap_capture_vars,
        tap_capture_boundary_zero,
        tap_capture_after_dispatch,
        tap_capture_flush,
        tap_observe_calls,
        ..
    } = sc;

    let tap_boundary_handoff_from_params = if tap_idx_usize.is_empty() {
        quote! {}
    } else {
        quote! {
            #(
                self.__tap_boundaries[#tap_idx_usize] = params
                    .__pending_slots[#tap_boundary_slot_idx]
                    .load(::std::sync::atomic::Ordering::Acquire);
            )*
        }
    };

    let current_tap_boundaries = if tap_idx_usize.is_empty() {
        quote! { let __tap_boundaries = [0u8; #t_lit]; }
    } else {
        quote! { let __tap_boundaries = self.__tap_boundaries; }
    };

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

        /// Returns JSON-serialized declared/default signal-chain slots.
        ///
        /// # Safety
        /// The returned pointer must be freed with `wavecraft_free_string`.
        #[unsafe(no_mangle)]
        pub extern "C" fn wavecraft_get_signal_chain_slots_json() -> *mut ::std::ffi::c_char {
            let slots = <__WavecraftParams as #krate::__nih::SignalChainOrderAccess>::get_order(
                &__WavecraftParams::default(),
            );
            let json = #krate::__internal::serde_json::to_string(&slots)
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

            const __DEV_FFI_MAX_BUFFER_SIZE: usize = 8_192;

            struct __DevProcessorState {
                #(#proc_struct_fields)*
                __current_order: [u8; #n_lit],
                __param_offsets: [usize; { #n_lit + 1 }],
                __cf_pos: usize,
                __cf_dir: i8,
                __param_scratch: ::std::vec::Vec<f32>,
                #tap_struct_fields
            }

            impl __DevProcessorState {
                fn default_state() -> (Self, #krate::OscilloscopeFrameConsumer) {
                    let (__oscilloscope_producer, __oscilloscope_consumer) =
                        #krate::create_oscilloscope_channel(8);

                    let __param_offsets: [usize; { #n_lit + 1 }] = {
                        let __counts: [usize; #n_lit] = [#(#param_count_exprs),*];
                        let mut __offs = [0usize; { #n_lit + 1 }];
                        let mut __acc = 0usize;
                        let mut __i = 0usize;
                        while __i < #n_lit {
                            __acc += __counts[__i];
                            __offs[__i + 1] = __acc;
                            __i += 1;
                        }
                        __offs
                    };

                    let mut __state = Self {
                        #(#proc_defaults)*
                        __current_order: [#(#proc_idx_usize as u8),*],
                        __param_offsets,
                        __cf_pos: 0,
                        __cf_dir: 0i8,
                        __param_scratch: vec![0.0_f32; __param_offsets[#n_lit]],
                        #tap_defaults
                    };
                    __state.initialize_for_dev_runtime(44_100.0);
                    (__state, __oscilloscope_consumer)
                }

                fn initialize_for_dev_runtime(&mut self, __sample_rate: f32) {
                    let _buffer_config = #krate::__nih::BufferConfig {
                        sample_rate: __sample_rate,
                        min_buffer_size: None,
                        max_buffer_size: __DEV_FFI_MAX_BUFFER_SIZE as u32,
                        process_mode: #krate::__nih::ProcessMode::Realtime,
                    };

                    #(#set_sample_rate_calls)*
                    #tap_initialize
                }

                fn reset_for_dev_runtime(&mut self) {
                    #(#reset_calls)*
                    #tap_reset
                }

                fn process_block(
                    &mut self,
                    params: &__WavecraftParams,
                    channels: *mut *mut f32,
                    num_channels: u32,
                    num_samples: u32,
                ) {
                    if channels.is_null() || num_channels == 0 || num_samples == 0 {
                        return;
                    }

                    // ── Block-start order handoff ─────────────────────────────
                    if self.__cf_dir == -1i8 && self.__cf_pos == 0 {
                        params
                            .__has_pending_order
                            .store(false, ::std::sync::atomic::Ordering::Release);
                        #(
                            self.__current_order[#proc_idx_usize] = params
                                .__pending_slots[#proc_idx_usize]
                                .load(::std::sync::atomic::Ordering::Acquire);
                        )*
                        #tap_boundary_handoff_from_params
                        self.__cf_dir = 1i8;
                    }
                    if self.__cf_dir == 0i8
                        && params
                            .__has_pending_order
                            .load(::std::sync::atomic::Ordering::Acquire)
                    {
                        self.__cf_pos = __CROSSFADE_SAMPLES;
                        self.__cf_dir = -1i8;
                    }

                    let __num_samples = num_samples as usize;
                    let __channels = num_channels as usize;

                    // SAFETY: `channels` is checked non-null above and points to at least
                    // `num_channels` pointers provided by the caller for the duration of this call.
                    let __left_ptr = unsafe { *channels.add(0) };
                    if __left_ptr.is_null() {
                        return;
                    }
                    // SAFETY: `__left_ptr` is non-null and valid for `__num_samples` samples.
                    let __left = unsafe {
                        ::std::slice::from_raw_parts_mut(__left_ptr, __num_samples)
                    };
                    let mut __right = None;
                    if __channels >= 2 {
                        // SAFETY: `channels` points to at least two channel pointers when
                        // `__channels >= 2`.
                        let __right_ptr = unsafe { *channels.add(1) };
                        if !__right_ptr.is_null() {
                            // SAFETY: `__right_ptr` is non-null and valid for `__num_samples`.
                            __right = Some(unsafe {
                                ::std::slice::from_raw_parts_mut(__right_ptr, __num_samples)
                            });
                        }
                    }

                    let __current_order = self.__current_order;
                    #current_tap_boundaries

                    // SAFETY: `__self_ptr` is derived from the exclusive `&mut self` receiver,
                    // is only used within this call, and never escapes.
                    let __self_ptr: *mut Self = self;

                    for __sample_idx in 0..__num_samples {
                        let __transport = #krate::Transport::default();
                        let mut __stereo: [f32; 2] = [
                            __left[__sample_idx],
                            __right
                                .as_ref()
                                .map(|__channel| __channel[__sample_idx])
                                .unwrap_or(0.0_f32),
                        ];
                        let __active_ch = __channels.min(2);

                        #tap_capture_vars
                        #tap_capture_boundary_zero

                        {
                            let (__sl0, __sl1) = __stereo.split_at_mut(1);
                            let mut __channel_slices: [&mut [f32]; 2] = [__sl0, __sl1];
                            let mut __pos: usize = 0;
                            while __pos < __PROC_COUNT {
                                let __slot = __current_order[__pos] as usize;
                                match __slot {
                                    #(#dispatch_arms)*
                                    _ => {}
                                }
                                #tap_capture_after_dispatch
                                __pos += 1;
                            }
                        }

                        #tap_capture_flush

                        let __gain: f32 = if self.__cf_dir != 0i8 {
                            let __g = self.__cf_pos as f32 / __CROSSFADE_SAMPLES as f32;
                            if self.__cf_dir > 0i8 {
                                self.__cf_pos += 1;
                                if self.__cf_pos >= __CROSSFADE_SAMPLES {
                                    self.__cf_pos = __CROSSFADE_SAMPLES;
                                    self.__cf_dir = 0i8;
                                }
                            } else if self.__cf_pos > 0 {
                                self.__cf_pos -= 1;
                            }
                            __g
                        } else {
                            1.0_f32
                        };

                        __left[__sample_idx] = __stereo[0] * __gain;
                        if let Some(__right_channel) = __right.as_mut() {
                            __right_channel[__sample_idx] = __stereo[1] * __gain;
                        }
                    }

                    #tap_observe_calls
                }
            }

            struct __DevProcessorInstance {
                params: ::std::sync::Arc<__WavecraftParams>,
                oscilloscope_consumer: ::std::sync::Mutex<::std::option::Option<#krate::OscilloscopeFrameConsumer>>,
                state: __DevProcessorState,
            }

            extern "C" fn create() -> *mut c_void {
                let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    let params = ::std::sync::Arc::new(__WavecraftParams::default());
                    let (state, oscilloscope_consumer) = __DevProcessorState::default_state();
                    let instance = ::std::boxed::Box::new(__DevProcessorInstance {
                        params,
                        oscilloscope_consumer: ::std::sync::Mutex::new(Some(oscilloscope_consumer)),
                        state,
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
                    if num_channels > 2 {
                        return;
                    }

                    instance.state.process_block(
                        instance.params.as_ref(),
                        channels,
                        num_channels,
                        num_samples,
                    );
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
                    let __copy_len = values.len().min(instance.state.__param_scratch.len());
                    instance.state.__param_scratch[..__copy_len]
                        .copy_from_slice(&values[..__copy_len]);
                }));
            }

            unsafe extern "C" fn set_signal_chain_order_json(
                instance: *mut c_void,
                json_ptr: *const ::std::ffi::c_char,
            ) -> bool {
                let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() || json_ptr.is_null() {
                        return false;
                    }

                    // SAFETY: `instance` is non-null (checked above), was created by
                    // `Box::into_raw` in `create`, and remains valid for this call.
                    let instance = unsafe { &*(instance as *const __DevProcessorInstance) };
                    // SAFETY: `json_ptr` is non-null (checked above) and must point to a
                    // valid NUL-terminated UTF-8 string for this call.
                    let json = unsafe { ::std::ffi::CStr::from_ptr(json_ptr) };
                    let Ok(json) = json.to_str() else {
                        return false;
                    };
                    let Ok(slots) = #krate::__internal::serde_json::from_str::<
                        ::std::vec::Vec<#krate::__nih::SignalChainSlot>,
                    >(json) else {
                        return false;
                    };

                    <__WavecraftParams as #krate::__nih::SignalChainOrderAccess>::set_order(
                        instance.params.as_ref(),
                        slots,
                    )
                    .is_ok()
                }));

                result.unwrap_or(false)
            }

            extern "C" fn take_latest_oscilloscope_frame_json(
                instance: *mut c_void,
            ) -> *mut ::std::ffi::c_char {
                let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() {
                        return None;
                    }

                    // SAFETY: `instance` is non-null (checked above), was created by
                    // `Box::into_raw` in `create`, and remains valid for this call.
                    let instance = unsafe { &*(instance as *const __DevProcessorInstance) };
                    let mut oscilloscope_consumer = instance
                        .oscilloscope_consumer
                        .lock()
                        .expect("oscilloscope_consumer mutex poisoned");
                    oscilloscope_consumer
                        .as_mut()
                        .and_then(|consumer| consumer.read_latest())
                        .map(|frame| frame.to_protocol_frame())
                }));

                let json = match result {
                    Ok(frame) => #krate::__internal::serde_json::to_string(&frame)
                        .unwrap_or_else(|_| "null".to_string()),
                    Err(_) => "null".to_string(),
                };

                ::std::ffi::CString::new(json)
                    .map(|s| s.into_raw())
                    .unwrap_or(::std::ptr::null_mut())
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
                    instance.state.initialize_for_dev_runtime(sample_rate);
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
                    instance.state.reset_for_dev_runtime();
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
                set_signal_chain_order_json,
                take_latest_oscilloscope_frame_json,
                set_sample_rate,
                reset,
                drop: drop_fn,
            }
        }
    }
}
