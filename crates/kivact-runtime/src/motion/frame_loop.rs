//! Role: Frame loop callbacks and refresh timing.
//!
//! `FrameLoop`: a per-frame callback at a target rate, driven by host
//! `FrameTick`s requested with `RequestFrame` (no fixed 16 ms timer).

use std::cell::Cell;
use std::panic::Location;
use std::rc::Rc;

use super::state::{
    Cadence, LoopCell, LoopGate, LoopMotion, LoopRun, LoopState, MotionState, motion_now,
};
use super::values::{FrameInfo, FrameRate};

use crate::{ErrorKind, HandlerResult, Owned, Owner, Runtime, UiError, Value, require_effectful};

/// Copy handle of a frame loop.
#[derive(Clone, Copy)]
pub struct FrameLoop {
    handle: Owned<Rc<LoopCell>>,
}

/// A frame callback of the current setup owner, stopped until `start`
/// and released with the owner. `spawn` and `context` inside `f` use
/// that owner. `f` may return `()` or `Result<(), UiError>`; an `Err` goes to
/// the nearest error handler. Setup only.
#[track_caller]
pub fn frame_loop<R: HandlerResult>(f: impl Fn(FrameInfo) -> R + 'static) -> FrameLoop {
    let at = Location::caller();
    let owner = Owner::current("frame_loop", at);
    let rt = owner.runtime();
    let state = owner.signal(LoopState::Stopped, at);
    let rate = crate::memo(move || match state.get() {
        LoopState::Stopped => None,
        LoopState::Running(run) => Some(run.rate),
    });
    let cell = Rc::new(LoopCell {
        owner,
        f: crate::reactive::Handler::from_fn(f),
        state,
        frames: Cell::new(0),
        gate: Cell::new(LoopGate::Always),
        rate,
        motion: Cell::new(LoopMotion::Decorative),
    });
    MotionState::of(&rt).register_loop(&cell);
    FrameLoop {
        handle: owner.store("FrameLoop", cell, at),
    }
}

impl FrameLoop {
    /// Keep this loop running under reduced motion. A loop is decorative by
    /// default: under reduced motion it requests no frame and gets no
    /// callback, and it counts again from the moment reduced motion is off.
    /// Setup only.
    #[track_caller]
    pub fn essential(self) -> Self {
        crate::require_setup("FrameLoop::essential");
        let (_rt, cell) = self.cell(Location::caller());
        cell.motion.set(LoopMotion::Essential);
        self
    }

    /// Host adapter bridge: restrict scheduling to an observed node lifetime.
    #[doc(hidden)]
    #[track_caller]
    pub fn with_presence_gate(self, gate: Value<bool>) -> Self {
        crate::require_setup("FrameLoop::with_presence_gate");
        let (rt, cell) = self.cell(Location::caller());
        cell.gate.set(LoopGate::While(gate));
        let app = rt.id();
        cell.owner.with_setup(move || {
            crate::effect(move || {
                let _ = gate.get();
                if let Some(rt) = Runtime::get(app) {
                    rt.request_turn();
                }
            })
        });
        self
    }

    fn cell(&self, at: &'static Location<'static>) -> (Rc<Runtime>, Rc<LoopCell>) {
        let c = self.handle.get_at(at);
        (c.owner.runtime(), c)
    }

    /// Start (or change the rate of) the loop.
    #[track_caller]
    pub fn start(&self, rate: FrameRate) {
        let at = Location::caller();
        let (rt, cell) = self.cell(at);
        // Checked before any state changes.
        require_effectful("FrameLoop::start");
        if let FrameRate::Hz(hz) = rate
            && !(hz.is_finite() && hz > 0.0)
        {
            rt.report(
                Some(cell.owner),
                UiError::new(
                    ErrorKind::Message,
                    format!("invalid frame rate {hz}"),
                    Some(at),
                ),
            );
            return;
        }
        let current = match cell.state.get_untracked() {
            LoopState::Stopped => None,
            LoopState::Running(run) => Some(run),
        };
        if current.is_some_and(|r| r.rate == rate) {
            return;
        }
        let now = motion_now(&rt);
        // A new rate counts from now; the time of the last callback stays
        // (`FrameInfo::delta`).
        let cadence = match current.and_then(|r| r.last_tick()) {
            Some(last_tick) => Cadence::Ticking {
                start_ns: now,
                index: 0,
                last_tick,
            },
            None => Cadence::Fresh { start_ns: now },
        };
        cell.set_run(Some(LoopRun { rate, cadence }));
        rt.request_turn();
    }

    /// Stop the loop (no more callbacks until the next `start`).
    #[track_caller]
    pub fn stop(&self) {
        let at = Location::caller();
        let (rt, cell) = self.cell(at);
        require_effectful("FrameLoop::stop");
        cell.set_run(None);
        drop(rt);
    }

    /// Current rate (`None` when stopped).
    #[track_caller]
    pub fn rate(&self) -> Value<Option<FrameRate>> {
        self.cell(Location::caller()).1.rate
    }
}
