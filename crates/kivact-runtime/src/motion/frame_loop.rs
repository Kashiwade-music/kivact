//! Role: Frame loop callbacks and refresh timing.
//!
//! `FrameLoop`: a per-frame callback at a target rate, driven by host
//! `FrameTick`s requested with `RequestFrame` (no fixed 16 ms timer).

use std::cell::Cell;
use std::panic::Location;
use std::rc::Rc;

use super::state::{LoopCell, LoopRun, LoopState, MotionState};
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
    let cell = Rc::new(LoopCell {
        owner,
        f: crate::reactive::Handler::from_fn(f),
        state: owner.signal(LoopState::Stopped, at),
        frames: Cell::new(0),
    });
    MotionState::of(&rt).register_loop(&cell);
    FrameLoop {
        handle: owner.store("FrameLoop", cell, at),
    }
}

impl FrameLoop {
    fn cell(&self, at: &'static Location<'static>) -> (Rc<Runtime>, Rc<LoopCell>) {
        let c = self.handle.get();
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
        let now = rt.now_ns();
        // A new rate counts from now; the time of the last callback stays
        // (`FrameInfo::delta`).
        cell.set_run(Some(LoopRun {
            rate,
            start_ns: now,
            index: 0,
            last_tick: current.and_then(|r| r.last_tick),
        }));
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
        {
            let cell = self.cell(Location::caller()).1;
            let state = cell.state;
            cell.owner.with_setup(|| {
                crate::memo(move || match state.get() {
                    LoopState::Stopped => None,
                    LoopState::Running(run) => Some(run.rate),
                })
            })
        }
    }
}
