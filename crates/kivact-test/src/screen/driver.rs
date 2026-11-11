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
use kivact_types::WindowId;

use super::model::{FIND_TIMEOUT, Record, Screen, TestApp};
use crate::RenderOptions;
use kivact_core::launch::{ClientMode, Inspect, Launch, TestTiming};
use kivact_core::view::View;
use kivact_style::asset::Asset;

/// Turns a settle may run before the App is declared never settling.
const SETTLE_LIMIT: usize = 100_000;

impl Screen {
    /// Mount `root` on the test driver with `options`, the App fonts
    /// `fonts` (as `App::font` passes them), and settle.
    #[track_caller]
    pub(crate) fn start(
        options: RenderOptions,
        fonts: Vec<(Asset, &'static Location<'static>)>,
        root: Box<dyn FnOnce() -> View>,
        at: &'static Location<'static>,
    ) -> Screen {
        let record = Rc::new(Record::default());
        let name = super::artifacts::test_name(at);
        let (app, host_options) = options.into_host(fonts);
        let timed = record.clone();
        let (client, launch) = Launch::new(
            app,
            ClientMode::Test {
                timing: TestTiming(Rc::new(move |elapsed| {
                    timed.core_ns.set(timed.core_ns.get() + elapsed);
                })),
                name: name.clone(),
            },
            root,
        )
        .into_engine_client();
        let host = ArvaneTestHost::new(client, host_options)
            .unwrap_or_else(|e| panic!("kivact test: the Arvane test host did not start: {e}"));
        let s = Screen {
            app: Rc::new(TestApp {
                host: RefCell::new(host),
                launch,
                record,
                actions: RefCell::new(Vec::new()),
                name,
                at,
                artifacts_written: kivact_types::cell::ReportOnce::new(),
                contacts: RefCell::new(Vec::new()),
                last_contact: Cell::new(0),
            }),
            window: WindowId::MAIN,
        };
        s.run_until_idle();
        s
    }

    /// Settle: run turns and wait (real time, ≤ 5 s) for blocking jobs.
    pub fn run_until_idle(&self) {
        self.app.run_until_idle(self.window);
    }

    /// Advance the virtual clock by `d`, stopping at every deadline on the
    /// way (frame ticks, timers), settling at each.
    pub fn advance(&self, d: Duration) {
        self.app.advance(self.window, d);
    }

    pub(crate) fn now_ns(&self) -> u64 {
        self.app.now_ns()
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

    /// Wait until the window's frames are drawn on the GPU, while its device
    /// starts on a worker thread ([`Renderer::GpuStarting`]): the host keeps
    /// running in real time, at most 5 s (the virtual clock does not move),
    /// then settles. Fails the test, with the failure report, when the device
    /// failed, did not start in time, or is not starting (the window draws
    /// on the CPU only, or nothing needed the GPU yet).
    #[track_caller]
    pub fn wait_for_gpu(&self) {
        self.log("wait for gpu".into());
        let waited = self.host_mut().wait_for_gpu(self.window);
        self.run_until_idle();
        if let Err(error) = waited {
            let report = self.report(&format!("wait_for_gpu: {error}"), None, &[]);
            self.fail(&report);
        }
    }

    /// One waiting step: to the next deadline, at most a display frame on.
    pub(super) fn step(&self) {
        let period = self.host().frame_period_ns(self.window);
        let now = self.now_ns();
        let next = self
            .app
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
        f(&mut self.app.host.borrow_mut())
    }

    pub(super) fn log(&self, s: String) {
        let mut a = self.app.actions.borrow_mut();
        if a.len() == 32 {
            a.remove(0);
        }
        a.push(s);
    }
}

impl TestApp {
    pub(super) fn inspect(&self) -> Inspect<'_> {
        self.launch
            .inspect()
            .expect("invariant: screen host started the app")
    }

    /// Work the core has without an event: a requested turn, ready tasks,
    /// queued effects.
    fn pending(&self) -> bool {
        self.inspect().pending()
    }

    /// Run turns until nothing is due (virtual time frozen). The host
    /// publishes the accessibility tree after each requested frame.
    fn pump(&self, window: WindowId) {
        for _ in 0..SETTLE_LIMIT {
            let mut host = self.host.borrow_mut();
            if let Some(e) = host.failure() {
                let e = e.to_string();
                drop(host);
                self.fail(window, &format!("the engine failed: {e}"));
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
            self.self_check(window);
            return;
        }
        self.fail(window, "the App never settles (turns keep being due)");
    }

    /// Debug check after every settle: the core's node records are the
    /// engine's live tree, and the engine's retained results (layout,
    /// paint list, text styles, edited lines, the published accessibility
    /// tree) equal a computation from scratch. A divergence fails at once.
    #[cfg(debug_assertions)]
    fn self_check(&self, window: WindowId) {
        if let Err(error) = self.inspect().check_tree() {
            self.fail(
                window,
                &format!("the core's node records and the engine's tree diverged: {error}"),
            );
        }
        let errors = self.host.borrow_mut().self_check();
        if !errors.is_empty() {
            self.fail(
                window,
                &format!(
                    "the engine's retained results diverged from a fresh computation:\n{}",
                    errors.join("\n")
                ),
            );
        }
    }

    /// Settle: run turns and wait (real time, ≤ 5 s) for blocking jobs.
    pub(super) fn run_until_idle(&self, window: WindowId) {
        self.pump(window);
        let start = Instant::now();
        while self.inspect().outstanding_blocking() > 0 || self.inspect().pending() {
            if start.elapsed() > FIND_TIMEOUT {
                break;
            }
            // The worker's wake unparks this thread (the driver's waker).
            if !self.host.borrow().woken() {
                std::thread::park_timeout(Duration::from_millis(5));
            }
            self.pump(window);
        }
        self.pump(window);
    }

    /// Advance the virtual clock by `d`, stopping at every deadline on the
    /// way (frame ticks, timers), settling at each.
    pub(super) fn advance(&self, window: WindowId, d: Duration) {
        let target = self
            .now_ns()
            .saturating_add(d.as_nanos().min(u64::MAX as u128) as u64);
        self.run_until_idle(window);
        loop {
            let next = self.host.borrow().next_deadline();
            match next {
                Some(t) if t <= target && t > self.now_ns() => {
                    self.host.borrow_mut().set_now(t);
                    self.run_until_idle(window);
                }
                _ => break,
            }
        }
        self.host.borrow_mut().set_now(target);
        self.run_until_idle(window);
    }

    pub(super) fn now_ns(&self) -> u64 {
        self.host.borrow().now_ns()
    }
}
