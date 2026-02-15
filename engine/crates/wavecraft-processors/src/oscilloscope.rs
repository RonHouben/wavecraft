//! Oscilloscope tap processor and lock-free frame transport.
//!
//! The oscilloscope tap is observation-only: it never modifies audio samples.

use wavecraft_dsp::{Processor, Transport};
use wavecraft_protocol::{OscilloscopeFrame, OscilloscopeTriggerMode};

/// Number of points per oscilloscope frame.
pub const OSCILLOSCOPE_FRAME_POINTS: usize = 1024;
const DEFAULT_NO_SIGNAL_THRESHOLD: f32 = 1e-4;

/// Internal snapshot format with fixed-size arrays (no heap allocations).
#[derive(Clone)]
pub struct OscilloscopeFrameSnapshot {
    pub points_l: [f32; OSCILLOSCOPE_FRAME_POINTS],
    pub points_r: [f32; OSCILLOSCOPE_FRAME_POINTS],
    pub sample_rate: f32,
    pub timestamp: u64,
    pub no_signal: bool,
    pub trigger_mode: OscilloscopeTriggerMode,
}

impl OscilloscopeFrameSnapshot {
    /// Convert fixed-size snapshot into IPC frame payload.
    pub fn to_protocol_frame(&self) -> OscilloscopeFrame {
        OscilloscopeFrame {
            points_l: self.points_l.to_vec(),
            points_r: self.points_r.to_vec(),
            sample_rate: self.sample_rate,
            timestamp: self.timestamp,
            no_signal: self.no_signal,
            trigger_mode: self.trigger_mode,
        }
    }
}

/// Producer side of oscilloscope frame channel.
pub struct OscilloscopeFrameProducer {
    producer: rtrb::Producer<OscilloscopeFrameSnapshot>,
}

impl OscilloscopeFrameProducer {
    /// Push the latest frame. If the channel is full, the frame is dropped.
    pub fn push(&mut self, frame: OscilloscopeFrameSnapshot) {
        let _ = self.producer.push(frame);
    }
}

/// Consumer side of oscilloscope frame channel.
pub struct OscilloscopeFrameConsumer {
    consumer: rtrb::Consumer<OscilloscopeFrameSnapshot>,
}

impl OscilloscopeFrameConsumer {
    /// Read and return the most recent available frame.
    pub fn read_latest(&mut self) -> Option<OscilloscopeFrameSnapshot> {
        let mut latest = None;
        while let Ok(frame) = self.consumer.pop() {
            latest = Some(frame);
        }
        latest
    }
}

/// Create a lock-free oscilloscope frame channel.
pub fn create_oscilloscope_channel(
    capacity: usize,
) -> (OscilloscopeFrameProducer, OscilloscopeFrameConsumer) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity);
    (
        OscilloscopeFrameProducer { producer },
        OscilloscopeFrameConsumer { consumer },
    )
}

/// Observation-only oscilloscope tap processor.
pub struct OscilloscopeTap {
    sample_rate: f32,
    frame_l: [f32; OSCILLOSCOPE_FRAME_POINTS],
    frame_r: [f32; OSCILLOSCOPE_FRAME_POINTS],
    aligned_l: [f32; OSCILLOSCOPE_FRAME_POINTS],
    aligned_r: [f32; OSCILLOSCOPE_FRAME_POINTS],
    timestamp: u64,
    no_signal_threshold: f32,
    output: Option<OscilloscopeFrameProducer>,
}

impl Default for OscilloscopeTap {
    fn default() -> Self {
        Self {
            sample_rate: 44_100.0,
            frame_l: [0.0; OSCILLOSCOPE_FRAME_POINTS],
            frame_r: [0.0; OSCILLOSCOPE_FRAME_POINTS],
            aligned_l: [0.0; OSCILLOSCOPE_FRAME_POINTS],
            aligned_r: [0.0; OSCILLOSCOPE_FRAME_POINTS],
            timestamp: 0,
            no_signal_threshold: DEFAULT_NO_SIGNAL_THRESHOLD,
            output: None,
        }
    }
}

impl OscilloscopeTap {
    /// Create a new oscilloscope tap without an output channel.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new oscilloscope tap with frame output channel.
    pub fn with_output(output: OscilloscopeFrameProducer) -> Self {
        Self {
            output: Some(output),
            ..Self::default()
        }
    }

    /// Attach or replace the output channel.
    pub fn set_output(&mut self, output: OscilloscopeFrameProducer) {
        self.output = Some(output);
    }

