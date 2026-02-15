//! Audio runtime status helpers.
//!
//! This module provides constructors for `AudioRuntimeStatus` snapshots used by
//! browser dev mode startup and diagnostics.

use std::time::{SystemTime, UNIX_EPOCH};
use wavecraft_protocol::{
    AudioDiagnostic, AudioDiagnosticCode, AudioRuntimePhase, AudioRuntimeStatus,
};

/// Build a status snapshot without diagnostics.
pub fn status(
    phase: AudioRuntimePhase,
    sample_rate: Option<f32>,
    buffer_size: Option<u32>,
) -> AudioRuntimeStatus {
    AudioRuntimeStatus {
        phase,
        diagnostic: None,
        sample_rate,
        buffer_size,
        updated_at_ms: now_millis(),
    }
}

/// Build a status snapshot with structured diagnostics.
pub fn status_with_diagnostic(
    phase: AudioRuntimePhase,
    code: AudioDiagnosticCode,
    message: impl Into<String>,
    hint: Option<&str>,
    sample_rate: Option<f32>,
    buffer_size: Option<u32>,
) -> AudioRuntimeStatus {
    AudioRuntimeStatus {
        phase,
        diagnostic: Some(AudioDiagnostic {
            code,
            message: message.into(),
            hint: hint.map(ToOwned::to_owned),
        }),
        sample_rate,
        buffer_size,
        updated_at_ms: now_millis(),
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}
