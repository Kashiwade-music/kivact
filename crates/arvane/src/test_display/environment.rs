//! Role: What this test process offers: display counters and GPU availability.

use crate::FailureKind;
use crate::env::Env;
use crate::presentation::Notes;
use crate::presenter::Presenter;
use arvane_native::test_display::TestDisplay;
use arvane_native::test_display::TestDisplayStats;
use kivact_types::GpuPreference;

/// Counters of the process-wide test display, if it was started.
pub fn display_stats() -> Option<TestDisplayStats> {
    TestDisplay::global().ok().map(TestDisplay::stats)
}

/// Why the GPU renderer cannot start here (no hardware adapter, or GPU
/// support not built in), or `None` when it can: a GPU test then fails
/// loudly on any other GPU failure instead of skipping.
pub fn gpu_unavailable() -> Option<String> {
    match Presenter::new(Notes::default(), &Env::read(GpuPreference::default())).enable_gpu() {
        Err(error) if error.kind == FailureKind::Unavailable => Some(error.to_string()),
        _ => None,
    }
}
