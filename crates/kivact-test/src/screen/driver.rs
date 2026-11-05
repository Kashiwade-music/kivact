//! Role: Test window driver and event automation.
//!
//! Lifecycle and virtual time of a `Screen`: mounting on the Arvane test
//! driver, settling turns and requested redraws, advancing the virtual
//! clock, and waiting for conditions in virtual time.

use std::cell::{Cell, RefCell};
use std::panic::Location;
use std::rc::Rc;
use std::time::{Duration, Instant};

use kivact_engine::testing::ArvaneTestHost;

use super::model::{FIND_TIMEOUT, Record, Screen};
use crate::RenderOptions;
use kivact_core::launch::{ClientMode, Launch, TestTiming};
use kivact_core::view::View;
use kivact_style::asset::Asset;

/// Turns a settle may run before the App is declared never settling.
const SETTLE_LIMIT: usize = 100_000;

impl Screen {
    /// Mount `root` on the test driver with `options`, the App fonts
    /// `fonts` (as `App::font` passes them), drawing on the GPU when `gpu`
    /// (the GPU smoke test), and settle.
    #[track_caller]
    pub(crate) fn start(
        options: RenderOptions,
        gpu: bool,
        fonts: Vec<(Asset, &'static Location<'static>)>,
        root: Box<dyn FnOnce() -> View>,
        at: &'static Location<'static>,
    ) -> Screen {
        let record = Rc::new(Record::default());
        let name = super::artifacts::test_name(at);
        let (app, host_options) = options.into_host(gpu, fonts);
        let scale = host_options.scale;
        let timed = record.clone();
        let (client, launch) = Launch::new(
            app,
            ClientMode::Test(TestTiming(Rc::new(move |elapsed| {
                timed.core_ns.set(timed.core_ns.get() + elapsed);
            }))),
            root,
        )
        .into_engine_client();
        let host = ArvaneTestHost::new(client, host_options)
            .unwrap_or_else(|e| panic!("kivact test: the Arvane test host did not start: {e}"));
        let s = Screen {
            host: RefCell::new(host),
            launch,
            record,
            actions: RefCell::new(Vec::new()),
            name,
            at,
            artifacts_written: Cell::new(false),
            scale: Cell::new(scale),
            contacts: RefCell::new(Vec::new()),
            last_contact: Cell::new(0),
        };
        #[cfg(debug_assertions)]
        s.inspect().open_trace(&s.name);
        s.run_until_idle();
        s
    }

    /// Work the core has without an event: a requested turn, ready tasks,
    /// queued effects.
    fn pending(&self) -> bool {
        self.inspect().pending()
    }

    /// Run turns until nothing is due (virtual time frozen). The host
    /// publishes the accessibility tree after each requested frame.
    fn pump(&self) {
        for _ in 0..SETTLE_LIMIT {
            let mut host = self.host.borrow_mut();
            if let Some(e) = host.failure() {
                let e = e.to_string();
                drop(host);
                self.fail(&format!("the engine failed: {e}"));
            }
            if host.exited() {
                return;
            }
            if host.due() || host.woken() || self.pending() {
                host.wake();
                continue;
            }
            if host.redraw_requested() {
                host.redraw();
                continue;
            }
            drop(host);
            #[cfg(debug_assertions)]
            self.self_check();
            return;
        }
        self.fail("the App never settles (turns keep being due)");
    }

    /// Debug check after every settle: the core's node records are the
    /// engine's live tree. A divergence fails at once.
    #[cfg(debug_assertions)]
    fn self_check(&self) {
        if let Err(error) = self.inspect().check_tree() {
            self.fail(&format!(
                "the core's node records and the engine's tree diverged: {error}"
            ));
        }
    }

    /// Settle: run turns and wait (real time, ≤ 5 s) for blocking jobs.
    pub fn run_until_idle(&self) {
        self.pump();
        let start = Instant::now();
        while self.inspect().outstanding_blocking() > 0 || self.inspect().pending() {
            if start.elapsed() > FIND_TIMEOUT {
                break;
            }
            // The worker's wake unparks this thread (the driver's waker).
            if !self.host.borrow().woken() {
                std::thread::park_timeout(Duration::from_millis(5));
            }
            self.pump();
        }
        self.pump();
    }

    /// Advance the virtual clock by `d`, stopping at every deadline on the
    /// way (frame ticks, timers), settling at each.
    pub fn advance(&self, d: Duration) {
        let target = self
            .now_ns()
            .saturating_add(d.as_nanos().min(u64::MAX as u128) as u64);
        self.run_until_idle();
        loop {
            let next = self.host.borrow().next_deadline();
            match next {
                Some(t) if t <= target && t > self.now_ns() => {
                    self.host.borrow_mut().set_now(t);
                    self.run_until_idle();
                }
                _ => break,
            }
        }
        self.host.borrow_mut().set_now(target);
        self.run_until_idle();
    }

    pub(crate) fn now_ns(&self) -> u64 {
        self.host.borrow().now_ns()
    }

    /// Virtual time since start.
    pub fn now(&self) -> Duration {
        Duration::from_nanos(self.now_ns())
    }

    /// Advance virtual time a display frame at a time until `cond` holds,
    /// for at most `timeout` (virtual); fails the test with the failure
    /// report if it never does. Returns the virtual time waited.
    #[track_caller]
    pub fn wait_for(&self, timeout: Duration, mut cond: impl FnMut(&Screen) -> bool) -> Duration {
        let start = self.now_ns();
        let limit = timeout.as_nanos().min(u64::MAX as u128) as u64;
        loop {
            self.run_until_idle();
            if cond(self) {
                return Duration::from_nanos(self.now_ns() - start);
            }
            if self.now_ns().saturating_sub(start) >= limit {
                let report = self.report(
                    &format!("wait_for: the condition did not hold within {timeout:?} (virtual)"),
                    None,
                    &[],
                );
                self.fail(&report);
            }
            self.step();
        }
    }

    /// One waiting step: to the next deadline, at most a display frame on.
    pub(super) fn step(&self) {
        let period = self.host.borrow().frame_period_ns();
        let now = self.now_ns();
        let next = self
            .host
            .borrow()
            .next_deadline()
            .filter(|t| *t > now)
            .unwrap_or(u64::MAX);
        self.advance(Duration::from_nanos(next.min(now + period) - now));
    }

    /// The test driver (internal tests).
    #[cfg(test)]
    pub(crate) fn with_host<R>(&self, f: impl FnOnce(&mut ArvaneTestHost) -> R) -> R {
        f(&mut self.host.borrow_mut())
    }

    pub(super) fn log(&self, s: String) {
        let mut a = self.actions.borrow_mut();
        if a.len() == 32 {
            a.remove(0);
        }
        a.push(s);
    }
}
