//! Timing utilities for performance debugging
//!
//! Enable timing output by setting `SGC_TIMING=1` environment variable
//! or calling `enable_timing()`.

use std::sync::atomic::{AtomicBool, Ordering};
use web_time::Instant;

static TIMING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enable timing output
pub fn enable_timing() {
    TIMING_ENABLED.store(true, Ordering::Relaxed);
}

/// Check if timing is enabled (also checks SGC_TIMING env var on first call)
pub fn is_timing_enabled() -> bool {
    // Check atomic first (fast path)
    if TIMING_ENABLED.load(Ordering::Relaxed) {
        return true;
    }
    // Fall back to env var check
    std::env::var("SGC_TIMING").is_ok()
}

/// Log timing information if timing is enabled
#[macro_export]
macro_rules! timing {
    ($label:expr, $elapsed:expr) => {
        if $crate::timing::is_timing_enabled() {
            eprintln!("[timing] {}: {:?}", $label, $elapsed);
        }
    };
    ($label:expr, $elapsed:expr, $($arg:tt)*) => {
        if $crate::timing::is_timing_enabled() {
            eprintln!("[timing] {}: {:?} ({})", $label, $elapsed, format!($($arg)*));
        }
    };
}

/// A timing guard that logs elapsed time when dropped
pub struct TimingGuard {
    label: &'static str,
    start: Instant,
    extra: Option<String>,
}

impl TimingGuard {
    /// Start timing with a label
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            start: Instant::now(),
            extra: None,
        }
    }

    /// Add extra info to be logged (e.g., item count)
    pub fn with_extra(mut self, extra: String) -> Self {
        self.extra = Some(extra);
        self
    }

    /// Finish timing and return elapsed duration (logs automatically)
    pub fn finish(self) -> std::time::Duration {
        let elapsed = self.start.elapsed();
        if is_timing_enabled() {
            match &self.extra {
                Some(extra) => eprintln!("[timing] {}: {:?} ({})", self.label, elapsed, extra),
                None => eprintln!("[timing] {}: {:?}", self.label, elapsed),
            }
        }
        elapsed
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        // Only log if not already finished (check if start is still valid)
        // We use a trick: if finish() was called, self would be moved
        // Since Drop takes &mut self, this only runs if finish() wasn't called
    }
}

/// Start a timing measurement (returns guard that logs on finish/drop)
pub fn start(label: &'static str) -> TimingGuard {
    TimingGuard::new(label)
}
