use wavecraft_protocol::MeterUpdateNotification;

pub(super) fn maybe_build_meter_update(
    frame_counter: u64,
    left: &[f32],
    right: &[f32],
) -> Option<MeterUpdateNotification> {
    // Send meter update approximately every other callback.
    // At 44100 Hz / 512 samples per buffer ≈ 86 callbacks/sec,
    // firing every 2nd callback gives ~43 Hz visual updates.
    // The WebSocket/UI side already rate-limits display.
    if !frame_counter.is_multiple_of(2) {
        return None;
    }

    let (peak_left, rms_left) = compute_peak_and_rms(left);
    let (peak_right, rms_right) = compute_peak_and_rms(right);

    Some(MeterUpdateNotification {
        timestamp_us: frame_counter,
        left_peak: peak_left,
        left_rms: rms_left,
        right_peak: peak_right,
        right_rms: rms_right,
    })
}

fn compute_peak_and_rms(samples: &[f32]) -> (f32, f32) {
    let peak = samples
        .iter()
        .copied()
        .fold(0.0f32, |acc, sample| acc.max(sample.abs()));
    let rms =
        (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt();

    (peak, rms)
}
