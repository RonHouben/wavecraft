// Import everything from Wavecraft SDK
// This re-exports nih-plug types, so you don't need to depend on nih-plug directly
use wavecraft_core::prelude::*;

// Wavecraft SDK components
use wavecraft_core::{editor::create_webview_editor, util::calculate_stereo_meters};
use wavecraft_dsp::{Processor as WavecraftProcessor, Transport as WavecraftTransport};
// Note: MeterFrame, MeterProducer, MeterConsumer, create_meter_channel are in prelude

use std::sync::Arc;

/// Example gain plugin using Wavecraft SDK
pub struct MyPlugin {
    params: Arc<MyPluginParams>,
    processor: GainProcessor,
    meter_producer: MeterProducer,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    meter_consumer: Arc<std::sync::Mutex<MeterConsumer>>,
}

#[derive(Params)]
pub struct MyPluginParams {
    /// Input gain in decibels
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for MyPlugin {
    fn default() -> Self {
        let (meter_producer, _meter_consumer) = create_meter_channel(64);
        Self {
            params: Arc::new(MyPluginParams::default()),
            processor: GainProcessor::default(),
            meter_producer,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            meter_consumer: Arc::new(std::sync::Mutex::new(_meter_consumer)),
        }
    }
}

impl Default for MyPluginParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-24.0),
                    max: util::db_to_gain(24.0),
                    factor: FloatRange::gain_skew_factor(-24.0, 24.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

impl Plugin for MyPlugin {
    const NAME: &'static str = "My Plugin";
    const VENDOR: &'static str = "My Company";
    const URL: &'static str = "https://example.com";
    const EMAIL: &'static str = "contact@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.processor
            .set_sample_rate(buffer_config.sample_rate);
        true
    }

    fn reset(&mut self) {
        self.processor.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Get current parameter values
        let gain = self.params.gain.smoothed.next();

        // Get transport info
        let transport_info = context.transport();
        let wavecraft_transport = WavecraftTransport {
            tempo: transport_info.tempo,
            playing: transport_info.playing,
            pos_samples: transport_info.pos_samples().unwrap_or(0),
        };

        // Create mutable slices for processor
        let channels = buffer.as_slice();
        let mut channel_ptrs: Vec<&mut [f32]> = channels.iter_mut().map(|c| &mut c[..]).collect();
        
        // Process audio using Wavecraft processor trait
        self.processor.process(&mut channel_ptrs, &wavecraft_transport);

        // Apply gain
        for sample in buffer.iter_samples() {
            for channel_sample in sample {
                *channel_sample *= gain;
            }
        }

        // Calculate and push meter data (after processing)
        if buffer.channels() >= 2 {
            let (peak_l, peak_r, rms_l, rms_r) = calculate_stereo_meters(&*buffer);
            self.meter_producer.push(MeterFrame {
                peak_l,
                peak_r,
                rms_l,
                rms_r,
                timestamp: 0,
            });
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        create_webview_editor(self.params.clone(), self.meter_consumer.clone())
    }
}
#[derive(Default)]
struct GainProcessor {
    sample_rate: f32,
}

impl WavecraftProcessor for GainProcessor {
    fn process(&mut self, buffer: &mut [&mut [f32]], _transport: &WavecraftTransport) {
        // Simple passthrough - actual gain is applied in the plugin's process()
        // This demonstrates the Processor trait pattern
        let _ = buffer; // Placeholder processing
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn reset(&mut self) {
        // Reset any internal state here
    }
}

impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "com.example.my-plugin";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("A simple gain plugin built with Wavecraft");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for MyPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"WavecraftPlugin0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Tools,
    ];
}

nih_export_clap!(MyPlugin);
nih_export_vst3!(MyPlugin);
