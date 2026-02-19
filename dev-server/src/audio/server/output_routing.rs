/// Routes stereo samples from the ring buffer into the device output layout.
///
/// Behavior is intentionally preserved from the original output callback:
/// - `output_channels == 0` => fill with silence.
/// - Ring underflow => missing samples become silence.
/// - Mono output => downmix stereo to `(L + R) * 0.5`.
/// - Multi-channel output => map L/R to channels 0/1, fill channels 2+ with silence.
#[inline]
pub(super) fn route_output_callback(
    data: &mut [f32],
    output_channels: usize,
    ring_consumer: &mut rtrb::Consumer<f32>,
) {
    if output_channels == 0 {
        data.fill(0.0);
        return;
    }

    // Route stereo frames from the ring into the device layout.
    // Underflow is filled with silence.
    for frame in data.chunks_mut(output_channels) {
        let left = ring_consumer.pop().unwrap_or(0.0);
        let right = ring_consumer.pop().unwrap_or(0.0);

        if output_channels == 1 {
            frame[0] = 0.5 * (left + right);
            continue;
        }

        frame[0] = left;
        frame[1] = right;

        for channel in frame.iter_mut().skip(2) {
            *channel = 0.0;
        }
    }
}
