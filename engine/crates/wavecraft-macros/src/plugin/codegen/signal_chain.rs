use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

pub(super) struct SignalChainTokens {
    pub(super) has_passthrough_processor: bool,
    pub(super) proc_struct_fields: Vec<TokenStream>,
    pub(super) proc_defaults: Vec<TokenStream>,
    pub(super) proc_validations: Vec<TokenStream>,
    pub(super) tap_validations: Vec<TokenStream>,
    pub(super) set_sample_rate_calls: Vec<TokenStream>,
    pub(super) reset_calls: Vec<TokenStream>,
    pub(super) dispatch_arms: Vec<TokenStream>,
    pub(super) param_count_exprs: Vec<TokenStream>,
    pub(super) passthrough_meter_fields: TokenStream,
    pub(super) passthrough_meter_defaults: TokenStream,
    pub(super) passthrough_meter_initialize: TokenStream,
    pub(super) passthrough_meter_reset: TokenStream,
    pub(super) passthrough_meter_push: TokenStream,
    pub(super) tap_struct_fields: TokenStream,
    pub(super) tap_defaults: TokenStream,
    pub(super) tap_initialize: TokenStream,
    pub(super) tap_reset: TokenStream,
    pub(super) tap_boundary_handoff: TokenStream,
    pub(super) tap_capture_vars: TokenStream,
    pub(super) tap_capture_boundary_zero: TokenStream,
    pub(super) tap_capture_after_dispatch: TokenStream,
    pub(super) tap_capture_flush: TokenStream,
    pub(super) tap_observe_calls: TokenStream,
    pub(super) initial_order_state_slots: Vec<TokenStream>,
}

