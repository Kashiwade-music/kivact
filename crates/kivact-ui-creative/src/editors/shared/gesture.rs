//! Role: One canvas press state and shared drag update, selection and finish order.
//!
//! Interaction Drag owns pointer identity, capture and edit commit/cancel. This
//! state owns only the domain's press decision and opaque drag snapshot.
use super::selection::merge_marquee;
use kivact::draw::{Point, Rect};
use kivact::interaction::{Drag, DragPhase, DragUpdate};
use kivact::view::{ButtonPress, Modifiers, MouseButton, PointerEvent, PointerPhase};
use kivact::{Signal, signal};
use std::rc::Rc;

#[derive(Clone)]
struct Pressed<T, D> {
    origin: Point,
    current: Point,
    target: Option<T>,
    payload: D,
    pending: Option<u64>,
    modifiers: Modifiers,
    double: bool,
    moved: bool,
}

pub(crate) struct CanvasGesture<T: Copy + 'static, D: Clone + 'static> {
    state: Signal<Option<Pressed<T, D>>>,
}
impl<T: Copy + 'static, D: Clone + 'static> Copy for CanvasGesture<T, D> {}
impl<T: Copy + 'static, D: Clone + 'static> Clone for CanvasGesture<T, D> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Copy + 'static, D: Clone + 'static> CanvasGesture<T, D> {
    pub(crate) fn new() -> Self {
        Self {
            state: signal(None),
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
                origin: event.pos(),
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
    pub(crate) fn origin(self) -> Point {
        self.state.with(|state| {
            state
                .as_ref()
                .map_or(Point::new(0., 0.), |state| state.origin)
        })
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
    pub(crate) fn update(self, pos: Point, motion: impl FnOnce(Point)) {
        self.state.update(|state| {
            if let Some(state) = state {
                state.current = pos;
                state.moved |= pos != state.origin;
            }
        });
        motion(pos);
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
        update: &DragUpdate<()>,
        marquee: impl Fn() -> bool,
        selection: &(impl Fn() -> Vec<u64>, impl Fn(Vec<u64>), impl Fn(u64)),
        callbacks: &(
            impl Fn(Point),
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
        self.update(update.pos, &callbacks.0);
        let activation = self.state.with(|state| {
            state
                .as_ref()
                .filter(|state| state.double && !state.moved)
                .and_then(|state| state.target)
        });
        if marquee() {
            let hit = callbacks.1(Rect::from_points(self.origin(), self.current()));
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
        drag: Drag<()>,
        marquee: impl Fn() -> bool + 'static,
        selection: (
            impl Fn() -> Vec<u64> + 'static,
            impl Fn(Vec<u64>) + 'static,
            impl Fn(u64) + 'static,
        ),
        callbacks: (
            impl Fn(Point) + 'static,
            impl Fn(Rect) -> Vec<u64> + 'static,
            impl Fn(T) + 'static,
            impl Fn(bool) + 'static,
        ),
    ) -> Drag<()> {
        let hooks = Rc::new((marquee, selection, callbacks));
        let updates = hooks.clone();
        drag.on_update(move |update: DragUpdate<()>| self.update(update.pos, &updates.2.0))
            .on_end(move |update: DragUpdate<()>| {
                if update.phase == DragPhase::End {
                    self.finish(&update, &hooks.0, &hooks.1, &hooks.2);
                } else {
                    self.cancel(|| hooks.2.3(false));
                }
            })
    }
}
