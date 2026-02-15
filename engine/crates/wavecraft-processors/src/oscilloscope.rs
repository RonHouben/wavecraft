//! Oscilloscope tap processor and lock-free frame transport.
//!
//! The oscilloscope tap is observation-only: it never modifies audio samples.

use wavecraft_dsp::{Processor, Transport};
use wavecraft_protocol::{OscilloscopeFrame, OscilloscopeTriggerMode};

/// Number of points per oscilloscope frame.
pub const OSCILLOSCOPE_FRAME_POINTS: usize = 1024;
const OSCILLOSCOPE_HISTORY_FRAMES: usize = 3;
const OSCILLOSCOPE_HISTORY_POINTS: usize = OSCILLOSCOPE_FRAME_POINTS * OSCILLOSCOPE_HISTORY_FRAMES;
const OSCILLOSCOPE_HISTORY_TAIL_START: usize =
    OSCILLOSCOPE_FRAME_POINTS * (OSCILLOSCOPE_HISTORY_FRAMES - 1);
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
    history_l: [f32; OSCILLOSCOPE_HISTORY_POINTS],
    history_r: [f32; OSCILLOSCOPE_HISTORY_POINTS],
    aligned_l: [f32; OSCILLOSCOPE_FRAME_POINTS],
    aligned_r: [f32; OSCILLOSCOPE_FRAME_POINTS],
    history_frames_filled: usize,
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
            history_l: [0.0; OSCILLOSCOPE_HISTORY_POINTS],
            history_r: [0.0; OSCILLOSCOPE_HISTORY_POINTS],
            aligned_l: [0.0; OSCILLOSCOPE_FRAME_POINTS],
            aligned_r: [0.0; OSCILLOSCOPE_FRAME_POINTS],
            history_frames_filled: 0,
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

        // Keep a rolling three-frame history so the trigger-aligned frame can
        // always be extracted as a contiguous 1024-sample window without
        // wrapping or synthetic tail padding.
        self.history_l
            .copy_within(OSCILLOSCOPE_FRAME_POINTS..OSCILLOSCOPE_HISTORY_POINTS, 0);
        self.history_r
            .copy_within(OSCILLOSCOPE_FRAME_POINTS..OSCILLOSCOPE_HISTORY_POINTS, 0);
        self.history_l[OSCILLOSCOPE_HISTORY_TAIL_START..].copy_from_slice(&self.frame_l);
        self.history_r[OSCILLOSCOPE_HISTORY_TAIL_START..].copy_from_slice(&self.frame_r);

        self.history_frames_filled =
            (self.history_frames_filled + 1).min(OSCILLOSCOPE_HISTORY_FRAMES);

        let min_trigger_start = match self.history_frames_filled {
            0 | 1 => None,
            // Avoid index 1024 during startup while oldest history is still zero-filled.
            2 => Some(OSCILLOSCOPE_FRAME_POINTS + 1),
            _ => Some(1),
        };

        let trigger_start = if no_signal {
            OSCILLOSCOPE_HISTORY_TAIL_START
        } else if let Some(min_start) = min_trigger_start {
            self.find_rising_zero_crossing_in_history(min_start)
                .unwrap_or(OSCILLOSCOPE_HISTORY_TAIL_START)
        } else {
            OSCILLOSCOPE_HISTORY_TAIL_START
        };

        let end = trigger_start + OSCILLOSCOPE_FRAME_POINTS;
        self.aligned_l
            .copy_from_slice(&self.history_l[trigger_start..end]);
        self.aligned_r
            .copy_from_slice(&self.history_r[trigger_start..end]);

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

    fn find_rising_zero_crossing_in_history(&self, min_start: usize) -> Option<usize> {
        // Search only starts that can provide a full 1024-point window.
        // With 3-frame history this allows deterministic trigger lock even
        // when low frequencies do not provide a crossing in the oldest frame.
        let max_start = OSCILLOSCOPE_HISTORY_POINTS - OSCILLOSCOPE_FRAME_POINTS;
        let preferred_start = (min_start + max_start) / 2;
        let mut best_index: Option<usize> = None;
        let mut best_distance = usize::MAX;

        for index in min_start..=max_start {
            let prev = self.history_l[index - 1];
            let current = self.history_l[index];
            if prev <= 0.0 && current > 0.0 {
                let distance = index.abs_diff(preferred_start);
                let prefer_candidate = distance < best_distance
                    || (distance == best_distance
                        && best_index.is_none_or(|existing| index > existing));
                if prefer_candidate {
                    best_index = Some(index);
                    best_distance = distance;
                }
            }
        }

        best_index
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
    fn trigger_alignment_uses_contiguous_window_across_frame_boundary() {
        let (producer, mut consumer) = create_oscilloscope_channel(8);
        let mut tap = OscilloscopeTap::with_output(producer);

        let mut left_prev = [0.5_f32; OSCILLOSCOPE_FRAME_POINTS];
        let mut right_prev = [0.25_f32; OSCILLOSCOPE_FRAME_POINTS];
        left_prev[1000] = -0.1;
        left_prev[1001] = 0.1;
        right_prev[1000] = -0.3;
        right_prev[1001] = -0.2;

        let mut left_curr = [0.0_f32; OSCILLOSCOPE_FRAME_POINTS];
        let mut right_curr = [0.0_f32; OSCILLOSCOPE_FRAME_POINTS];
        for index in 0..OSCILLOSCOPE_FRAME_POINTS {
            left_curr[index] = -1.0 + (2.0 * index as f32 / OSCILLOSCOPE_FRAME_POINTS as f32);
            right_curr[index] = 1.0 - (2.0 * index as f32 / OSCILLOSCOPE_FRAME_POINTS as f32);
        }

        tap.capture_stereo(&left_prev, &right_prev);
        let _first = consumer.read_latest().expect("first frame should exist");

        tap.capture_stereo(&left_curr, &right_curr);
        let second = consumer.read_latest().expect("second frame should exist");

        // Frame starts at the trigger crossing in the previous frame.
        assert!((second.points_l[0] - left_prev[1001]).abs() < f32::EPSILON);

        // The window remains contiguous through the boundary into current data.
        assert!((second.points_l[23] - left_curr[0]).abs() < f32::EPSILON);
        assert!((second.points_r[23] - right_curr[0]).abs() < f32::EPSILON);

        // Right edge is true continuation, not a padded flat tail.
        assert!((second.points_l[1023] - left_curr[1000]).abs() < f32::EPSILON);
        assert!((second.points_r[1023] - right_curr[1000]).abs() < f32::EPSILON);
    }

    #[test]
    fn trigger_alignment_finds_low_frequency_crossings_beyond_oldest_frame() {
        let (producer, mut consumer) = create_oscilloscope_channel(8);
        let mut tap = OscilloscopeTap::with_output(producer);

        let mut left_prev = [-0.5_f32; OSCILLOSCOPE_FRAME_POINTS];
        let mut right_prev = [-0.25_f32; OSCILLOSCOPE_FRAME_POINTS];
        left_prev[200] = 0.5;
        right_prev[200] = 0.25;

        let left_curr = [-0.5_f32; OSCILLOSCOPE_FRAME_POINTS];
        let right_curr = [-0.25_f32; OSCILLOSCOPE_FRAME_POINTS];

        tap.capture_stereo(&left_prev, &right_prev);
        let _first = consumer.read_latest().expect("first frame should exist");

        tap.capture_stereo(&left_curr, &right_curr);
        let second = consumer.read_latest().expect("second frame should exist");

        // Crossing in the middle history frame at index 1024 + 200 = 1224
        // should be selected. The previous two-frame implementation searched
        // only up to index 1024 and would miss this crossing.
        assert!(
            (second.points_l[0] - left_prev[200]).abs() < f32::EPSILON,
            "expected left start {}, got {}",
            left_prev[200],
            second.points_l[0]
        );
        assert!(
            (second.points_r[0] - right_prev[200]).abs() < f32::EPSILON,
            "expected right start {}, got {}",
            right_prev[200],
            second.points_r[0]
        );

        // Window remains contiguous into current frame data.
        assert!((second.points_l[824] - left_curr[0]).abs() < f32::EPSILON);
        assert!((second.points_r[824] - right_curr[0]).abs() < f32::EPSILON);
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
