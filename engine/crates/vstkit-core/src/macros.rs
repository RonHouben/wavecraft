/// `vstkit_plugin!` — thin macro to generate a minimal plugin skeleton.
///
/// The macro is intentionally minimal: it generates a plugin struct with
/// default constructor, `Plugin` impl using supplied metadata, and
/// `nih_export_*` registrations. It expects the supplied `params` type to
/// implement `Default + nih_plug::prelude::Params` and the `processor`
/// type to implement `Default` and a small runtime API (`set_sample_rate`).
///
/// Example (doc-test):
///
/// ```rust,ignore
/// use vstkit_core::prelude::*;
///
/// struct TestProcessor;
/// impl TestProcessor {
///     fn new() -> Self { Self }
///     fn set_sample_rate(&mut self, _sr: f32) {}
/// }
/// impl Default for TestProcessor { fn default() -> Self { Self::new() } }
///
/// #[derive(Params)]
/// struct TestParams {
///     #[id = "p"]
///     p: FloatParam,
/// }
///
/// impl Default for TestParams {
///     fn default() -> Self {
///         Self {
///             p: FloatParam::new("P", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
///         }
///     }
/// }
///
/// mod example {
///     use vstkit_core::prelude::*;
///
///     struct TestProcessor;
///     impl TestProcessor {
///         fn new() -> Self { Self }
///         fn set_sample_rate(&mut self, _sr: f32) {}
///     }
///     impl Default for TestProcessor { fn default() -> Self { Self::new() } }
///
///     #[derive(Params)]
///     struct TestParams {
///         #[id = "p"]
///         p: FloatParam,
///     }
///
///     impl Default for TestParams {
///         fn default() -> Self {
///             Self {
///                 p: FloatParam::new("P", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
///             }
///         }
///     }
///
///     vstkit_core::vstkit_plugin! {
///         ident: MacroPlugin,
///         name: "Macro Plugin",
///         vendor: "Test",
///         url: "https://example.com",
///         email: "test@example.com",
///         version: env!("CARGO_PKG_VERSION"),
///         audio: { inputs: 2, outputs: 2 },
///         params: [TestParams],
///         processor: TestProcessor,
///     }
///
///     fn use_it() {
///         let _ = MacroPlugin::default();
///     }
/// }

