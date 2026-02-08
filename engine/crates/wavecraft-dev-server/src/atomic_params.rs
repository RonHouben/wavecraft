//! Lock-free parameter bridge for audio thread access.
//!
//! Provides `AtomicParameterBridge` — a collection of `AtomicF32` values keyed
//! by parameter ID. The WebSocket thread writes parameter updates via `store()`,
//! and the audio thread reads them via `load()` with zero allocations and zero
//! locks. Relaxed ordering is sufficient because parameter updates are not
//! synchronization points; a one-block delay in propagation is acceptable.

#[cfg(feature = "audio")]
pub mod implementation {
    use atomic_float::AtomicF32;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use wavecraft_protocol::ParameterInfo;

    /// Lock-free bridge for passing parameter values from the WebSocket thread
    /// to the audio thread.
    ///
    /// Constructed once at startup with one `Arc<AtomicF32>` per parameter. The
    /// inner `HashMap` is never mutated after construction — only the atomic
    /// values change. This makes reads fully lock-free and real-time safe.
    pub struct AtomicParameterBridge {
        params: HashMap<String, Arc<AtomicF32>>,
    }

    impl AtomicParameterBridge {
        /// Create a new bridge from parameter metadata.
        ///
        /// Each parameter gets an `AtomicF32` initialized to its default value.
        pub fn new(parameters: &[ParameterInfo]) -> Self {
            let params = parameters
                .iter()
                .map(|p| (p.id.clone(), Arc::new(AtomicF32::new(p.default))))
                .collect();
            Self { params }
        }

        /// Write a parameter value (called from WebSocket thread).
        ///
        /// Uses `Ordering::Relaxed` — no synchronization guarantee beyond
        /// eventual visibility. The audio thread will see the update at the
        /// next block boundary.
        pub fn write(&self, id: &str, value: f32) {
            if let Some(atomic) = self.params.get(id) {
                atomic.store(value, Ordering::Relaxed);
            }
        }

        /// Read a parameter value (called from audio thread — RT-safe).
        ///
        /// Returns `None` if the parameter ID is unknown. Uses
        /// `Ordering::Relaxed` — single atomic load, no allocation.
        pub fn read(&self, id: &str) -> Option<f32> {
            self.params.get(id).map(|a| a.load(Ordering::Relaxed))
        }
    }

    // SAFETY: AtomicParameterBridge is safe to share between threads.
    // The HashMap is immutable after construction, and all values are
    // accessed through atomic operations. Arc<AtomicF32> is Send + Sync.
    // These impls are auto-derived, but stated explicitly for clarity.
    unsafe impl Send for AtomicParameterBridge {}
    unsafe impl Sync for AtomicParameterBridge {}

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
                    unit: Some("dB".to_string()),
                    group: Some("Input".to_string()),
                },
                ParameterInfo {
                    id: "mix".to_string(),
                    name: "Mix".to_string(),
                    param_type: ParameterType::Float,
                    value: 1.0,
                    default: 1.0,
                    unit: Some("%".to_string()),
                    group: None,
                },
            ]
        }

        #[test]
        fn test_default_values() {
            let bridge = AtomicParameterBridge::new(&test_params());

            let gain = bridge.read("gain").expect("gain should exist");
            assert!(
                (gain - 0.5).abs() < f32::EPSILON,
                "gain default should be 0.5"
            );

            let mix = bridge.read("mix").expect("mix should exist");
            assert!(
                (mix - 1.0).abs() < f32::EPSILON,
                "mix default should be 1.0"
            );
        }

        #[test]
        fn test_write_and_read() {
            let bridge = AtomicParameterBridge::new(&test_params());

            bridge.write("gain", 0.75);
            let gain = bridge.read("gain").expect("gain should exist");
            assert!(
                (gain - 0.75).abs() < f32::EPSILON,
                "gain should be updated to 0.75"
            );
        }

        #[test]
        fn test_read_unknown_param() {
            let bridge = AtomicParameterBridge::new(&test_params());
            assert!(
                bridge.read("nonexistent").is_none(),
                "unknown param should return None"
            );
        }

        #[test]
        fn test_write_unknown_param_is_noop() {
            let bridge = AtomicParameterBridge::new(&test_params());
            // Should not panic
            bridge.write("nonexistent", 0.5);
        }

        #[test]
        fn test_concurrent_write_read() {
            use std::sync::Arc;
            use std::thread;

            let bridge = Arc::new(AtomicParameterBridge::new(&test_params()));

            let writer = {
                let bridge = Arc::clone(&bridge);
                thread::spawn(move || {
                    for i in 0..1000 {
                        bridge.write("gain", i as f32 / 1000.0);
                    }
                })
            };

            let reader = {
                let bridge = Arc::clone(&bridge);
                thread::spawn(move || {
                    for _ in 0..1000 {
                        let val = bridge.read("gain");
                        assert!(val.is_some(), "gain should always be readable");
                        let v = val.unwrap();
                        assert!(
                            (0.0..=1.0).contains(&v) || v == 0.5,
                            "value should be in range"
                        );
                    }
                })
            };

            writer.join().expect("writer thread should not panic");
            reader.join().expect("reader thread should not panic");
        }
    }
}

#[cfg(feature = "audio")]
pub use implementation::*;
