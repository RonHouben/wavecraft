//! Concurrency control for rebuild operations
//!
//! Ensures only one build runs at a time, with at most one pending.
//! Uses atomics for lock-free coordination between watcher and builder.

use std::sync::atomic::{AtomicBool, Ordering};

/// Concurrency control for rebuild operations.
///
/// Ensures only one build runs at a time, with at most one pending.
/// Uses atomics for lock-free coordination between watcher and builder.
pub struct BuildGuard {
    building: AtomicBool,
    pending: AtomicBool,
}

impl BuildGuard {
    pub fn new() -> Self {
        Self {
            building: AtomicBool::new(false),
            pending: AtomicBool::new(false),
        }
    }

    /// Try to start a build. Returns true if acquired.
    pub fn try_start(&self) -> bool {
        self.building
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// Mark a pending rebuild request (received during active build).
    pub fn mark_pending(&self) {
        self.pending.store(true, Ordering::SeqCst);
    }

    /// Complete current build. Returns true if a pending build should start.
    pub fn complete(&self) -> bool {
        self.building.store(false, Ordering::SeqCst);
        self.pending.swap(false, Ordering::SeqCst)
    }
}

impl Default for BuildGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_guard_single_build() {
        let guard = BuildGuard::new();

        // First try_start should succeed
        assert!(guard.try_start());

        // Second try_start should fail (build in progress)
        assert!(!guard.try_start());

        // Complete without pending should return false
        assert!(!guard.complete());

        // After complete, try_start should succeed again
        assert!(guard.try_start());
        guard.complete();
    }

    #[test]
    fn test_build_guard_pending() {
        let guard = BuildGuard::new();

        // Start a build
        assert!(guard.try_start());

        // Try to start another (should fail)
        assert!(!guard.try_start());

        // Mark as pending
        guard.mark_pending();

        // Complete should return true (pending build)
        assert!(guard.complete());

        // Now try_start should succeed (for the pending build)
        assert!(guard.try_start());
        assert!(!guard.complete());
    }

    #[test]
    fn test_build_guard_multiple_pending() {
        let guard = BuildGuard::new();

        // Start a build
        assert!(guard.try_start());

        // Mark pending multiple times (only one pending should be stored)
        guard.mark_pending();
        guard.mark_pending();
        guard.mark_pending();

        // Complete should return true once
        assert!(guard.complete());

        // Second complete should return false (no more pending)
        assert!(!guard.complete());
    }
}
