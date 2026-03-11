use proc_macro2::TokenStream;
use quote::quote;

/// Generate the complete `fn process(...)` method body for `impl Plugin for __WavecraftPlugin`.
///
/// The returned `TokenStream` is a single `fn process` item, ready to be embedded inside
/// `impl #krate::__nih::Plugin for __WavecraftPlugin { ... }`.
pub(super) fn build(
    krate: &syn::Path,
    s: &super::context::SharedSymbols,
    sc: &super::signal_chain::SignalChainTokens,
) -> TokenStream {
    let proc_idx_usize = &s.proc_idx_usize;

    let super::signal_chain::SignalChainTokens {
        tap_boundary_handoff,
        tap_capture_vars,
        tap_capture_boundary_zero,
        dispatch_arms,
        tap_capture_after_dispatch,
        tap_capture_flush,
        tap_observe_calls,
        ..
    } = sc;

    let osc_block_capture = quote! {
        let __osc_left_len = buffer
            .as_slice()
            .get(0)
            .map(|c| c.len().min(__num_samples))
            .unwrap_or(0usize);
        let __osc_right_len = buffer
            .as_slice()
            .get(1)
            .map(|c| c.len().min(__num_samples))
            .unwrap_or(0usize);
        let __osc_left_ptr = buffer
            .as_slice()
            .get(0)
            .map(|c| c.as_ptr())
            .unwrap_or(::std::ptr::null());
        let __osc_right_ptr = buffer
            .as_slice()
            .get(1)
            .map(|c| c.as_ptr())
            .unwrap_or(::std::ptr::null());
        let __osc_left: &[f32] = if __osc_left_ptr.is_null() {
            &[]
        } else {
            unsafe { ::std::slice::from_raw_parts(__osc_left_ptr, __osc_left_len) }
        };
        let __osc_right: &[f32] = if __osc_right_ptr.is_null() {
            &[]
        } else {
            unsafe { ::std::slice::from_raw_parts(__osc_right_ptr, __osc_right_len) }
        };
        self.oscilloscope_tap.capture_stereo(__osc_left, __osc_right);
    };

    quote! {
        fn process(
            &mut self,
            buffer: &mut #krate::__nih::Buffer,
            _aux: &mut #krate::__nih::AuxiliaryBuffers,
            _context: &mut impl #krate::__nih::ProcessContext<Self>,
        ) -> #krate::__nih::ProcessStatus {
            // ── Phase 2/4: Block-start order handoff ─────────────────────────────
            if self.__cf_dir == -1i8 && self.__cf_pos == 0 {
                self.params.__has_pending_order
                    .store(false, ::std::sync::atomic::Ordering::Release);
                #(
                    self.__current_order[#proc_idx_usize] = self
                        .params
                        .__pending_slots[#proc_idx_usize]
                        .load(::std::sync::atomic::Ordering::Acquire);
                )*
                #tap_boundary_handoff
                self.__cf_dir = 1i8;
            }
            if self.__cf_dir == 0i8
                && self.params.__has_pending_order
                    .load(::std::sync::atomic::Ordering::Acquire)
            {
                self.__cf_pos = __CROSSFADE_SAMPLES;
                self.__cf_dir = -1i8;
            }

            let __num_samples = buffer.samples();
            let __channels = buffer.channels();

            // Pre-collect all current parameter plain-values into pre-allocated scratch buffer.
            self.__param_scratch.clear();
            for __p in self.params.params.iter() {
                self.__param_scratch.push(__p.modulated_plain_value());
            }

            let __current_order = self.__current_order;
            let __tap_boundaries = self.__tap_boundaries;

            // SAFETY: __self_ptr is derived from the exclusive &mut self receiver. It is
            // only used within this call and never aliased with another mutable reference.
            let __self_ptr: *mut Self = self;

            for __sample_idx in 0..__num_samples {
                let __transport = #krate::Transport::default();
                let mut __stereo: [f32; 2] = [
                    buffer.as_slice()[0][__sample_idx],
                    if __channels >= 2 {
                        buffer.as_slice()[1][__sample_idx]
                    } else {
                        0.0_f32
                    },
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

                unsafe {
                    let __ptr = buffer.as_slice()[0].as_ptr() as *mut f32;
                    *__ptr.add(__sample_idx) = __stereo[0] * __gain;
                }
                if __channels >= 2 {
                    unsafe {
                        let __ptr = buffer.as_slice()[1].as_ptr() as *mut f32;
                        *__ptr.add(__sample_idx) = __stereo[1] * __gain;
                    }
                }
            }

            let mut __peak_l = 0.0_f32;
            let mut __peak_r = 0.0_f32;
            if __channels >= 1 {
                __peak_l = buffer.as_slice()[0]
                    .iter()
                    .map(|&s| s.abs())
                    .fold(0.0_f32, f32::max);
            }
            if __channels >= 2 {
                __peak_r = buffer.as_slice()[1]
                    .iter()
                    .map(|&s| s.abs())
                    .fold(0.0_f32, f32::max);
            }

                    #tap_observe_calls
                    #osc_block_capture

            let _ = self.meter_producer.push(#krate::MeterFrame {
                peak_l: __peak_l,
                peak_r: __peak_r,
                rms_l: __peak_l * 0.707,
                rms_r: __peak_r * 0.707,
                timestamp: 0,
            });

            #krate::__nih::ProcessStatus::Normal
        }
    }
}
