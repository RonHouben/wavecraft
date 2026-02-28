use quote::quote;
use syn::Type;

/// Returns the last path segment identifier of a type as a PascalCase string.
///
/// Used as slot ID (e.g., "TestTone", "OscilloscopeTap") in runtime order.
fn type_last_segment_name(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_else(|| quote!(#ty).to_string()),
        _ => quote!(#ty).to_string(),
    }
}

pub(super) struct CodegenInput<'a> {
    pub(super) name: &'a syn::LitStr,
    pub(super) processors: &'a [Type],
    /// Tap processor types declared in `taps: [...]`. Empty when none declared.
    pub(super) taps: &'a [Type],
    pub(super) krate: &'a syn::Path,
    pub(super) runtime_param_blocks: &'a [proc_macro2::TokenStream],
    pub(super) processor_param_mappings: &'a [proc_macro2::TokenStream],
    pub(super) processor_info_entries: &'a [proc_macro2::TokenStream],
    pub(super) vendor: &'a str,
    pub(super) url: &'a str,
    pub(super) vst3_id: &'a proc_macro2::TokenStream,
    pub(super) clap_id: &'a str,
}

pub(super) fn generate_plugin_code(
    input: CodegenInput<'_>,
) -> syn::Result<proc_macro2::TokenStream> {
    let CodegenInput {
        name,
        processors,
        taps,
        krate,
        runtime_param_blocks,
        processor_param_mappings,
        processor_info_entries,
        vendor,
        url,
        vst3_id,
        clap_id,
    } = input;

    let n = processors.len();
    let t = taps.len();
    let total_slots = n + t;

    // ── Step 14: Compile-time overlap check (type appears in both processors and taps) ──
    //
    // A type cannot implement both Processor and TapProcessor. Detect this at macro
    // expansion time (before code generation) for a clear, actionable error message.
    {
        let proc_names: Vec<String> = processors.iter().map(type_last_segment_name).collect();
        for tap_ty in taps.iter() {
            let tap_name = type_last_segment_name(tap_ty);
            if proc_names.contains(&tap_name) {
                return Err(syn::Error::new_spanned(
                    tap_ty,
                    format!(
                        "type `{}` appears in both `processors` and `taps`\n\
                         \n\
                         A type cannot be both a `Processor` and a `TapProcessor`. Move \
                         `{}` to only one of the two lists.",
                        tap_name, tap_name
                    ),
                ));
            }
        }
    }

    let n_lit = syn::LitInt::new(&n.to_string(), proc_macro2::Span::call_site());
    let t_lit = syn::LitInt::new(&t.to_string(), proc_macro2::Span::call_site());
    let total_slots_lit =
        syn::LitInt::new(&total_slots.to_string(), proc_macro2::Span::call_site());
    let np1_lit = syn::LitInt::new(&(n + 1).to_string(), proc_macro2::Span::call_site());

    // ── Processor field names: __proc_0, __proc_1, ... ───────────────────────
    let proc_field_names: Vec<syn::Ident> = (0..n)
        .map(|i| syn::Ident::new(&format!("__proc_{}", i), proc_macro2::Span::call_site()))
        .collect();

    // ── u8 / usize index literals ─────────────────────────────────────────────
    let proc_idx_u8: Vec<proc_macro2::TokenStream> = (0..n)
        .map(|i| {
            let lit = syn::LitInt::new(&format!("{}u8", i), proc_macro2::Span::call_site());
            quote! { #lit }
        })
        .collect();

    let proc_idx_usize: Vec<proc_macro2::TokenStream> = (0..n)
        .map(|i| {
            let lit = syn::LitInt::new(&format!("{}usize", i), proc_macro2::Span::call_site());
            quote! { #lit }
        })
        .collect();

    // ── Processor type names (PascalCase strings for slot IDs) ───────────────
    let proc_type_names: Vec<String> = processors.iter().map(type_last_segment_name).collect();
    let proc_name_str_lits: Vec<syn::LitStr> = proc_type_names
        .iter()
        .map(|s| syn::LitStr::new(s, proc_macro2::Span::call_site()))
        .collect();

    // ── Tap field names and type names ────────────────────────────────────────
    let tap_field_names: Vec<syn::Ident> = (0..t)
        .map(|i| syn::Ident::new(&format!("__tap_{}", i), proc_macro2::Span::call_site()))
        .collect();
    let tap_scratch_l_names: Vec<syn::Ident> = (0..t)
        .map(|i| {
            syn::Ident::new(
                &format!("__tap_{}_scratch_l", i),
                proc_macro2::Span::call_site(),
            )
        })
        .collect();
    let tap_scratch_r_names: Vec<syn::Ident> = (0..t)
        .map(|i| {
            syn::Ident::new(
                &format!("__tap_{}_scratch_r", i),
                proc_macro2::Span::call_site(),
            )
        })
        .collect();
    let tap_idx_usize: Vec<proc_macro2::TokenStream> = (0..t)
        .map(|i| {
            let lit = syn::LitInt::new(&format!("{}usize", i), proc_macro2::Span::call_site());
            quote! { #lit }
        })
        .collect();
    // Per-tap local capture variable names: __tap_0_capt_l, __tap_0_capt_r, ...
    // Used to defer writes to self.__tap_*_scratch_* to a single flush point per outer
    // for-loop iteration, avoiding cross-iteration E0499 borrow checker errors.
    let tap_capt_l_names: Vec<syn::Ident> = (0..t)
        .map(|i| {
            syn::Ident::new(
                &format!("__tap_{}_capt_l", i),
                proc_macro2::Span::call_site(),
            )
        })
        .collect();
    let tap_capt_r_names: Vec<syn::Ident> = (0..t)
        .map(|i| {
            syn::Ident::new(
                &format!("__tap_{}_capt_r", i),
                proc_macro2::Span::call_site(),
            )
        })
        .collect();
    let tap_type_names: Vec<String> = taps.iter().map(type_last_segment_name).collect();
    let tap_name_str_lits: Vec<syn::LitStr> = tap_type_names
        .iter()
        .map(|s| syn::LitStr::new(s, proc_macro2::Span::call_site()))
        .collect();

    // Default boundary = N (all taps start after all processors = final output)
    let default_tap_boundary_u8: Vec<proc_macro2::TokenStream> = (0..t)
        .map(|_| {
            let lit = syn::LitInt::new(&format!("{}u8", n), proc_macro2::Span::call_site());
            quote! { #lit }
        })
        .collect();
    // Indices for pending_slots[N + tap_idx]
    let tap_boundary_slot_idx: Vec<proc_macro2::TokenStream> = (0..t)
        .map(|i| {
            let idx = n + i;
            let lit = syn::LitInt::new(&format!("{}usize", idx), proc_macro2::Span::call_site());
            quote! { #lit }
        })
        .collect();

    // ── Per-processor struct fields ───────────────────────────────────────────
    let proc_struct_fields: Vec<proc_macro2::TokenStream> = processors
        .iter()
        .zip(proc_field_names.iter())
        .map(|(ty, fname)| {
            quote! { #fname: #krate::Bypassed<#ty>, }
        })
        .collect();

    // ── Per-processor defaults ────────────────────────────────────────────────
    let proc_defaults: Vec<proc_macro2::TokenStream> = processors
        .iter()
        .zip(proc_field_names.iter())
        .map(|(ty, fname)| {
            quote! {
                #fname: <#krate::Bypassed<#ty> as ::std::default::Default>::default(),
            }
        })
        .collect();

    // ── Compile-time trait validations (processors) ───────────────────────────
    let proc_validations: Vec<proc_macro2::TokenStream> = processors
        .iter()
        .map(|ty| {
            quote! { assert_processor_traits::<#krate::Bypassed<#ty>>(); }
        })
        .collect();

    // ── Compile-time trait validations (taps) ────────────────────────────────
    let tap_validations: Vec<proc_macro2::TokenStream> = taps
        .iter()
        .map(|ty| {
            quote! { assert_tap_traits::<#ty>(); }
        })
        .collect();

    // ── set_sample_rate calls in initialize() ─────────────────────────────────
    let set_sample_rate_calls: Vec<proc_macro2::TokenStream> = proc_field_names
        .iter()
        .map(|fname| {
            quote! {
                #krate::Processor::set_sample_rate(&mut self.#fname, _buffer_config.sample_rate);
            }
        })
        .collect();

    // ── reset() calls (processors) ───────────────────────────────────────────
    let reset_calls: Vec<proc_macro2::TokenStream> = proc_field_names
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
    let dispatch_arms: Vec<proc_macro2::TokenStream> = processors
        .iter()
        .zip(proc_field_names.iter())
        .enumerate()
        .map(|(i, (ty, fname))| {
            let i_lit = syn::LitInt::new(&i.to_string(), proc_macro2::Span::call_site());
            let i_lit_usize = syn::LitInt::new(&format!("{}", i), proc_macro2::Span::call_site());
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
                        // Use a temporary slice from self.__param_scratch rather than a
                        // long-lived &[f32] binding to avoid E0499 cross-iteration borrow.
                        &self.__param_scratch[__start..__end],
                    );
                    use #krate::Processor as _;
                    // SAFETY: __self_ptr is a *mut Self created outside the for loop from
                    // the exclusive &mut self receiver.  All processor fields are distinct
                    // (each arm touches a different one), no aliasing occurs, and the call
                    // is entirely single-threaded.  Routing through *__self_ptr prevents NLL
                    // from tracking the mutable access as a loan of `self`, eliminating the
                    // cross-iteration E0499 false-positive that arises with `self.field`.
                    unsafe { (*__self_ptr).#fname.process(&mut __channel_slices[..__active_ch], &__transport, &__pp); }
                }
            }
        })
        .collect();

    // ── param_count exprs ─────────────────────────────────────────────────────
    let param_count_exprs: Vec<proc_macro2::TokenStream> = processors
        .iter()
        .map(|ty| {
            quote! {
                <<#krate::Bypassed<#ty> as #krate::Processor>::Params
                    as #krate::ProcessorParams>::plain_value_count()
            }
        })
        .collect();

    // ── Step 15: Tap struct fields + scratch buffers ──────────────────────────
    let tap_struct_fields: proc_macro2::TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let fields: Vec<proc_macro2::TokenStream> = taps
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
    let tap_defaults: proc_macro2::TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let defaults: Vec<proc_macro2::TokenStream> = taps
            .iter()
            .zip(tap_field_names.iter())
            .zip(tap_scratch_l_names.iter())
            .zip(tap_scratch_r_names.iter())
            .map(|(((ty, fname), sl), sr)| {
                quote! {
                    #fname: <#ty as ::std::default::Default>::default(),
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
    let tap_initialize: proc_macro2::TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let calls: Vec<proc_macro2::TokenStream> = taps
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
    let tap_reset: proc_macro2::TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let calls: Vec<proc_macro2::TokenStream> = tap_field_names
            .iter()
            .map(|fname| {
                quote! { #krate::TapProcessor::reset(&mut self.#fname); }
            })
            .collect();
        quote! { #(#calls)* }
    };

    // ── Step 16: Crossfade handoff — also load tap boundaries ────────────────
    let tap_boundary_handoff: proc_macro2::TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let loads: Vec<proc_macro2::TokenStream> = tap_idx_usize
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
    let tap_capture_boundary_zero: proc_macro2::TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let captures: Vec<proc_macro2::TokenStream> = tap_idx_usize
            .iter()
            .zip(tap_capt_l_names.iter())
            .zip(tap_capt_r_names.iter())
            .map(|((ti, cl), cr)| {
                quote! {
                    if self.__tap_boundaries[#ti] == 0u8 {
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
    let tap_capture_after_dispatch: proc_macro2::TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let captures: Vec<proc_macro2::TokenStream> = tap_idx_usize
            .iter()
            .zip(tap_capt_l_names.iter())
            .zip(tap_capt_r_names.iter())
            .map(|((ti, cl), cr)| {
                quote! {
                    if self.__tap_boundaries[#ti] == __exec_count {
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
    let tap_capture_flush: proc_macro2::TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let flushes: Vec<proc_macro2::TokenStream> = tap_capt_l_names
            .iter()
            .zip(tap_capt_r_names.iter())
            .zip(tap_scratch_l_names.iter())
            .zip(tap_scratch_r_names.iter())
            .map(|(((cl, cr), sl), sr)| {
                quote! {
                    self.#sl[__sample_idx] = #cl;
                    self.#sr[__sample_idx] = #cr;
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
    let tap_capture_vars: proc_macro2::TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let vars: Vec<proc_macro2::TokenStream> = tap_capt_l_names
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
    let tap_observe_calls: proc_macro2::TokenStream = if taps.is_empty() {
        quote! {}
    } else {
        let calls: Vec<proc_macro2::TokenStream> = tap_field_names
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

    // ── Step 16 (updated): osc_block_capture — always final-output, no heap alloc ──
    //
    // The OscilloscopeTap position-detection (osc_tap_slot / has_osc_in_chain) is
    // removed. The hardcoded `oscilloscope_tap` field always captures final output.
    // Use get() + map() chaining to avoid creating long-lived slice references that
    // confuse the borrow checker in older NLL (stable rustc 1.93+).
    let osc_block_capture = quote! {
        self.oscilloscope_tap.capture_stereo(
            buffer.as_slice().get(0).map(|c| &c[..__num_samples]).unwrap_or(&[]),
            buffer.as_slice().get(1).map(|c| &c[..__num_samples]).unwrap_or(&[]),
        );
    };

    // ── Step 16: Default SignalChainOrder (identity: processors first, taps at end) ──
    //
    // Initial __order_state: all processors in declaration order, then all taps.
    // Initial __pending_slots: first N = processor identity order, next T = boundary N (after all procs).
    let initial_order_state_slots: Vec<proc_macro2::TokenStream> = {
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

    let expanded = quote! {
        // Keep SignalChain type alias for dev-FFI vtable (uses static dispatch in registration order)
        type __ProcessorType = #krate::SignalChain![#(#processors),*];

        // Compile-time validation: every processor type must satisfy required trait bounds,
        // and every tap type must satisfy TapProcessor trait bounds.
        const _: () = {
            fn assert_processor_traits<T>()
            where
                T: #krate::Processor + ::std::default::Default + ::std::marker::Send + 'static,
                T::Params: #krate::ProcessorParams
                    + ::std::default::Default
                    + ::std::marker::Send
                    + ::std::marker::Sync
                    + 'static,
            {
            }
            fn assert_tap_traits<T>()
            where
                T: #krate::TapProcessor + ::std::default::Default + ::std::marker::Send + 'static,
            {
            }
            fn validate() {
                #(#proc_validations)*
                #(#tap_validations)*
            }
        };

        /// Number of processors in the signal chain (compile-time constant).
        const __PROC_COUNT: usize = #n_lit;

        /// Number of samples to crossfade over when reordering processors.
        const __CROSSFADE_SAMPLES: usize = 256;

        /// Generated plugin struct.
        pub struct __WavecraftPlugin {
            params: ::std::sync::Arc<__WavecraftParams>,
            // --- Per-processor fields (Phase 2: runtime reordering) ---
            #(#proc_struct_fields)*
            // Audio-thread-local processing order (index array into per-proc fields)
            __current_order: [u8; #n_lit],
            // Cached cumulative parameter offsets for each processor
            __param_offsets: [usize; #np1_lit],
            // --- Crossfade state (Phase 4) ---
            __cf_pos: usize,
            __cf_dir: i8, // -1 = fade-down, 0 = normal, 1 = fade-up
            // Pre-allocated parameter scratch buffer (reused each block, zero allocation after init)
            __param_scratch: ::std::vec::Vec<f32>,
            // --- Tap processors (Step 15: __tap_N fields + scratch buffers + boundaries) ---
            // Generated for each declared taps entry. Scratch buffers are pre-allocated in
            // initialize() and reused each block — zero RT allocation after init.
            #tap_struct_fields
            // --- Metering ---
            oscilloscope_tap: #krate::OscilloscopeTap,
            meter_producer: #krate::MeterProducer,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            meter_consumer: ::std::sync::Mutex<::std::option::Option<#krate::MeterConsumer>>,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            oscilloscope_consumer:
                ::std::sync::Mutex<::std::option::Option<#krate::OscilloscopeFrameConsumer>>,
        }

        /// Generated params struct.
        ///
        /// Parameters are discovered at runtime from processor param_specs().
        enum __WavecraftRuntimeParam {
            Float(#krate::__nih::FloatParam),
            Int(#krate::__nih::IntParam),
        }

        impl __WavecraftRuntimeParam {
            fn as_ptr(&self) -> #krate::__nih::ParamPtr {
                use #krate::__nih::Param;
                match self {
                    Self::Float(param) => param.as_ptr(),
                    Self::Int(param) => param.as_ptr(),
                }
            }

            fn modulated_plain_value(&self) -> f32 {
                let ptr = self.as_ptr();
                // SAFETY: ParamPtr originates from `self` and remains valid for this call.
                unsafe { ptr.modulated_plain_value() }
            }
        }

        pub struct __WavecraftParams {
            // Runtime parameter values (flat, all processors in chain order)
            params: ::std::vec::Vec<__WavecraftRuntimeParam>,
            // Runtime IDs aligned with FFI-generated contract IDs (e.g. test_tone_frequency)
            ids: ::std::vec::Vec<::std::string::String>,
            // Optional parameter group names (empty string when none)
            groups: ::std::vec::Vec<::std::string::String>,
            // --- Lock-free SPSC order handoff (shared between UI and audio threads) ---
            // First N slots: processor execution order (u8 index into proc fields).
            // Next T slots: tap boundary values (count of procs before each tap).
            __has_pending_order: ::std::sync::atomic::AtomicBool,
            __pending_slots: ::std::vec::Vec<::std::sync::atomic::AtomicU8>,
            // --- Order state for IPC reads and persistence ---
            // Full unified SignalChainOrder (processors + taps in slot order).
            __order_state: ::std::sync::Mutex<::std::vec::Vec<#krate::__nih::SignalChainSlot>>,
        }

        impl __WavecraftParams {
            fn from_processor_specs() -> Self {
                use #krate::ParamRange;

                let mut params = ::std::vec::Vec::new();
                let mut ids = ::std::vec::Vec::new();
                let mut groups = ::std::vec::Vec::new();

                #(#runtime_param_blocks)*

                // Initialise pending-order slots:
                //   slots[0..N]   = identity processor order (0u8, 1u8, ...)
                //   slots[N..N+T] = default tap boundaries = N (taps after all procs)
                let mut __pending: ::std::vec::Vec<::std::sync::atomic::AtomicU8> =
                    ::std::vec::Vec::with_capacity(#total_slots_lit);
                #(
                    __pending.push(::std::sync::atomic::AtomicU8::new(#proc_idx_u8));
                )*
                // Tap boundary defaults: N = after all processors
                #(
                    __pending.push(::std::sync::atomic::AtomicU8::new(#default_tap_boundary_u8));
                )*

                // Initial order: all processors in declaration order, then all taps.
                let __initial_order: ::std::vec::Vec<#krate::__nih::SignalChainSlot> =
                    ::std::vec![#(#initial_order_state_slots),*];

                Self {
                    params,
                    ids,
                    groups,
                    __has_pending_order: ::std::sync::atomic::AtomicBool::new(false),
                    __pending_slots: __pending,
                    __order_state: ::std::sync::Mutex::new(__initial_order),
                }
            }
        }

        impl ::std::default::Default for __WavecraftParams {
            fn default() -> Self {
                Self::from_processor_specs()
            }
        }

        // Manual Params implementation (can't use derive due to Vec)
        unsafe impl #krate::__nih::Params for __WavecraftParams {
            fn param_map(&self) -> ::std::vec::Vec<(
                ::std::string::String,
                #krate::__nih::ParamPtr,
                ::std::string::String,
            )> {
                self.params
                    .iter()
                    .zip(self.ids.iter())
                    .zip(self.groups.iter())
                    .map(|((param, id), group)| (id.clone(), param.as_ptr(), group.clone()))
                    .collect()
            }

            fn serialize_fields(
                &self,
            ) -> ::std::collections::BTreeMap<::std::string::String, ::std::string::String>
            {
                // Serialize the unified SignalChainOrder as JSON.
                // Key: "signalChainOrder".
                let __order_guard = self
                    .__order_state
                    .lock()
                    .unwrap_or_else(|__e| __e.into_inner());

                // Serialize as [{"id":"TestTone","type":"processor"}, ...]
                let __json = #krate::__internal::serde_json::to_string(&*__order_guard)
                    .unwrap_or_default();

                let mut __map =
                    ::std::collections::BTreeMap::<::std::string::String, ::std::string::String>::new();
                __map.insert("signalChainOrder".to_string(), __json);
                __map
            }

            fn deserialize_fields(
                &self,
                fields: &::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
            ) {
                if let Some(__json) = fields.get("signalChainOrder") {
                    if let Ok(__slots) =
                        #krate::__internal::serde_json::from_str::<
                            ::std::vec::Vec<#krate::__nih::SignalChainSlot>,
                        >(__json)
                    {
                        // Re-use set_order for validation + apply.
                        if let ::std::result::Result::Ok(()) =
                        <Self as #krate::__nih::SignalChainOrderAccess>::set_order(self, __slots) {
                            // Already applied by set_order.
                        } else {
                            eprintln!(
                                "[wavecraft] signalChainOrder restore failed: persisted order \
                                 is invalid. Falling back to default declaration order."
                            );
                        }
                    }
                }
            }
        }

        impl #krate::__nih::SignalChainOrderAccess for __WavecraftParams {
            fn get_order(&self) -> ::std::vec::Vec<#krate::__nih::SignalChainSlot> {
                // Return a clone of the current unified order (processors + taps by type name).
                self.__order_state
                    .lock()
                    .unwrap_or_else(|__e| __e.into_inner())
                    .clone()
            }

            fn set_order(
                &self,
                order: ::std::vec::Vec<#krate::__nih::SignalChainSlot>,
            ) -> ::std::result::Result<(), #krate::__nih::BridgeError> {
                // ── Validation ──────────────────────────────────────────────────────
                // Known processor names (in declaration order)
                let __proc_names: [&str; #n_lit] = [#(#proc_name_str_lits),*];
                // Known tap names (in declaration order)
                let __tap_names: [&str; #t_lit] = [#(#tap_name_str_lits),*];
                let __total: usize = #n_lit + #t_lit;

                if order.len() != __total {
                    return ::std::result::Result::Err(
                        #krate::__nih::BridgeError::InvalidSignalChainOrder {
                            reason: ::std::format!(
                                "expected {} total slots ({} processor(s) + {} tap(s)), got {}",
                                __total, #n_lit, #t_lit, order.len()
                            ),
                        },
                    );
                }

                // Check each slot: id must match a known name, type must match declaration.
                let mut __proc_seen = [false; #n_lit];
                let mut __tap_seen = [false; #t_lit];
                for __slot in order.iter() {
                    match __slot.slot_type {
                        #krate::__nih::SlotType::Processor => {
                            match __proc_names.iter().position(|&__n| __n == __slot.id) {
                                Some(__idx) => {
                                    if __proc_seen[__idx] {
                                        return ::std::result::Result::Err(
                                            #krate::__nih::BridgeError::InvalidSignalChainOrder {
                                                reason: ::std::format!(
                                                    "duplicate processor slot '{}'",
                                                    __slot.id
                                                ),
                                            },
                                        );
                                    }
                                    __proc_seen[__idx] = true;
                                }
                                None => {
                                    return ::std::result::Result::Err(
                                        #krate::__nih::BridgeError::InvalidSignalChainOrder {
                                            reason: ::std::format!(
                                                "unknown processor slot '{}'",
                                                __slot.id
                                            ),
                                        },
                                    );
                                }
                            }
                        }
                        #krate::__nih::SlotType::Tap => {
                            match __tap_names.iter().position(|&__n| __n == __slot.id) {
                                Some(__idx) => {
                                    if __tap_seen[__idx] {
                                        return ::std::result::Result::Err(
                                            #krate::__nih::BridgeError::InvalidSignalChainOrder {
                                                reason: ::std::format!(
                                                    "duplicate tap slot '{}'",
                                                    __slot.id
                                                ),
                                            },
                                        );
                                    }
                                    __tap_seen[__idx] = true;
                                }
                                None => {
                                    return ::std::result::Result::Err(
                                        #krate::__nih::BridgeError::InvalidSignalChainOrder {
                                            reason: ::std::format!(
                                                "unknown tap slot '{}'",
                                                __slot.id
                                            ),
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
                // All slots must be present (no omissions)
                if __proc_seen.iter().any(|&__s| !__s)
                    || __tap_seen.iter().any(|&__s| !__s)
                {
                    return ::std::result::Result::Err(
                        #krate::__nih::BridgeError::InvalidSignalChainOrder {
                            reason: "order is missing one or more declared slots".to_string(),
                        },
                    );
                }

                // ── Resolve ────────────────────────────────────────────────────────
                // Processor execution order: u8 index in declaration order for each proc slot.
                // Tap boundaries: count of Processor slots appearing before each tap in the order.
                let mut __proc_exec: [u8; #n_lit] = [0u8; #n_lit];
                let mut __tap_boundaries: [u8; #t_lit] = [0u8; #t_lit];
                let mut __proc_exec_pos: usize = 0;
                let mut __proc_count_so_far: u8 = 0;
                for __slot in order.iter() {
                    match __slot.slot_type {
                        #krate::__nih::SlotType::Processor => {
                            let __idx = __proc_names
                                .iter()
                                .position(|&__n| __n == __slot.id)
                                .unwrap() as u8;
                            __proc_exec[__proc_exec_pos] = __idx;
                            __proc_exec_pos += 1;
                            __proc_count_so_far += 1;
                        }
                        #krate::__nih::SlotType::Tap => {
                            let __tap_idx = __tap_names
                                .iter()
                                .position(|&__n| __n == __slot.id)
                                .unwrap();
                            __tap_boundaries[__tap_idx] = __proc_count_so_far;
                        }
                    }
                }

                // ── Atomic store ────────────────────────────────────────────────────
                use ::std::sync::atomic::Ordering;
                for (__i, &__b) in __proc_exec.iter().enumerate() {
                    self.__pending_slots[__i].store(__b, Ordering::Release);
                }
                for (__i, &__b) in __tap_boundaries.iter().enumerate() {
                    self.__pending_slots[#n_lit + __i].store(__b, Ordering::Release);
                }
                self.__has_pending_order.store(true, Ordering::Release);
                *self.__order_state.lock().unwrap_or_else(|__e| __e.into_inner()) = order;
                ::std::result::Result::Ok(())
            }
        }

        impl ::std::default::Default for __WavecraftPlugin {
            fn default() -> Self {
                let (meter_producer, _meter_consumer) = #krate::create_meter_channel(64);
                let (oscilloscope_producer, _oscilloscope_consumer) =
                    #krate::create_oscilloscope_channel(8);

                // Compute cumulative param offsets at construction time (not audio-thread)
                let __param_offsets: [usize; #np1_lit] = {
                    let __counts: [usize; #n_lit] = [#(#param_count_exprs),*];
                    let mut __offs = [0usize; #np1_lit];
                    let mut __acc = 0usize;
                    let mut __i = 0usize;
                    while __i < #n_lit {
                        __acc += __counts[__i];
                        __offs[__i + 1] = __acc;
                        __i += 1;
                    }
                    __offs
                };

                Self {
                    params: ::std::sync::Arc::new(__WavecraftParams::default()),
                    #(#proc_defaults)*
                    __current_order: [#(#proc_idx_u8),*],
                    __param_offsets,
                    __cf_pos: 0,
                    __cf_dir: 0i8,
                    // Pre-grow to total param count so process() never reallocates.
                    __param_scratch: ::std::vec::Vec::with_capacity(__param_offsets[#n_lit]),
                    // Tap processors (Step 15: constructed via Default, scratch buffers grown in initialize())
                    #tap_defaults
                    oscilloscope_tap: #krate::OscilloscopeTap::with_output(oscilloscope_producer),
                    meter_producer,
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    meter_consumer: ::std::sync::Mutex::new(
                        ::std::option::Option::Some(_meter_consumer),
                    ),
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    oscilloscope_consumer: ::std::sync::Mutex::new(
                        ::std::option::Option::Some(_oscilloscope_consumer),
                    ),
                }
            }
        }

        impl #krate::__nih::Plugin for __WavecraftPlugin {
            const NAME: &'static str = #name;
            const VENDOR: &'static str = #vendor;
            const URL: &'static str = #url;
            const EMAIL: &'static str = "";
            const VERSION: &'static str = env!("CARGO_PKG_VERSION");

            const AUDIO_IO_LAYOUTS: &'static [#krate::__nih::AudioIOLayout] = &[
                #krate::__nih::AudioIOLayout {
                    main_input_channels: ::std::num::NonZeroU32::new(2),
                    main_output_channels: ::std::num::NonZeroU32::new(2),
                    ..#krate::__nih::AudioIOLayout::const_default()
                },
            ];

            const MIDI_INPUT: #krate::__nih::MidiConfig = #krate::__nih::MidiConfig::None;
            const MIDI_OUTPUT: #krate::__nih::MidiConfig = #krate::__nih::MidiConfig::None;

            type SysExMessage = ();
            type BackgroundTask = ();

            fn params(&self) -> ::std::sync::Arc<dyn #krate::__nih::Params> {
                self.params.clone()
            }

            fn editor(
                &mut self,
                _async_executor: #krate::__nih::AsyncExecutor<Self>,
            ) -> ::std::option::Option<::std::boxed::Box<dyn #krate::__nih::Editor>> {
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                {
                    let meter_consumer = self
                        .meter_consumer
                        .lock()
                        .expect("meter_consumer mutex poisoned")
                        .take();
                    let oscilloscope_consumer = self
                        .oscilloscope_consumer
                        .lock()
                        .expect("oscilloscope_consumer mutex poisoned")
                        .take();
                    #krate::editor::create_webview_editor(
                        self.params.clone(),
                        meter_consumer,
                        oscilloscope_consumer,
                        800,
                        600,
                    )
                }
                #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                {
                    None
                }
            }

            fn initialize(
                &mut self,
                _audio_io_layout: &#krate::__nih::AudioIOLayout,
                _buffer_config: &#krate::__nih::BufferConfig,
                _context: &mut impl #krate::__nih::InitContext<Self>,
            ) -> bool {
                #(#set_sample_rate_calls)*
                self.oscilloscope_tap
                    .set_sample_rate_hz(_buffer_config.sample_rate);
                // Step 15: initialize tap processors + resize scratch buffers
                #tap_initialize
                true
            }

            fn reset(&mut self) {
                #(#reset_calls)*
                // oscilloscope_tap is a TapProcessor (not Processor); call the correct reset.
                #krate::TapProcessor::reset(&mut self.oscilloscope_tap);
                // Step 15: reset tap processors
                #tap_reset
            }

            fn process(
                &mut self,
                buffer: &mut #krate::__nih::Buffer,
                _aux: &mut #krate::__nih::AuxiliaryBuffers,
                _context: &mut impl #krate::__nih::ProcessContext<Self>,
            ) -> #krate::__nih::ProcessStatus {
                // ── Phase 2/4: Block-start order handoff ─────────────────────────────
                //
                // 1. If fade-down complete (cf_dir == -1, cf_pos == 0), apply the
                //    pending order and switch to fade-up.
                if self.__cf_dir == -1i8 && self.__cf_pos == 0 {
                    self.params.__has_pending_order
                        .store(false, ::std::sync::atomic::Ordering::Release);
                    // Load processor execution order from pending_slots[0..N]
                    #(
                        self.__current_order[#proc_idx_usize] = self
                            .params
                            .__pending_slots[#proc_idx_usize]
                            .load(::std::sync::atomic::Ordering::Acquire);
                    )*
                    // Step 16: load tap boundaries from pending_slots[N..N+T]
                    #tap_boundary_handoff
                    self.__cf_dir = 1i8; // start fade-up from silence
                }
                // 2. If idle and a new pending order has arrived, start a fade-down.
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
                // clear() sets len to 0 without freeing; push() never reallocates because capacity
                // was pre-grown to the total param count at construction. Zero heap allocation per block.
                self.__param_scratch.clear();
                for __p in self.params.params.iter() {
                    self.__param_scratch.push(__p.modulated_plain_value());
                }
                // Note: __param_scratch is accessed directly in each dispatch arm below.
                // Avoiding a long-lived `&self.__param_scratch` binding here prevents a
                // cross-iteration borrow conflict (E0499) with the mutable borrows on
                // `self.__proc_N` and `self.__tap_N_scratch_*` inside the per-sample loop.

                // SAFETY: Convert self to a raw pointer BEFORE the per-sample loop so that
                // all processor dispatch calls use `(*__self_ptr).field.process(...)` rather
                // than `self.field.process(...)`.  NLL does not track loans through raw
                // pointers, so cross-iteration E0499 false-positives are avoided. The
                // pointer is valid for the entire loop duration (process_audio is
                // single-threaded and &mut self cannot alias anything else).
                let __self_ptr: *mut Self = self;

                for __sample_idx in 0..__num_samples {
                    let __transport = #krate::Transport::default();
                    // Load samples into a stack-allocated 2-channel buffer.
                    // Plugin audio layout is fixed stereo (2-in/2-out); no heap involved.
                    let mut __stereo: [f32; 2] = [
                        buffer.as_slice()[0][__sample_idx],
                        if __channels >= 2 {
                            buffer.as_slice()[1][__sample_idx]
                        } else {
                            0.0_f32
                        },
                    ];
                    let __active_ch = __channels.min(2);

                    // Per-tap capture locals: initialized to 0.0 each iteration.
                    // tap_capture_boundary_zero and tap_capture_after_dispatch write
                    // into these locals (not self.*scratch*) to keep all mutable borrows
                    // of self.__tap_*_scratch_* at a single flush point (tap_capture_flush).
                    #tap_capture_vars

                    // Step 17: capture for taps with boundary = 0 (raw input, before any processor)
                    #tap_capture_boundary_zero

                    {
                        // Create mutable slice refs over the stack array — both are stack-allocated.
                        let (__sl0, __sl1) = __stereo.split_at_mut(1);
                        let mut __channel_slices: [&mut [f32]; 2] = [__sl0, __sl1];
                        // ── Phase 2: Runtime-ordered static dispatch ──────────────────────
                        let mut __pos: usize = 0;
                        while __pos < __PROC_COUNT {
                            let __slot = self.__current_order[__pos] as usize;
                            match __slot {
                                #(#dispatch_arms)*
                                _ => {}
                            }
                            // Step 17: capture for taps whose boundary = __pos + 1
                            #tap_capture_after_dispatch
                            __pos += 1;
                        }
                    } // __channel_slices (and mutable borrow of __stereo) released here

                    // Flush per-iteration tap capture locals to self.*scratch*.
                    // This is the ONLY write to self.__tap_*_scratch_* per outer loop
                    // iteration — consolidating here avoids cross-iteration E0499 borrows.
                    #tap_capture_flush

                    // ── Phase 4: Per-sample crossfade gain ────────────────────────────
                    let __gain: f32 = if self.__cf_dir != 0i8 {
                        let __g = self.__cf_pos as f32 / __CROSSFADE_SAMPLES as f32;
                        if self.__cf_dir > 0i8 {
                            self.__cf_pos += 1;
                            if self.__cf_pos >= __CROSSFADE_SAMPLES {
                                self.__cf_pos = __CROSSFADE_SAMPLES;
                                self.__cf_dir = 0i8;
                            }
                        } else {
                            // fading down
                            if self.__cf_pos > 0 {
                                self.__cf_pos -= 1;
                            }
                        }
                        __g
                    } else {
                        1.0_f32
                    };

                    // Write processed (and gain-adjusted) samples back to the buffer.
                    // SAFETY: nih-plug process() guarantees exclusive buffer access;
                    // `__sample_idx` < `__num_samples` which came from `buffer.samples()`.
                    unsafe {
                        let __ptr = buffer.as_slice()[0].as_ptr() as *mut f32;
                        *__ptr.add(__sample_idx) = __stereo[0] * __gain;
                    }
                    if __channels >= 2 {
                        // SAFETY: channel 1 exists (__channels >= 2); same exclusive-access
                        // and bounds guarantees as channel 0.
                        unsafe {
                            let __ptr = buffer.as_slice()[1].as_ptr() as *mut f32;
                            *__ptr.add(__sample_idx) = __stereo[1] * __gain;
                        }
                    }
                }

                // ── Metering ─────────────────────────────────────────────────────────
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

                // Step 17: block-end observe_stereo call for each declared tap processor.
                // Each tap receives the captured slice from its scratch buffers (pre-filled per-sample above).
                #tap_observe_calls

                // Block-end oscilloscope capture: always final output (no allocation).
                // The hardcoded oscilloscope_tap field captures from buffer slices directly.
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

        impl #krate::__nih::ClapPlugin for __WavecraftPlugin {
            const CLAP_ID: &'static str = #clap_id;
            const CLAP_DESCRIPTION: Option<&'static str> = None;
            const CLAP_MANUAL_URL: Option<&'static str> = None;
            const CLAP_SUPPORT_URL: Option<&'static str> = None;
            const CLAP_FEATURES: &'static [#krate::__nih::ClapFeature] = &[
                #krate::__nih::ClapFeature::AudioEffect,
                #krate::__nih::ClapFeature::Stereo,
            ];
        }

        impl #krate::__nih::Vst3Plugin for __WavecraftPlugin {
            const VST3_CLASS_ID: [u8; 16] = #vst3_id;
            const VST3_SUBCATEGORIES: &'static [#krate::__nih::Vst3SubCategory] =
                &[#krate::__nih::Vst3SubCategory::Fx];
        }

        // When building with `_param-discovery` feature, skip nih-plug's
        // static initializers (VST3/CLAP factory registration) to prevent
        // dlopen from hanging on macOS audio subsystem services.
        #[cfg(not(feature = "_param-discovery"))]
        #krate::__nih::nih_export_clap!(__WavecraftPlugin);
        #[cfg(not(feature = "_param-discovery"))]
        #krate::__nih::nih_export_vst3!(__WavecraftPlugin);

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
    };
    Ok(expanded)
}
