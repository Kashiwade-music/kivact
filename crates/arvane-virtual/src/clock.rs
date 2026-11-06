//! Role: Monotonic virtual clock shared by script and backend.

use std::cell::Cell;
#[derive(Default)]
pub(crate) struct Clock(Cell<u64>);
impl Clock {
    pub(crate) fn now(&self) -> u64 {
        self.0.get()
    }
    pub(crate) fn set(&self, ns: u64) {
        self.0.set(self.now().max(ns));
    }
}
