mod context;
mod ffi;
mod params;
mod plugin_impl;
mod process_method;
mod signal_chain;

use context::SharedSymbols;
use quote::quote;

pub(super) struct CodegenInput<'a> {
    pub(super) name: &'a syn::LitStr,
    pub(super) processors: &'a [syn::Type],
    /// Tap processor types declared in `taps: [...]`. Empty when none declared.
    pub(super) taps: &'a [syn::Type],
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

    // ── Step 14: Compile-time overlap check (type appears in both processors and taps) ──
    //
    // A type cannot implement both Processor and TapProcessor. Detect this at macro
    // expansion time (before code generation) for a clear, actionable error message.
    {
        let proc_names: Vec<String> = processors
            .iter()
            .map(context::type_last_segment_name)
            .collect();
        for tap_ty in taps.iter() {
            let tap_name = context::type_last_segment_name(tap_ty);
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

    let s = SharedSymbols::new(processors, taps);
    let n_lit = &s.n_lit;
    let np1_lit = &s.np1_lit;

    let sc = signal_chain::build(processors, taps, krate, &s);
    let proc_struct_fields = &sc.proc_struct_fields;
    let tap_struct_fields = &sc.tap_struct_fields;
    let proc_validations = &sc.proc_validations;
    let tap_validations = &sc.tap_validations;
    let initial_order_state_slots = &sc.initial_order_state_slots;

    let params_tokens = params::build(krate, runtime_param_blocks, &s, initial_order_state_slots);
    let process_method_tokens = process_method::build(krate, &s, &sc);
    let plugin_impl_tokens = plugin_impl::build(
        krate,
        name,
        vendor,
        url,
        clap_id,
        vst3_id,
        &s,
        &sc,
        &process_method_tokens,
    );
    let ffi_tokens = ffi::build(krate, processor_param_mappings, processor_info_entries);

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

        #params_tokens
        #plugin_impl_tokens
        #ffi_tokens
    };
    Ok(expanded)
}
