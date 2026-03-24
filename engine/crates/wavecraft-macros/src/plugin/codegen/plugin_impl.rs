use proc_macro2::TokenStream;
use quote::quote;

/// Generate `impl Default`, `impl Plugin`, `impl ClapPlugin`, `impl Vst3Plugin`,
/// and the `nih_export_*!` calls for `__WavecraftPlugin`.
pub(super) fn build(
    krate: &syn::Path,
    name: &syn::LitStr,
    vendor: &str,
    url: &str,
    clap_id: &str,
    vst3_id: &TokenStream,
    s: &super::context::SharedSymbols,
    sc: &super::signal_chain::SignalChainTokens,
    process_method_tokens: &TokenStream,
) -> TokenStream {
    let n_lit = &s.n_lit;
    let np1_lit = &s.np1_lit;
    let proc_idx_u8 = &s.proc_idx_u8;

    let param_count_exprs = &sc.param_count_exprs;
    let proc_defaults = &sc.proc_defaults;
    let tap_defaults = &sc.tap_defaults;
    let tap_initialize = &sc.tap_initialize;
    let set_sample_rate_calls = &sc.set_sample_rate_calls;
    let tap_reset = &sc.tap_reset;
    let reset_calls = &sc.reset_calls;

    quote! {
        impl ::std::default::Default for __WavecraftPlugin {
            fn default() -> Self {
                let (meter_producer, _meter_consumer) = #krate::create_meter_channel(64);
                let (__oscilloscope_producer, __oscilloscope_consumer) =
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
                    meter_producer,
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    meter_consumer: ::std::sync::Mutex::new(
                        ::std::option::Option::Some(_meter_consumer),
                    ),
                    oscilloscope_consumer: ::std::sync::Mutex::new(
                        ::std::option::Option::Some(__oscilloscope_consumer),
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
                // Step 15: initialize tap processors + resize scratch buffers
                #tap_initialize
                true
            }

            fn reset(&mut self) {
                #(#reset_calls)*
                // Step 15: reset tap processors
                #tap_reset
            }

            #process_method_tokens
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
    }
}
