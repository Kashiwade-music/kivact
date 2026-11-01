//! Role: Probe the test display: GPU availability and display statistics.

use arvane::test_display::TestDisplayStats;

/// Why the GPU renderer cannot start here (no hardware adapter, or GPU
/// support not built in), or `None` when it can: a GPU test then fails
/// loudly on any other GPU failure instead of skipping.
pub fn gpu_unavailable() -> Option<String> {
    arvane::test_display::gpu_unavailable()
}

/// Counters of the process-wide test display, if it was started.
pub fn display_stats() -> Option<TestDisplayStats> {
    arvane::test_display::display_stats()
}