/// ```
#[macro_export]
macro_rules! vstkit_plugin {
    (
        ident: $ident:ident,
        name: $name:expr,
        vendor: $vendor:expr,
        url: $url:expr,
        email: $email:expr,
        version: $version:expr,
        audio: { inputs: $inputs:expr, outputs: $outputs:expr },
        params: [$param:ty],
        processor: $processor:ty $(,)?
    ) => {
        $crate::paste::paste! {
            /// Generated plugin type by `vstkit_plugin!` macro
            pub struct $ident {
                params: std::sync::Arc<$param>,
                processor: $processor,
                meter_producer: ::vstkit_metering::MeterProducer,
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                meter_consumer: std::sync::Arc<std::sync::Mutex<::vstkit_metering::MeterConsumer>>,
            }

            impl std::default::Default for $ident {
                fn default() -> Self {
                    let (meter_producer, _meter_consumer) = ::vstkit_metering::create_meter_channel(64);
                    Self {
                        params: std::sync::Arc::new(<$param>::default()),
                        processor: <$processor>::default(),
                        meter_producer,
                        #[cfg(any(target_os = "macos", target_os = "windows"))]
                        meter_consumer: std::sync::Arc::new(std::sync::Mutex::new(_meter_consumer)),
                    }
                }
            }

            impl nih_plug::prelude::Plugin for $ident {
                const NAME: &'static str = $name;
                const VENDOR: &'static str = $vendor;
                const URL: &'static str = $url;
                const EMAIL: &'static str = $email;
                const VERSION: &'static str = $version;

                const AUDIO_IO_LAYOUTS: &'static [nih_plug::prelude::AudioIOLayout] = &[nih_plug::prelude::AudioIOLayout {
                    main_input_channels: std::num::NonZeroU32::new($inputs),
                    main_output_channels: std::num::NonZeroU32::new($outputs),
                    ..nih_plug::prelude::AudioIOLayout::const_default()
                }];

                const MIDI_INPUT: nih_plug::prelude::MidiConfig = nih_plug::prelude::MidiConfig::None;
                const MIDI_OUTPUT: nih_plug::prelude::MidiConfig = nih_plug::prelude::MidiConfig::None;

                type SysExMessage = ();
                type BackgroundTask = ();

                fn params(&self) -> std::sync::Arc<dyn nih_plug::prelude::Params> {
                    self.params.clone()
                }

                fn editor(&mut self, _async_executor: nih_plug::prelude::AsyncExecutor<Self>) -> Option<Box<dyn nih_plug::prelude::Editor>> {
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    {
                        $crate::editor::create_webview_editor(self.params.clone(), self.meter_consumer.clone())
                    }

                    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                    {
                        None
                    }
                }

                fn initialize(
                    &mut self,
                    _audio_io_layout: &nih_plug::prelude::AudioIOLayout,
                    buffer_config: &nih_plug::prelude::BufferConfig,
                    _context: &mut impl nih_plug::prelude::InitContext<Self>,
                ) -> bool {
                    // Allow processor to update SR
                    self.processor.set_sample_rate(buffer_config.sample_rate);
                    true
                }

                fn process(
                    &mut self,
                    _buffer: &mut nih_plug::prelude::Buffer,
                    _aux: &mut nih_plug::prelude::AuxiliaryBuffers,
                    _context: &mut impl nih_plug::prelude::ProcessContext<Self>,
                ) -> nih_plug::prelude::ProcessStatus {
                    // Default implementation: leave processing to the user-provided processor.
                    // The processor's `process` signature must be adapted by the user; this macro
                    // intentionally does not assume a single concrete audio buffer API to avoid
                    // imposing allocations or runtime choices. Users are encouraged to implement
                    // their own `plugin::process` if they need custom behavior.

                    // Push meters after processing if desired by the processor (no-op by default)
                    let _ = &self.meter_producer; // use field to avoid dead_code warnings in unit tests
                    nih_plug::prelude::ProcessStatus::Normal
                }
            }

            impl nih_plug::prelude::Vst3Plugin for $ident {
                const VST3_CLASS_ID: [u8; 16] = *b"MacroPlug0000001";
                const VST3_SUBCATEGORIES: &'static [nih_plug::prelude::Vst3SubCategory] = &[nih_plug::prelude::Vst3SubCategory::Fx];
            }

            impl nih_plug::prelude::ClapPlugin for $ident {
                const CLAP_ID: &'static str = "dev.vstkit.macro";
                const CLAP_DESCRIPTION: Option<&'static str> = Some("Generated plugin from vstkit_plugin!");
                const CLAP_MANUAL_URL: Option<&'static str> = Some($url);
                const CLAP_SUPPORT_URL: Option<&'static str> = Some($url);
                const CLAP_FEATURES: &'static [nih_plug::prelude::ClapFeature] = &[nih_plug::prelude::ClapFeature::AudioEffect];
            }

            $crate::paste::paste! {
                #[cfg(not(test))]
                mod [<__vstkit_exports_ $ident>] {
                    nih_plug::nih_export_vst3!(crate::$ident);
                    nih_plug::nih_export_clap!(crate::$ident);
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    // Use the crate-local params and macro in unit tests

    struct TestProcessor;
    impl TestProcessor { fn new() -> Self { Self } fn set_sample_rate(&mut self, _sr: f32) {} }
    impl Default for TestProcessor { fn default() -> Self { Self::new() } }

    vstkit_plugin! {
        ident: TestMacroPlugin,
        name: "Test Macro Plugin",
        vendor: "Test",
        url: "https://example.com",
        email: "test@example.com",
        version: env!("CARGO_PKG_VERSION"),
        audio: { inputs: 2, outputs: 2 },
        params: [crate::params::VstKitParams],
        processor: TestProcessor,
    }

    #[test]
    fn macro_constructs_plugin() {
        let _p = TestMacroPlugin::default();
        assert_eq!(<TestMacroPlugin as nih_plug::prelude::Plugin>::NAME, "Test Macro Plugin");
    }
}
