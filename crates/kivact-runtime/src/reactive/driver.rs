//! Role: Runtime link to the driver and turn scheduling.
//!
//! The runtime's link to what drives its turns: the App's clock and the
//! request for a turn. The driver (the client, which owns the host clock
//! and the host waker) installs itself once; the runtime knows neither
//! hosts nor wakers.
//!
//! Invariants:
//! - Without a driver the clock reads 0 and a turn request wakes nobody (a
//!   runtime used on its own, as in this crate's unit tests).
//! - A turn is requested of the driver at most once until the driver
//!   starts it ([`Runtime::clear_turn_request`]). The flag is the
//!   runtime's, so a write that queues an effect costs one `Cell` read
//!   when a turn is already requested.

use std::rc::Rc;

use crate::reactive::data::{Driver, Runtime};

impl Runtime {
    /// Install the driver of this runtime (once, when the App is created).
    pub(crate) fn set_driver(&self, driver: Rc<dyn Driver>) {
        let installed = self.driver.set(driver).is_ok();
        assert!(installed, "invariant: a runtime has one driver");
    }

    /// The App's clock in nanoseconds (0 without a driver).
    #[inline]
    pub fn now_ns(&self) -> u64 {
        self.driver.get().map_or(0, |d| d.now_ns())
    }

    /// Ask the driver for a turn (once until the turn starts).
    #[inline]
    pub fn request_turn(&self) {
        if !self.turn_requested.replace(true) {
            self.wake_driver();
        }
    }

    #[cold]
    #[inline(never)]
    fn wake_driver(&self) {
        if let Some(d) = self.driver.get() {
            d.request_turn();
        }
    }

    /// Whether a turn was requested and has not started yet.
    pub(crate) fn turn_requested(&self) -> bool {
        self.turn_requested.get()
    }

    /// Forget the turn request: the requested turn is starting.
    pub fn clear_turn_request(&self) {
        self.turn_requested.set(false);
    }
}

/// The clock of the App whose code is running, in nanoseconds; 0 outside
/// every App frame. It never goes back.
pub fn clock_now() -> u64 {
    crate::reactive::registry::current_runtime().map_or(0, |rt| rt.now_ns())
}