pub(super) fn build(
    processors: &[Type],
    taps: &[Type],
    krate: &syn::Path,
    s: &super::context::SharedSymbols,
) -> SignalChainTokens {
    let proc_field_names = &s.proc_field_names;
    let tap_field_names = &s.tap_field_names;
    let tap_scratch_l_names = &s.tap_scratch_l_names;
    let tap_scratch_r_names = &s.tap_scratch_r_names;
    let tap_capt_l_names = &s.tap_capt_l_names;
    let tap_capt_r_names = &s.tap_capt_r_names;
    let tap_idx_usize = &s.tap_idx_usize;
    let proc_name_str_lits = &s.proc_name_str_lits;
    let tap_name_str_lits = &s.tap_name_str_lits;
    let default_tap_boundary_u8 = &s.default_tap_boundary_u8;
    let tap_boundary_slot_idx = &s.tap_boundary_slot_idx;
    let t_lit = &s.t_lit;
    let total_slots = s.total_slots;
    let has_passthrough_processor = processors
        .iter()
        .any(|ty| super::context::type_last_segment_name(ty) == "Passthrough");

    // ── Per-processor struct fields ───────────────────────────────────────────
    let proc_struct_fields: Vec<TokenStream> = processors
        .iter()
        .zip(proc_field_names.iter())
        .map(|(ty, fname)| {
            quote! { #fname: #krate::Bypassed<#ty>, }
        })
        .collect();

    // ── Per-processor defaults ────────────────────────────────────────────────
    let proc_defaults: Vec<TokenStream> = processors
        .iter()
        .zip(proc_field_names.iter())
        .map(|(ty, fname)| {
            quote! {
                #fname: <#krate::Bypassed<#ty> as ::std::default::Default>::default(),
            }
        })
        .collect();

    // ── Compile-time trait validations (processors) ───────────────────────────
    let proc_validations: Vec<TokenStream> = processors
        .iter()
        .map(|ty| {
            quote! { assert_processor_traits::<#krate::Bypassed<#ty>>(); }
        })
        .collect();

    // ── Compile-time trait validations (taps) ────────────────────────────────
    let tap_validations: Vec<TokenStream> = taps
        .iter()
        .map(|ty| {
            quote! { assert_tap_traits::<#ty>(); }
        })
        .collect();

    // ── set_sample_rate calls in initialize() ─────────────────────────────────
    let set_sample_rate_calls: Vec<TokenStream> = proc_field_names
        .iter()
        .map(|fname| {
            quote! {
                #krate::Processor::set_sample_rate(&mut self.#fname, _buffer_config.sample_rate);
            }
        })
        .collect();

    // ── reset() calls (processors) ───────────────────────────────────────────
    let reset_calls: Vec<TokenStream> = proc_field_names
        .iter()
        .map(|fname| {
            quote! { #krate::Processor::reset(&mut self.#fname); }
        })
        .collect();

    // ── Static dispatch arms for process() loop ───────────────────────────────
    //
    // NOTE: After Step 14, OscilloscopeTap is no longer in the processors list
    // (the compile-time guard prevents it). All processors use the normal dispatch arm.
    // Tap observation now happens via the new `__tap_N` system.
    let dispatch_arms: Vec<TokenStream> = processors
        .iter()
        .zip(proc_field_names.iter())
        .enumerate()
        .map(|(i, (ty, fname))| {
            let i_lit = syn::LitInt::new(&i.to_string(), proc_macro2::Span::call_site());
            let i_lit_usize = syn::LitInt::new(&format!("{}", i), proc_macro2::Span::call_site());
            let processor_name = super::context::type_last_segment_name(ty);
            let passthrough_meter_capture = if processor_name == "Passthrough" {
                quote! {
                    // SAFETY: `__self_ptr` comes from the exclusive `&mut self` receiver.
                    // The scratch buffers do not alias the currently processed channel slices.
                    unsafe {
                        let __ptr = (*__self_ptr).__passthrough_meter_scratch_l.as_mut_ptr();
                        *__ptr.add(__sample_idx) = __channel_slices[0][0];
                    }
                    unsafe {
                        let __ptr = (*__self_ptr).__passthrough_meter_scratch_r.as_mut_ptr();
                        *__ptr.add(__sample_idx) = __channel_slices[1][0];
                    }
                }
            } else {
                quote! {}
            };
            quote! {
                #i_lit => {
                    let __start = self.__param_offsets[#i_lit_usize];
                    let __end   = self.__param_offsets[#i_lit_usize + 1];
                    let mut __pp =
                        <<#krate::Bypassed<#ty> as #krate::Processor>::Params
                            as #krate::ProcessorParams>::from_param_defaults();
                    <<#krate::Bypassed<#ty> as #krate::Processor>::Params
                        as #krate::ProcessorParams>::apply_plain_values(
                        &mut __pp,
                        &self.__param_scratch[__start..__end],
                    );
                    use #krate::Processor as _;
                    unsafe {
                        (*__self_ptr).#fname.process(
                            &mut __channel_slices[..__active_ch],
                            &__transport,
                            &__pp,
                        );
                    }
                    #passthrough_meter_capture
                }
            }
        })
        .collect();

    // ── param_count exprs ─────────────────────────────────────────────────────
    let param_count_exprs: Vec<TokenStream> = processors
        .iter()
        .map(|ty| {
            quote! {
                <<#krate::Bypassed<#ty> as #krate::Processor>::Params
                    as #krate::ProcessorParams>::plain_value_count()
            }
        })
        .collect();

    let passthrough_meter_fields: TokenStream = if has_passthrough_processor {
        quote! {
            __passthrough_meter_scratch_l: ::std::vec::Vec<f32>,
            __passthrough_meter_scratch_r: ::std::vec::Vec<f32>,
            __passthrough_meter_timestamp: u64,
        }
    } else {
        quote! {}
    };

    let passthrough_meter_defaults: TokenStream = if has_passthrough_processor {
        quote! {
            __passthrough_meter_scratch_l: ::std::vec::Vec::new(),
            __passthrough_meter_scratch_r: ::std::vec::Vec::new(),
            __passthrough_meter_timestamp: 0,
        }
    } else {
        quote! {}
    };

    let passthrough_meter_initialize: TokenStream = if has_passthrough_processor {
        quote! {
            let __max_buf = _buffer_config.max_buffer_size as usize;
            self.__passthrough_meter_scratch_l.resize(__max_buf, 0.0_f32);
            self.__passthrough_meter_scratch_r.resize(__max_buf, 0.0_f32);
        }
    } else {
        quote! {}
    };

    let passthrough_meter_reset: TokenStream = if has_passthrough_processor {
        quote! {
            self.__passthrough_meter_scratch_l.fill(0.0_f32);
            self.__passthrough_meter_scratch_r.fill(0.0_f32);
            self.__passthrough_meter_timestamp = 0;
        }
    } else {
        quote! {}
    };

    let passthrough_meter_push: TokenStream = if has_passthrough_processor {
        quote! {
            if __num_samples > 0 {
                let mut __passthrough_peak_l = 0.0_f32;
                let mut __passthrough_peak_r = 0.0_f32;

                for &__sample in self.__passthrough_meter_scratch_l[..__num_samples].iter() {
                    __passthrough_peak_l = __passthrough_peak_l.max(__sample.abs());
                }
                for &__sample in self.__passthrough_meter_scratch_r[..__num_samples].iter() {
                    __passthrough_peak_r = __passthrough_peak_r.max(__sample.abs());
                }

                let _ = self.passthrough_meter_producer.push(#krate::MeterFrame {
                    peak_l: __passthrough_peak_l,
                    peak_r: __passthrough_peak_r,
                    rms_l: __passthrough_peak_l * 0.707,
                    rms_r: __passthrough_peak_r * 0.707,
                    timestamp: self.__passthrough_meter_timestamp,
                });

                self.__passthrough_meter_timestamp =
                    self.__passthrough_meter_timestamp.wrapping_add(1);
            }
        }
    } else {
        quote! {}
    };

    // ── Step 15: Tap struct fields + scratch buffers ──────────────────────────
    let tap_struct_fields: TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let fields: Vec<TokenStream> = taps
            .iter()
            .zip(tap_field_names.iter())
            .zip(tap_scratch_l_names.iter())
            .zip(tap_scratch_r_names.iter())
            .map(|(((ty, fname), sl), sr)| {
                quote! {
                    #fname: #ty,
                    #sl: ::std::vec::Vec<f32>,
                    #sr: ::std::vec::Vec<f32>,
                }
            })
            .collect();
        let boundary_field = quote! {
            /// Per-tap insertion boundary (count of processors before tap in runtime slot order).
            /// Loaded from __pending_slots[N..N+T] at crossfade handoff; no allocation on RT path.
            __tap_boundaries: [u8; #t_lit],
        };
        quote! {
            #(#fields)*
            #boundary_field
        }
    };

    // ── Tap defaults ──────────────────────────────────────────────────────────
    let tap_defaults: TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let defaults: Vec<TokenStream> = taps
            .iter()
            .zip(tap_field_names.iter())
            .zip(tap_scratch_l_names.iter())
            .zip(tap_scratch_r_names.iter())
            .map(|(((ty, fname), sl), sr)| {
                let tap_name = super::context::type_last_segment_name(ty);
                let tap_init = if tap_name == "OscilloscopeTap" {
                    quote! { #ty::with_output(__oscilloscope_producer) }
                } else {
                    quote! { <#ty as ::std::default::Default>::default() }
                };
                quote! {
                    #fname: #tap_init,
                    #sl: ::std::vec::Vec::new(),
                    #sr: ::std::vec::Vec::new(),
                }
            })
            .collect();
        // All taps start with boundary = N (after all processors = final output)
        quote! {
            #(#defaults)*
            __tap_boundaries: [#(#default_tap_boundary_u8),*],
        }
    };

    // ── Tap initialize() body (set_sample_rate + scratch resize) ─────────────
    let tap_initialize: TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let calls: Vec<TokenStream> = taps
            .iter()
            .zip(tap_field_names.iter())
            .zip(tap_scratch_l_names.iter())
            .zip(tap_scratch_r_names.iter())
            .map(|(((_, fname), sl), sr)| {
                quote! {
                    #krate::TapProcessor::set_sample_rate(
                        &mut self.#fname,
                        _buffer_config.sample_rate,
                    );
                    let __max_buf = _buffer_config.max_buffer_size as usize;
                    self.#sl.resize(__max_buf, 0.0_f32);
                    self.#sr.resize(__max_buf, 0.0_f32);
                }
            })
            .collect();
        quote! { #(#calls)* }
    };

    // ── Tap reset() body ──────────────────────────────────────────────────────
    let tap_reset: TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let calls: Vec<TokenStream> = tap_field_names
            .iter()
            .map(|fname| {
                quote! { #krate::TapProcessor::reset(&mut self.#fname); }
            })
            .collect();
        quote! { #(#calls)* }
    };

    // ── Step 16: Crossfade handoff — also load tap boundaries ────────────────
    let tap_boundary_handoff: TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let loads: Vec<TokenStream> = tap_idx_usize
            .iter()
            .zip(tap_boundary_slot_idx.iter())
            .map(|(ti, si)| {
                quote! {
                    self.__tap_boundaries[#ti] = self
                        .params
                        .__pending_slots[#si]
                        .load(::std::sync::atomic::Ordering::Acquire);
                }
            })
            .collect();
        quote! { #(#loads)* }
    };

    // ── Step 17: Per-sample tap capture (boundary = 0) before dispatch loop ──
    //
    // DESIGN: To avoid E0499 (cross-iteration mutable borrow of self.__tap_*_scratch_*),
    // we do NOT write to self.*scratch* here. Instead we write to local capture variables
    // (__tap_N_capt_l / __tap_N_capt_r) that are declared at the start of each outer
    // for-loop iteration. A single flush (tap_capture_flush) after the { } block then
    // writes those locals to self.*scratch* exactly once per iteration.
    let tap_capture_boundary_zero: TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let captures: Vec<TokenStream> = tap_idx_usize
            .iter()
            .zip(tap_capt_l_names.iter())
            .zip(tap_capt_r_names.iter())
            .map(|((ti, cl), cr)| {
                quote! {
                    if __tap_boundaries[#ti] == 0u8 {
                        #cl = __stereo[0];
                        #cr = __stereo[1];
                    }
                }
            })
            .collect();
        quote! { #(#captures)* }
    };

    // ── Per-sample tap capture after each dispatch position ───────────────────
    //
    // DESIGN: Writes go to local capture variables (not self.*scratch*) to avoid
    // the cross-iteration E0499 borrow. The { } block (split_at_mut scope) holds
    // &mut [f32] refs via __channel_slices; writing to self.*scratch* while those
    // refs are alive across multiple while-loop iterations caused the conflict.
    // Instead we capture into __tap_N_capt_l / __tap_N_capt_r (plain f32 locals),
    // then flush to self.*scratch* in tap_capture_flush after the { } block ends.
    let tap_capture_after_dispatch: TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let captures: Vec<TokenStream> = tap_idx_usize
            .iter()
            .zip(tap_capt_l_names.iter())
            .zip(tap_capt_r_names.iter())
            .map(|((ti, cl), cr)| {
                quote! {
                    if __tap_boundaries[#ti] == __exec_count {
                        // Read through __channel_slices (not __stereo which is mutably
                        // borrowed via split_at_mut for the duration of this block).
                        // __channel_slices[0][0] / [1][0] are plain f32 copies — no
                        // persistent borrow of self or __channel_slices after these lines.
                        #cl = __channel_slices[0][0];
                        #cr = __channel_slices[1][0];
                    }
                }
            })
            .collect();
        // __exec_count is emitted in the dispatch loop; we add it conditionally
        quote! {
            let __exec_count = (__pos + 1) as u8;
            #(#captures)*
        }
    };

    // ── Flush per-iteration capture locals to self.*scratch* (after { } block) ─
    //
    // This is the single write point to self.__tap_*_scratch_*[__sample_idx] per
    // outer for-loop iteration. Consolidating all writes here avoids the E0499
    // borrow checker error that arose from writing to self.*scratch* in two places
    // (tap_capture_boundary_zero before the { } block AND tap_capture_after_dispatch
    // inside the while loop within the { } block).
    let tap_capture_flush: TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let flushes: Vec<TokenStream> = tap_capt_l_names
            .iter()
            .zip(tap_capt_r_names.iter())
            .zip(tap_scratch_l_names.iter())
            .zip(tap_scratch_r_names.iter())
            .map(|(((cl, cr), sl), sr)| {
                quote! {
                    unsafe {
                        let __ptr = (*__self_ptr).#sl.as_mut_ptr();
                        *__ptr.add(__sample_idx) = #cl;
                    }
                    unsafe {
                        let __ptr = (*__self_ptr).#sr.as_mut_ptr();
                        *__ptr.add(__sample_idx) = #cr;
                    }
                }
            })
            .collect();
        quote! { #(#flushes)* }
    };

    // ── Declare per-tap capture locals at the top of each for-loop iteration ──
    //
    // Default to 0.0. Will be overwritten by tap_capture_boundary_zero (boundary=0)
    // or tap_capture_after_dispatch (boundary=1..N). tap_capture_flush then writes
    // the final values to self.*scratch* once per outer loop iteration.
    let tap_capture_vars: TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let vars: Vec<TokenStream> = tap_capt_l_names
            .iter()
            .zip(tap_capt_r_names.iter())
            .map(|(cl, cr)| {
                quote! {
                    let mut #cl: f32 = 0.0_f32;
                    let mut #cr: f32 = 0.0_f32;
                }
            })
            .collect();
        quote! { #(#vars)* }
    };

    // ── Block-end observe_stereo calls (one per tap, after per-sample loop) ───
    let tap_observe_calls: TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let calls: Vec<TokenStream> = tap_field_names
            .iter()
            .zip(tap_scratch_l_names.iter())
            .zip(tap_scratch_r_names.iter())
            .map(|((fname, sl), sr)| {
                quote! {
                    #krate::TapProcessor::observe_stereo(
                        &mut self.#fname,
                        &self.#sl[..__num_samples],
                        &self.#sr[..__num_samples],
                    );
                }
            })
            .collect();
        quote! { #(#calls)* }
    };

    // ── Step 16: Default SignalChainOrder (identity: processors first, taps at end) ──
    //
    // Initial __order_state: all processors in declaration order, then all taps.
    // Initial __pending_slots: first N = processor identity order, next T = boundary N (after all procs).
    let initial_order_state_slots: Vec<TokenStream> = {
        let mut slots = Vec::with_capacity(total_slots);
        for pname in proc_name_str_lits.iter() {
            slots.push(quote! {
                #krate::__nih::SignalChainSlot {
                    id: #pname.to_string(),
                    slot_type: #krate::__nih::SlotType::Processor,
                }
            });
        }
        for tname in tap_name_str_lits.iter() {
            slots.push(quote! {
                #krate::__nih::SignalChainSlot {
                    id: #tname.to_string(),
                    slot_type: #krate::__nih::SlotType::Tap,
                }
            });
        }
        slots
    };

    SignalChainTokens {
        has_passthrough_processor,
        proc_struct_fields,
        proc_defaults,
        proc_validations,
        tap_validations,
        set_sample_rate_calls,
        reset_calls,
        dispatch_arms,
        param_count_exprs,
        passthrough_meter_fields,
        passthrough_meter_defaults,
        passthrough_meter_initialize,
        passthrough_meter_reset,
        passthrough_meter_push,
        tap_struct_fields,
        tap_defaults,
        tap_initialize,
        tap_reset,
        tap_boundary_handoff,
        tap_capture_vars,
        tap_capture_boundary_zero,
        tap_capture_after_dispatch,
        tap_capture_flush,
        tap_observe_calls,
        initial_order_state_slots,
    }
}
