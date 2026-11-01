//! Role: Two small apps from different domains built on `kivact-external-component`.
//! They are not renames of
//! the example apps:
//!
//! * [`log_viewer`] — two log sources side by side. Each pane has a search
//!   box (IME), a level filter, a timeline of matching lines per 5 minutes
//!   (drag to restrict the time range) and a lazy list of the lines. The
//!   same three components appear twice on one screen.
//! * [`device_monitor`] — a live status board of 1,000 sensors: search,
//!   "alarms only" toggle, lazy list with status chips and sparklines (the
//!   timeline's drawing function in a local draw layer), and a detail view
//!   with two timelines (temperature, voltage) whose selected ranges give
//!   statistics. It switches between the Business and Console skins
//!   (style/skin invariance).
//!
//! Both only use `kivact::prelude`, `kivact::view`/`task`/`style` public items
//! and the external components.
#![warn(redundant_imports)]

pub mod device_monitor;
pub mod log_viewer;

/// Small deterministic generator for sample data (xorshift64*).
#[derive(Clone, Copy, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.max(1))
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// Uniform in `0..n`.
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n.max(1)
    }

    /// Uniform in `[0, 1)`.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}
