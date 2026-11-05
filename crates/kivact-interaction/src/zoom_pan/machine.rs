//! Role: Own zoom and pan contacts and the edit attached to each active gesture.
use super::data::{Finger, Pan, Pinch, ViewTransform, ZoomPanOptions, finite, panned, pinched};
use crate::EditSession;
use kivact_core::geometry::Point;
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::{Handled, MouseButton, PointerEvent, PointerId, PointerKind, PointerPhase};
use std::rc::Rc;
#[derive(Clone)]
pub(super) enum Gesture {
    Idle {
        touches: Vec<(PointerId, Point)>,
    },
    Pan {
        pan: Pan,
        touches: Vec<(PointerId, Point)>,
        session: Rc<EditSession<ViewTransform>>,
    },
    Pinch {
        pinch: Pinch,
        touches: Vec<(PointerId, Point)>,
        session: Rc<EditSession<ViewTransform>>,
    },
}
impl PartialEq for Gesture {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Idle { touches: a }, Self::Idle { touches: b }) => a == b,
            (
                Self::Pan {
                    pan: a,
                    touches: at,
                    session: ae,
                },
                Self::Pan {
                    pan: b,
                    touches: bt,
                    session: be,
                },
            ) => a == b && at == bt && Rc::ptr_eq(ae, be),
            (
                Self::Pinch {
                    pinch: a,
                    touches: at,
                    session: ae,
                },
                Self::Pinch {
                    pinch: b,
                    touches: bt,
                    session: be,
                },
            ) => a == b && at == bt && Rc::ptr_eq(ae, be),
            _ => false,
        }
    }
}
impl Gesture {
    pub(super) fn active(&self) -> bool {
        !matches!(self, Self::Idle { .. })
    }
    fn touches(&mut self) -> &mut Vec<(PointerId, Point)> {
        match self {
            Self::Idle { touches } | Self::Pan { touches, .. } | Self::Pinch { touches, .. } => {
                touches
            }
        }
    }
    fn session(&self) -> Option<Rc<EditSession<ViewTransform>>> {
        match self {
            Self::Idle { .. } => None,
            Self::Pan { session, .. } | Self::Pinch { session, .. } => Some(session.clone()),
        }
    }
}
pub(super) type State = Gesture;
pub(super) enum Event {
    Pointer(PointerEvent),
    Stop(bool),
    Available(bool),
}
pub(super) enum Effect {
    Begin(Rc<EditSession<ViewTransform>>, ViewTransform),
    Write(ViewTransform),
    Finish(Rc<EditSession<ViewTransform>>, bool),
}
pub(super) struct Machine {
    pub options: ZoomPanOptions,
    pub transform: ViewTransform,
    pub armed: bool,
    pub session: Rc<EditSession<ViewTransform>>,
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match event {
            Event::Pointer(event) => self.pointer(state, event).0,
            Event::Stop(commit) => self.stop(state, commit, None),
            Event::Available(false) => self.stop(state, false, None),
            Event::Available(true) => Step::stay(),
        }
    }
}
impl Machine {
    fn stop(
        &self,
        state: &State,
        commit: bool,
        write: Option<ViewTransform>,
    ) -> Step<State, Event, Effect> {
        let Some(session) = state.session() else {
            return Step::stay();
        };
        let mut snapshot = state.clone();
        let touches = std::mem::take(snapshot.touches());
        let mut step = Step::to(State::Idle { touches });
        if let Some(value) = write {
            step = step.with(Effect::Write(value));
        }
        step.with(Effect::Finish(session, commit))
    }
    fn pinch(&self, first: kivact_core::view::PointerId, at: Point, event: PointerEvent) -> Pinch {
        Pinch {
            fingers: [
                Finger {
                    pointer: first,
                    start: at,
                    now: at,
                },
                Finger {
                    pointer: event.id(),
                    start: event.pos(),
                    now: event.pos(),
                },
            ],
            transform: self.transform,
        }
    }
    fn touch_pointer(
        &self,
        state: &State,
        next: &mut State,
        event: PointerEvent,
        valid: bool,
    ) -> Option<(Step<State, Event, Effect>, Handled)> {
        match event.phase() {
            PointerPhase::Down(_) => {
                if valid {
                    next.touches().push((event.id(), event.pos()));
                }
                let others: Vec<_> = next
                    .touches()
                    .iter()
                    .copied()
                    .filter(|(id, _)| *id != event.id())
                    .collect();
                if valid && others.len() == 1 && !state.active() {
                    let (first, at) = others[0];
                    let pinch = self.pinch(first, at, event);
                    let touches = std::mem::take(next.touches());
                    return Some((
                        Step::to(State::Pinch {
                            pinch,
                            touches,
                            session: self.session.clone(),
                        })
                        .with(Effect::Begin(self.session.clone(), self.transform)),
                        Handled::Yes,
                    ));
                }
            }
            PointerPhase::Move => {
                if valid
                    && let Some(touch) = next.touches().iter_mut().find(|(id, _)| *id == event.id())
                {
                    touch.1 = event.pos();
                }
                if let State::Pinch { pinch, .. } = &mut *next
                    && let Some(finger) = pinch
                        .fingers
                        .iter_mut()
                        .find(|finger| finger.pointer == event.id())
                {
                    if valid {
                        finger.now = event.pos();
                        let value = pinched(*pinch, self.options);
                        return Some((
                            Step::to(next.clone()).with(Effect::Write(value)),
                            Handled::Yes,
                        ));
                    }
                    return Some((Step::to(next.clone()), Handled::Yes));
                }
            }
            PointerPhase::Up(_) | PointerPhase::Cancel => {
                next.touches().retain(|(id, _)| *id != event.id());
                if let State::Pinch { pinch, .. } = &mut *next
                    && let Some(finger) = pinch
                        .fingers
                        .iter_mut()
                        .find(|finger| finger.pointer == event.id())
                {
                    let commit = matches!(event.phase(), PointerPhase::Up(_));
                    let write = if commit && valid {
                        finger.now = event.pos();
                        Some(pinched(*pinch, self.options))
                    } else {
                        None
                    };
                    return Some((self.stop(next, commit, write), Handled::Yes));
                }
            }
            PointerPhase::Enter | PointerPhase::Leave => {}
        }
        None
    }
    pub(super) fn pointer(
        &self,
        state: &State,
        event: PointerEvent,
    ) -> (Step<State, Event, Effect>, Handled) {
        let mut next = state.clone();
        let valid = finite(event.pos());
        if event.kind() == PointerKind::Touch
            && let Some(step) = self.touch_pointer(state, &mut next, event, valid)
        {
            return step;
        }

        if let PointerPhase::Down(press) = event.phase() {
            let button = press.button;
            if !state.active()
                && valid
                && (button == MouseButton::Middle || (button == MouseButton::Primary && self.armed))
            {
                let touches = std::mem::take(next.touches());
                let pan = Pan {
                    pointer: event.id(),
                    position: event.pos(),
                    transform: self.transform,
                    button,
                };
                return (
                    Step::to(State::Pan {
                        pan,
                        touches,
                        session: self.session.clone(),
                    })
                    .with(Effect::Begin(self.session.clone(), self.transform)),
                    Handled::Yes,
                );
            }
        }
        if let State::Pan { pan, .. } = &next
            && event.id() == pan.pointer
        {
            match event.phase() {
                PointerPhase::Cancel => return (self.stop(&next, false, None), Handled::Yes),
                PointerPhase::Up(press) if press.button == pan.button => {
                    return (
                        self.stop(&next, true, valid.then(|| panned(*pan, event.pos()))),
                        Handled::Yes,
                    );
                }
                PointerPhase::Move if valid => {
                    return (
                        Step::to(next.clone()).with(Effect::Write(panned(*pan, event.pos()))),
                        Handled::Yes,
                    );
                }
                _ => {}
            }
        }
        (Step::to(next), Handled::No)
    }
}
