//! Role: This module stores pending host-clock timers.

use std::collections::BTreeMap;
use std::task::Waker;

#[derive(Default)]
pub(crate) struct TimeState {
    pub(crate) timers: BTreeMap<(u64, u64), Waker>,
    pub(crate) next_id: u64,
}

impl TimeState {
    pub(crate) fn next_deadline(&self) -> Option<u64> {
        self.timers.keys().next().map(|k| k.0)
    }

    pub(crate) fn len(&self) -> usize {
        self.timers.len()
    }

    /// Remove and return wakers of timers due at `now` (ns).
    pub(crate) fn take_due(&mut self, now: u64) -> Vec<Waker> {
        let due: Vec<(u64, u64)> = self.timers.range(..(now + 1, 0)).map(|(k, _)| *k).collect();
        due.into_iter()
            .filter_map(|k| self.timers.remove(&k))
            .collect()
    }
}
