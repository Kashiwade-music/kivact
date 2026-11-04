//! Role: Runtime clock and wake scheduling.
use crate::reactive::data::Runtime;
use crate::reactive::host::TurnState;
impl Runtime {
    /// The App's monotonic clock in nanoseconds.
    pub fn now_ns(&self) -> u64 {
        self.clock.get()
    }
    /// Advance the App's clock without moving it backwards.
    pub fn advance_clock(&self, ns: u64) {
        self.clock.set(self.clock.get().max(ns));
    }
    /// Wake the host once when a quiet App gains pending work.
    pub fn request_turn(&self) {
        if self.state.get() == TurnState::Quiet {
            self.state.set(TurnState::WakeSent);
            self.host.wake();
        }
    }
}
/// The clock of the executing App, or zero outside an App frame.
pub fn clock_now() -> u64 {
    crate::reactive::registry::current_runtime().map_or(0, |rt| rt.now_ns())
}
