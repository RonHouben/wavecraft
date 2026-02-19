//! Development server host implementing ParameterHost trait
//!
//! This module provides a ParameterHost implementation for the embedded
//! development server. It stores parameter values in memory and forwards
//! parameter changes to an optional AtomicParameterBridge for lock-free
//! audio-thread access.

#[cfg(feature = "audio")]
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use wavecraft_bridge::{BridgeError, InMemoryParameterHost, ParameterHost};
use wavecraft_protocol::{
    AudioRuntimePhase, AudioRuntimeStatus, MeterFrame, MeterUpdateNotification, OscilloscopeFrame,
    ParameterInfo,
};

#[cfg(feature = "audio")]
use crate::audio::atomic_params::AtomicParameterBridge;

/// Development server host for browser-based UI testing
///
/// This implementation stores parameter values locally and optionally
/// forwards updates to an `AtomicParameterBridge` for lock-free reads
/// on the audio thread. Meter data is provided externally via the audio
/// server's meter channel (not generated synthetically).
///
/// # Thread Safety
///
/// Parameter state is protected by RwLock (in `InMemoryParameterHost`).
/// The `AtomicParameterBridge` uses lock-free atomics for audio thread.
pub struct DevServerHost {
    inner: InMemoryParameterHost,
    latest_meter_frame: Arc<RwLock<Option<MeterFrame>>>,
    latest_oscilloscope_frame: Arc<RwLock<Option<OscilloscopeFrame>>>,
    audio_status: Arc<RwLock<AudioRuntimeStatus>>,
    #[cfg(feature = "audio")]
    param_bridge: Option<Arc<AtomicParameterBridge>>,
}

impl DevServerHost {
    /// Create a new dev server host with parameter metadata.
    ///
    /// # Arguments
    ///
    /// * `parameters` - Parameter metadata loaded from the plugin FFI
    ///
    /// Used by tests and the non-audio build path. When `audio` is
    /// enabled (default), production code uses `with_param_bridge()` instead.
    #[cfg_attr(feature = "audio", allow(dead_code))]
    pub fn new(parameters: Vec<ParameterInfo>) -> Self {
        let inner = InMemoryParameterHost::new(parameters);
        let latest_meter_frame = Arc::new(RwLock::new(None));
        let latest_oscilloscope_frame = Arc::new(RwLock::new(None));
        let audio_status = Arc::new(RwLock::new(AudioRuntimeStatus {
            phase: AudioRuntimePhase::Disabled,
            diagnostic: None,
            sample_rate: None,
            buffer_size: None,
            updated_at_ms: now_millis(),
        }));

        Self {
            inner,
            latest_meter_frame,
            latest_oscilloscope_frame,
            audio_status,
            #[cfg(feature = "audio")]
            param_bridge: None,
        }
    }

    /// Create a new dev server host with an `AtomicParameterBridge`.
    ///
    /// When a bridge is provided, `set_parameter()` will write updates
    /// to both the inner store and the bridge (for audio-thread reads).
    #[cfg(feature = "audio")]
    pub fn with_param_bridge(
        parameters: Vec<ParameterInfo>,
        bridge: Arc<AtomicParameterBridge>,
    ) -> Self {
        let inner = InMemoryParameterHost::new(parameters);
        let latest_meter_frame = Arc::new(RwLock::new(None));
        let latest_oscilloscope_frame = Arc::new(RwLock::new(None));
        let audio_status = Arc::new(RwLock::new(AudioRuntimeStatus {
            phase: AudioRuntimePhase::Disabled,
            diagnostic: None,
            sample_rate: None,
            buffer_size: None,
            updated_at_ms: now_millis(),
        }));

        Self {
            inner,
            latest_meter_frame,
            latest_oscilloscope_frame,
            audio_status,
            param_bridge: Some(bridge),
        }
    }

    /// Replace all parameters with new metadata from a hot-reload.
    ///
    /// Preserves values for parameters with matching IDs. New parameters
    /// get their default values. This is used by the hot-reload pipeline
    /// to update parameter definitions without restarting the server.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter replacement fails (e.g., unrecoverable
    /// lock poisoning).
    pub fn replace_parameters(&self, new_params: Vec<ParameterInfo>) -> Result<(), String> {
        self.inner.replace_parameters(new_params)
    }

    /// Store the latest metering snapshot for polling-based consumers.
    pub fn set_latest_meter_frame(&self, update: &MeterUpdateNotification) {
        let mut meter = self
            .latest_meter_frame
            .write()
            .expect("latest_meter_frame lock poisoned");
        *meter = Some(MeterFrame {
            peak_l: update.left_peak,
            peak_r: update.right_peak,
            rms_l: update.left_rms,
            rms_r: update.right_rms,
            timestamp: update.timestamp_us,
        });
    }

    /// Store the latest oscilloscope frame for polling-based consumers.
    pub fn set_latest_oscilloscope_frame(&self, frame: OscilloscopeFrame) {
        let mut oscilloscope = self
            .latest_oscilloscope_frame
            .write()
            .expect("latest_oscilloscope_frame lock poisoned");
        *oscilloscope = Some(frame);
    }

    /// Update the shared audio runtime status.
    pub fn set_audio_status(&self, status: AudioRuntimeStatus) {
        let mut current = self
            .audio_status
            .write()
            .expect("audio_status lock poisoned");
        *current = status;
    }
}

