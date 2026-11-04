//! Role: One canvas press state and shared drag update, selection and finish order.
//!
//! Interaction DragApi owns pointer identity, capture and edit commit/cancel. This
//! state owns only the domain's press decision and opaque drag snapshot.
use super::selection::merge_marquee;
use kivact::geometry::{Point, Rect};
use kivact::interaction::{DragApi, DragPhase, DragUpdate};
use kivact::state::{Signal, Value, signal};
use kivact::view::{ButtonPress, Modifiers, MouseButton, PointerEvent, PointerPhase};
use std::rc::Rc;

#[derive(Clone)]
struct Pressed<T, D> {
    current: Point,
    target: Option<T>,
    payload: D,
    pending: Option<u64>,
    modifiers: Modifiers,
    double: bool,
    moved: bool,
}

pub(crate) struct CanvasGesture<T: Copy + 'static, D: Clone + 'static, S: Clone + PartialEq + 'static = ()> {
    state: Signal<Option<Pressed<T, D>>>,
    snapshot: Value<Option<S>>,
}
impl<T: Copy + 'static, D: Clone + 'static, S: Clone + PartialEq + 'static> Copy for CanvasGesture<T, D, S> {}
impl<T: Copy + 'static, D: Clone + 'static, S: Clone + PartialEq + 'static> Clone for CanvasGesture<T, D, S> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Copy + 'static, D: Clone + 'static, S: Clone + PartialEq + 'static> CanvasGesture<T, D, S> {
    pub(crate) fn new(drag: DragApi<S>) -> Self {
        Self {
            state: signal(None),
            snapshot: drag.active_payload(),
        }
    }
    pub(crate) fn begin(
        self,
        event: &PointerEvent,
        target: Option<T>,
        payload: D,
        pending: Option<u64>,
    ) {
        self.state.update(|state| {
            *state = Some(Pressed {
                current: event.pos(),
                target,
                payload,
                pending,
                modifiers: event.modifiers(),
                double: matches!(
                    event.phase(),
                    PointerPhase::Down(ButtonPress {
                        button: MouseButton::Primary,
                        clicks: 2
                    })
                ),
                moved: false,
            })
        });
    }
    pub(crate) fn with_payload<R>(self, read: impl FnOnce(&D) -> R) -> Option<R> {
        self.state
            .with(|state| state.as_ref().map(|state| read(&state.payload)))
    }
    pub(crate) fn update_payload(self, change: impl FnOnce(&mut D)) {
        self.state.update(|state| {
            if let Some(state) = state {
                change(&mut state.payload);
            }
        });
    }
    pub(crate) fn pending(self) -> Option<u64> {
        self.state
            .with(|state| state.as_ref().and_then(|state| state.pending))
    }
    pub(crate) fn set_pending(self, pending: Option<u64>) {
        self.state.update(|state| {
            if let Some(state) = state {
                state.pending = pending;
            }
        });
    }
    pub(crate) fn initial(self) -> Option<S> {
        self.snapshot.get()
    }
    pub(crate) fn current(self) -> Point {
        self.state.with(|state| {
            state
                .as_ref()
                .map_or(Point::new(0., 0.), |state| state.current)
        })
    }
    pub(crate) fn pressed(self) -> Option<T> {
        self.state
            .with(|state| state.as_ref().and_then(|state| state.target))
    }
    pub(crate) fn moved(self) -> bool {
        self.state
            .with(|state| state.as_ref().is_some_and(|state| state.moved))
    }
    pub(crate) fn update(self, pos: Point, origin: Point, motion: impl FnOnce(Point, Point)) {
        self.state.update(|state| {
            if let Some(state) = state {
                state.current = pos;
                state.moved |= pos != origin;
            }
        });
        motion(pos, origin);
    }
    fn clear(self) {
        self.state.update(|state| *state = None);
    }
    pub(crate) fn cancel(self, reset: impl FnOnce()) {
        reset();
        self.clear();
    }
    pub(crate) fn finish(
        self,
        update: &DragUpdate<S>,
        marquee: impl Fn() -> bool,
        selection: &(impl Fn() -> Vec<u64>, impl Fn(Vec<u64>), impl Fn(u64)),
        callbacks: &(
            impl Fn(Point, Point),
            impl Fn(Rect) -> Vec<u64>,
            impl Fn(T),
            impl Fn(bool),
        ),
    ) {
        self.state.update(|state| {
            if let Some(state) = state {
                state.modifiers = update.modifiers;
            }
        });
        self.update(update.pos, update.start_pos, &callbacks.0);
        let activation = self.state.with(|state| {
            state
                .as_ref()
                .filter(|state| state.double && !state.moved)
                .and_then(|state| state.target)
        });
        if marquee() {
            let hit = callbacks.1(Rect::from_points(update.start_pos, self.current()));
            let modifiers = self.state.with(|state| {
                state
                    .as_ref()
                    .map_or(Modifiers::default(), |state| state.modifiers)
            });
            selection.1(merge_marquee(&selection.0(), hit, modifiers));
        }
        if !self.moved()
            && let Some(id) = self.pending()
        {
            selection.2(id);
        }
        callbacks.3(true);
        self.clear();
        if let Some(target) = activation {
            callbacks.2(target);
        }
    }
    /// Every domain keeps its own on_start, coordinates and payload operations.
    /// `reset(true)` finalizes domain-only work; `reset(false)` cancels it.
    pub(crate) fn install(
        self,
        drag: DragApi<S>,
        marquee: impl Fn() -> bool + 'static,
        selection: (
            impl Fn() -> Vec<u64> + 'static,
            impl Fn(Vec<u64>) + 'static,
            impl Fn(u64) + 'static,
        ),
        callbacks: (
            impl Fn(Point, Point) + 'static,
            impl Fn(Rect) -> Vec<u64> + 'static,
            impl Fn(T) + 'static,
            impl Fn(bool) + 'static,
        ),
    ) -> DragApi<S> {
        let hooks = Rc::new((marquee, selection, callbacks));
        let updates = hooks.clone();
        drag.on_update(move |update: DragUpdate<S>| self.update(update.pos, update.start_pos, &updates.2.0))
            .on_end(move |update: DragUpdate<S>| {
                if update.phase == DragPhase::End {
                    self.finish(&update, &hooks.0, &hooks.1, &hooks.2);
                } else {
                    self.cancel(|| hooks.2.3(false));
                }
            })
    }
}
