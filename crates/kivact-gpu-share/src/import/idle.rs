//! Role: The longest an import waits for the device's queue to finish.

use std::time::Duration;

/// The longest the queue is waited for when an import is prepared or
/// dropped.
pub(super) const IDLE: Duration = Duration::from_secs(5);