    /// Set sample rate used in frame metadata.
    pub fn set_sample_rate_hz(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    /// Capture and publish a frame from stereo slices.
    pub fn capture_stereo(&mut self, left: &[f32], right: &[f32]) {
        if left.is_empty() {
            return;
        }

        let right = if right.is_empty() { left } else { right };

        // Downsample or upsample source block into fixed 1024-point frame.
        let left_len = left.len();
        let right_len = right.len();
        let mut max_abs = 0.0_f32;

        for index in 0..OSCILLOSCOPE_FRAME_POINTS {
            let source_l = index * left_len / OSCILLOSCOPE_FRAME_POINTS;
            let source_r = index * right_len / OSCILLOSCOPE_FRAME_POINTS;

            let l = left[source_l.min(left_len - 1)];
            let r = right[source_r.min(right_len - 1)];

            self.frame_l[index] = l;
            self.frame_r[index] = r;
            max_abs = max_abs.max(l.abs()).max(r.abs());
        }

        let no_signal = max_abs < self.no_signal_threshold;

        let trigger_start = if no_signal {
            0
        } else {
            self.find_rising_zero_crossing().unwrap_or(0)
        };

        if trigger_start == 0 {
            self.aligned_l.copy_from_slice(&self.frame_l);
            self.aligned_r.copy_from_slice(&self.frame_r);
        } else {
            for index in 0..OSCILLOSCOPE_FRAME_POINTS {
                let source = (trigger_start + index) % OSCILLOSCOPE_FRAME_POINTS;
                self.aligned_l[index] = self.frame_l[source];
                self.aligned_r[index] = self.frame_r[source];
            }
        }

        let frame = OscilloscopeFrameSnapshot {
            points_l: self.aligned_l,
            points_r: self.aligned_r,
            sample_rate: self.sample_rate,
            timestamp: self.timestamp,
            no_signal,
            trigger_mode: OscilloscopeTriggerMode::RisingZeroCrossing,
        };

        self.timestamp = self.timestamp.wrapping_add(1);

        if let Some(output) = self.output.as_mut() {
            output.push(frame);
        }
    }

    fn find_rising_zero_crossing(&self) -> Option<usize> {
        for index in 1..OSCILLOSCOPE_FRAME_POINTS {
            let prev = self.frame_l[index - 1];
            let current = self.frame_l[index];
            if prev <= 0.0 && current > 0.0 {
                return Some(index);
            }
        }
        None
    }
}

impl Processor for OscilloscopeTap {
    type Params = ();

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.set_sample_rate_hz(sample_rate);
    }

    fn process(
        &mut self,
        buffer: &mut [&mut [f32]],
        _transport: &Transport,
        _params: &Self::Params,
    ) {
        if buffer.is_empty() {
            return;
        }

        let left = &*buffer[0];
        let right = if buffer.len() > 1 { &*buffer[1] } else { left };

        // Observation-only capture. Audio data is never modified.
        self.capture_stereo(left, right);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_invariance() {
        let mut tap = OscilloscopeTap::new();

        let mut left = [0.25_f32, -0.1, 0.4, -0.3];
        let mut right = [-0.2_f32, 0.5, -0.4, 0.1];
        let expected_left = left;
        let expected_right = right;
        let mut buffer = [&mut left[..], &mut right[..]];

        tap.process(&mut buffer, &Transport::default(), &());

        assert_eq!(left, expected_left);
        assert_eq!(right, expected_right);
    }

    #[test]
    fn frame_length_is_1024() {
        let (producer, mut consumer) = create_oscilloscope_channel(8);
        let mut tap = OscilloscopeTap::with_output(producer);

        let left = [0.5_f32; 64];
        let right = [0.25_f32; 64];
        tap.capture_stereo(&left, &right);

        let frame = consumer.read_latest().expect("frame should exist");
        assert_eq!(frame.points_l.len(), OSCILLOSCOPE_FRAME_POINTS);
        assert_eq!(frame.points_r.len(), OSCILLOSCOPE_FRAME_POINTS);
    }

    #[test]
    fn trigger_alignment_rising_zero_crossing() {
        let (producer, mut consumer) = create_oscilloscope_channel(8);
        let mut tap = OscilloscopeTap::with_output(producer);

        let mut left = [0.0_f32; 128];
        let mut right = [0.0_f32; 128];

        for i in 0..128 {
            let phase = (i as f32 / 128.0) * std::f32::consts::TAU;
            left[i] = phase.sin();
            right[i] = left[i];
        }

        tap.capture_stereo(&left, &right);

        let frame = consumer.read_latest().expect("frame should exist");
        let first = frame.points_l[0];
        let second = frame.points_l[1];

        assert!(first <= 0.05, "expected start near zero, got {first}");
        assert!(second >= first, "expected rising edge at frame start");
    }

    #[test]
    fn no_signal_detection() {
        let (producer, mut consumer) = create_oscilloscope_channel(8);
        let mut tap = OscilloscopeTap::with_output(producer);

        let left = [1e-6_f32; 128];
        let right = [1e-6_f32; 128];
        tap.capture_stereo(&left, &right);

        let frame = consumer.read_latest().expect("frame should exist");
        assert!(frame.no_signal);
    }

    #[test]
    fn stereo_capture_integrity() {
        let (producer, mut consumer) = create_oscilloscope_channel(8);
        let mut tap = OscilloscopeTap::with_output(producer);

        let left = [0.75_f32; 128];
        let right = [-0.25_f32; 128];
        tap.capture_stereo(&left, &right);

        let frame = consumer.read_latest().expect("frame should exist");
        assert!(
            frame
                .points_l
                .iter()
                .all(|v| (*v - 0.75).abs() < f32::EPSILON)
        );
        assert!(
            frame
                .points_r
                .iter()
                .all(|v| (*v + 0.25).abs() < f32::EPSILON)
        );
    }
}