impl ParameterHost for DevServerHost {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        self.inner.get_parameter(id)
    }

    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
        let result = self.inner.set_parameter(id, value);

        // Forward to atomic bridge for audio-thread access (lock-free)
        #[cfg(feature = "audio")]
        if result.is_ok()
            && let Some(ref bridge) = self.param_bridge
        {
            bridge.write(id, value);
        }

        result
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        self.inner.get_all_parameters()
    }

    fn get_meter_frame(&self) -> Option<MeterFrame> {
        *self
            .latest_meter_frame
            .read()
            .expect("latest_meter_frame lock poisoned")
    }

    fn get_oscilloscope_frame(&self) -> Option<OscilloscopeFrame> {
        self.latest_oscilloscope_frame
            .read()
            .expect("latest_oscilloscope_frame lock poisoned")
            .clone()
    }

    fn request_resize(&self, _width: u32, _height: u32) -> bool {
        self.inner.request_resize(_width, _height)
    }

    fn get_audio_status(&self) -> Option<AudioRuntimeStatus> {
        Some(
            self.audio_status
                .read()
                .expect("audio_status lock poisoned")
                .clone(),
        )
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wavecraft_protocol::ParameterType;

    fn test_params() -> Vec<ParameterInfo> {
        vec![
            ParameterInfo {
                id: "gain".to_string(),
                name: "Gain".to_string(),
                param_type: ParameterType::Float,
                value: 0.5,
                default: 0.5,
                min: 0.0,
                max: 1.0,
                unit: Some("dB".to_string()),
                group: Some("Input".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "mix".to_string(),
                name: "Mix".to_string(),
                param_type: ParameterType::Float,
                value: 1.0,
                default: 1.0,
                min: 0.0,
                max: 1.0,
                unit: Some("%".to_string()),
                group: None,
                variants: None,
            },
        ]
    }

    #[test]
    fn test_get_parameter() {
        let host = DevServerHost::new(test_params());

        let param = host.get_parameter("gain").expect("should find gain");
        assert_eq!(param.id, "gain");
        assert_eq!(param.name, "Gain");
        assert!((param.value - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_get_parameter_not_found() {
        let host = DevServerHost::new(test_params());
        assert!(host.get_parameter("nonexistent").is_none());
    }

    #[test]
    fn test_set_parameter() {
        let host = DevServerHost::new(test_params());

        host.set_parameter("gain", 0.75).expect("should set gain");

        let param = host.get_parameter("gain").expect("should find gain");
        assert!((param.value - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_parameter_invalid_id() {
        let host = DevServerHost::new(test_params());
        let result = host.set_parameter("invalid", 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_parameter_out_of_range() {
        let host = DevServerHost::new(test_params());

        let result = host.set_parameter("gain", 1.5);
        assert!(result.is_err());

        let result = host.set_parameter("gain", -0.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_parameters() {
        let host = DevServerHost::new(test_params());

        let params = host.get_all_parameters();
        assert_eq!(params.len(), 2);
        assert!(params.iter().any(|p| p.id == "gain"));
        assert!(params.iter().any(|p| p.id == "mix"));
    }

    #[test]
    fn test_get_meter_frame() {
        let host = DevServerHost::new(test_params());
        // Initially no externally provided meter data.
        assert!(host.get_meter_frame().is_none());

        host.set_latest_meter_frame(&MeterUpdateNotification {
            timestamp_us: 42,
            left_peak: 0.9,
            left_rms: 0.4,
            right_peak: 0.8,
            right_rms: 0.3,
        });

        let frame = host
            .get_meter_frame()
            .expect("meter frame should be populated after update");
        assert!((frame.peak_l - 0.9).abs() < f32::EPSILON);
        assert!((frame.rms_r - 0.3).abs() < f32::EPSILON);
        assert_eq!(frame.timestamp, 42);
    }

    #[test]
    fn test_audio_status_roundtrip() {
        let host = DevServerHost::new(test_params());

        let status = AudioRuntimeStatus {
            phase: AudioRuntimePhase::RunningInputOnly,
            diagnostic: None,
            sample_rate: Some(44100.0),
            buffer_size: Some(512),
            updated_at_ms: 100,
        };

        host.set_audio_status(status.clone());

        let stored = host
            .get_audio_status()
            .expect("audio status should always be present in dev host");
        assert_eq!(stored.phase, status.phase);
        assert_eq!(stored.buffer_size, status.buffer_size);
    }

    #[test]
    fn test_get_oscilloscope_frame() {
        let host = DevServerHost::new(test_params());
        assert!(host.get_oscilloscope_frame().is_none());

        host.set_latest_oscilloscope_frame(OscilloscopeFrame {
            points_l: vec![0.1; 1024],
            points_r: vec![0.2; 1024],
            sample_rate: 48_000.0,
            timestamp: 777,
            no_signal: false,
            trigger_mode: wavecraft_protocol::OscilloscopeTriggerMode::RisingZeroCrossing,
        });

        let frame = host
            .get_oscilloscope_frame()
            .expect("oscilloscope frame should be populated");
        assert_eq!(frame.points_l.len(), 1024);
        assert_eq!(frame.points_r.len(), 1024);
        assert_eq!(frame.timestamp, 777);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_set_audio_status_inside_runtime_does_not_panic() {
        let host = DevServerHost::new(test_params());

        host.set_audio_status(AudioRuntimeStatus {
            phase: AudioRuntimePhase::Initializing,
            diagnostic: None,
            sample_rate: Some(48000.0),
            buffer_size: Some(256),
            updated_at_ms: 200,
        });

        let stored = host
            .get_audio_status()
            .expect("audio status should always be present in dev host");
        assert_eq!(stored.phase, AudioRuntimePhase::Initializing);
        assert_eq!(stored.buffer_size, Some(256));
    }
}
