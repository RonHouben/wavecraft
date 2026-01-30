//! Real-time safe metering for audio→UI communication.
//!
//! Provides lock-free SPSC ring buffers for transferring peak/RMS meter data
//! from the audio thread to the UI thread without allocations or blocking.

/// Frame of stereo metering data.
///
/// Sent from audio thread → UI thread via SPSC ring buffer.
/// All values are linear (not dB) for real-time efficiency.
#[derive(Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct MeterFrame {
    /// Left channel peak (linear, 0.0 to 1.0+)
    pub peak_l: f32,
    /// Right channel peak (linear, 0.0 to 1.0+)
    pub peak_r: f32,
    /// Left channel RMS (linear, 0.0 to 1.0+)
    pub rms_l: f32,
    /// Right channel RMS (linear, 0.0 to 1.0+)
    pub rms_r: f32,
    /// Sample timestamp (monotonic, for UI interpolation)
    pub timestamp: u64,
}

/// Producer side of meter channel (audio thread).
///
/// Real-time safe: no allocations, no locks.
pub struct MeterProducer {
    producer: rtrb::Producer<MeterFrame>,
}

impl MeterProducer {
    /// Push a meter frame to the ring buffer.
    ///
    /// Fails silently if buffer is full (drops oldest frame).
    /// This is acceptable for metering; UI will get next frame.
    #[inline]
    pub fn push(&mut self, frame: MeterFrame) {
        // Non-blocking write; if full, we just skip this frame
        let _ = self.producer.push(frame);
    }

    /// Returns the number of frames that can be written without blocking.
    #[inline]
    pub fn available_write(&self) -> usize {
        self.producer.slots()
    }
}

/// Consumer side of meter channel (UI thread).
///
/// Not real-time safe (can allocate), but non-blocking on audio thread.
pub struct MeterConsumer {
    consumer: rtrb::Consumer<MeterFrame>,
}

impl MeterConsumer {
    /// Read the latest meter frame, discarding all older frames.
    ///
    /// Returns `None` if no frames are available.
    /// Efficient for UI polling: only processes most recent data.
    pub fn read_latest(&mut self) -> Option<MeterFrame> {
        let mut latest = None;
        while let Ok(frame) = self.consumer.pop() {
            latest = Some(frame);
        }
        latest
    }

    /// Pop the oldest meter frame from the buffer.
    ///
    /// Returns `None` if no frames are available.
    pub fn pop(&mut self) -> Option<MeterFrame> {
        self.consumer.pop().ok()
    }

    /// Returns the number of frames available to read.
    pub fn available_read(&self) -> usize {
        self.consumer.slots()
    }
}

/// Create a pair of meter producer/consumer with the given buffer capacity.
///
/// Capacity should be large enough to handle UI polling delays without drops,
/// but small enough to avoid stale data.
///
/// Recommended: 32-128 frames (enough for 60 Hz UI @ 512-sample audio blocks).
pub fn create_meter_channel(capacity: usize) -> (MeterProducer, MeterConsumer) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity);
    (
        MeterProducer { producer },
        MeterConsumer { consumer },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_ring_push_pop() {
        let (mut producer, mut consumer) = create_meter_channel(4);

        let frame = MeterFrame {
            peak_l: 0.5,
            peak_r: 0.6,
            rms_l: 0.3,
            rms_r: 0.4,
            timestamp: 1000,
        };

        producer.push(frame);
        let read = consumer.pop().expect("should read frame");

        assert_eq!(read.peak_l, 0.5);
        assert_eq!(read.peak_r, 0.6);
        assert_eq!(read.rms_l, 0.3);
        assert_eq!(read.rms_r, 0.4);
        assert_eq!(read.timestamp, 1000);
    }

    #[test]
    fn meter_ring_overflow() {
        let (mut producer, mut consumer) = create_meter_channel(2);

        // Fill buffer
        producer.push(MeterFrame { peak_l: 1.0, ..Default::default() });
        producer.push(MeterFrame { peak_l: 2.0, ..Default::default() });
        
        // Overflow silently drops (ring buffer behavior)
        producer.push(MeterFrame { peak_l: 3.0, ..Default::default() });

        // First two frames should still be readable
        assert_eq!(consumer.pop().unwrap().peak_l, 1.0);
        assert_eq!(consumer.pop().unwrap().peak_l, 2.0);
        
        // Third frame was dropped
        assert!(consumer.pop().is_none());
    }

    #[test]
    fn read_latest_discards_old() {
        let (mut producer, mut consumer) = create_meter_channel(8);

        // Push multiple frames
        for i in 0..5 {
            producer.push(MeterFrame { peak_l: i as f32, ..Default::default() });
        }

        // read_latest should return only the newest frame
        let latest = consumer.read_latest().expect("should have frame");
        assert_eq!(latest.peak_l, 4.0);

        // All frames consumed
        assert!(consumer.pop().is_none());
    }

    #[test]
    fn empty_buffer_returns_none() {
        let (_, mut consumer) = create_meter_channel(4);
        assert!(consumer.pop().is_none());
        assert!(consumer.read_latest().is_none());
    }

    #[test]
    fn available_counts() {
        let (mut producer, mut consumer) = create_meter_channel(4);

        assert_eq!(consumer.available_read(), 0);
        
        producer.push(MeterFrame::default());
        producer.push(MeterFrame::default());
        
        assert_eq!(consumer.available_read(), 2);
        
        consumer.pop();
        assert_eq!(consumer.available_read(), 1);
    }
}
