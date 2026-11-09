//! Role: Decide switch direct manipulation and owner-scoped release timing.
use super::geometry::Geometry;
use crate::values::{Lift, SwitchDragState, TrackInput};
use kivact_core::geometry::Point;
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::{MouseButton, PointerEvent, PointerId, PointerKind, PointerPhase};
use kivact_interaction::DragAxis;
use std::time::Duration;
const SWITCH_SLOP: f64 = 10.0;
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Contact {
    pointer: PointerId,
    origin: Point,
    started_on: bool,
    kind: PointerKind,
    started: u64,
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Latch {
    Initial,
    Armed,
    Flipped,
    Returned,
    ReturnedArmed,
}
impl Latch {
    fn flipped(self) -> bool {
        matches!(self, Self::Flipped | Self::Returned | Self::ReturnedArmed)
    }
    fn on(self, started: bool) -> bool {
        if self == Self::Flipped {
            !started
        } else {
            started
        }
    }
    fn update(self, toward: f64, past: bool) -> Self {
        if self == Self::Flipped {
            if toward < 0.0 { Self::Returned } else { self }
        } else if matches!(self, Self::Armed | Self::ReturnedArmed) && past {
            Self::Flipped
        } else {
            match (self.flipped(), past) {
                (true, true) => Self::ReturnedArmed,
                (true, false) => Self::Returned,
                (false, true) => Self::Armed,
                (false, false) => Self::Initial,
            }
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Axis {
    Horizontal { offset: f64 },
    Vertical,
}
impl Axis {
    fn direction(self) -> DragAxis {
        match self {
            Self::Horizontal { .. } => DragAxis::Horizontal,
            Self::Vertical => DragAxis::Vertical,
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum SwitchGesture {
    Idle,
    Pressed {
        contact: Contact,
    },
    Dragging {
        contact: Contact,
        axis: Axis,
        latch: Latch,
    },
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum State {
    Gesture(SwitchGesture),
    Obscured(SwitchGesture),
    Released {
        on: bool,
        kind: PointerKind,
        lift: Lift,
    },
}
impl State {
    pub(super) fn visual(self) -> Option<SwitchDragState> {
        match self {
            Self::Gesture(SwitchGesture::Pressed { contact })
                if contact.kind == PointerKind::Touch =>
            {
                Some(SwitchDragState::TouchPressed)
            }
            Self::Gesture(SwitchGesture::Dragging {
                contact,
                axis: Axis::Horizontal { offset },
                ..
            }) => Some(SwitchDragState::Tracking(
                if contact.kind == PointerKind::Touch {
                    TrackInput::Touch { offset }
                } else {
                    TrackInput::Pointer
                },
            )),
            Self::Released {
                kind: PointerKind::Touch,
                lift,
                ..
            } => Some(SwitchDragState::TouchReleased(lift)),
            _ => None,
        }
    }
    pub(super) fn contact(self) -> Option<Contact> {
        match self {
            Self::Gesture(
                SwitchGesture::Pressed { contact } | SwitchGesture::Dragging { contact, .. },
            )
            | Self::Obscured(
                SwitchGesture::Pressed { contact } | SwitchGesture::Dragging { contact, .. },
            ) => Some(contact),
            _ => None,
        }
    }
    pub(super) fn tracks(self, event: PointerEvent) -> bool {
        self.contact()
            .is_some_and(|contact| contact.pointer == event.id())
    }
    pub(super) fn handles(self, event: PointerEvent) -> bool {
        if self.contact().is_none_or(|c| c.pointer != event.id()) {
            return false;
        }
        matches!(
            self,
            Self::Gesture(SwitchGesture::Dragging {
                axis: Axis::Horizontal { .. },
                ..
            }) | Self::Obscured(SwitchGesture::Dragging {
                axis: Axis::Horizontal { .. },
                ..
            })
        )
    }
}
pub(super) enum Event {
    Pointer(PointerEvent),
    Disabled(bool),
    Settle,
}
pub(super) enum Effect {
    Set(bool),
    Suppress(PointerEvent),
}
pub(super) struct Machine {
    pub value: bool,
    pub geometry: Geometry,
    pub now: u64,
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        let event = match event {
            Event::Disabled(true) => return idle(),
            Event::Disabled(false) => return Step::stay(),
            Event::Settle => {
                return match *state {
                    State::Released { on, kind, .. } => Step::to(State::Released {
                        on,
                        kind,
                        lift: Lift::Settled,
                    }),
                    _ => Step::stay(),
                };
            }
            Event::Pointer(event) => event,
        };
        let touch = event.kind() == PointerKind::Touch;
        if touch && !event.is_primary() {
            return Step::stay();
        }
        if matches!(event.phase(), PointerPhase::Up(press) if press.button != MouseButton::Primary)
        {
            return Step::stay();
        }
        if matches!(event.phase(), PointerPhase::Down(press) if press.button == MouseButton::Primary)
            && event.is_primary()
        {
            let next = Step::to(State::Gesture(SwitchGesture::Pressed {
                contact: Contact {
                    pointer: event.id(),
                    origin: event.window_pos(),
                    started_on: self.value,
                    kind: event.kind(),
                    started: self.now,
                },
            }))
            .cancel(Event::Settle);
            return if touch {
                next.with(Effect::Suppress(event))
            } else {
                next
            };
        }
        let Some(contact) = state.contact().filter(|c| c.pointer == event.id()) else {
            return if !touch
                && matches!(event.phase(), PointerPhase::Enter | PointerPhase::Move)
                && state.visual().is_some_and(|visual| visual.is_touch())
            {
                match *state {
                    State::Gesture(gesture) => {
                        Step::to(State::Obscured(gesture)).cancel(Event::Settle)
                    }
                    _ => idle(),
                }
            } else {
                Step::stay()
            };
        };
        match event.phase() {
            PointerPhase::Cancel => idle(),
            PointerPhase::Move | PointerPhase::Up(_) => self.motion(*state, contact, event),
            _ => Step::stay(),
        }
    }
}

/// Cancel gesture tracking and any delayed release settling.
fn idle() -> Step<State, Event, Effect> {
    Step::to(State::Gesture(SwitchGesture::Idle)).cancel(Event::Settle)
}
impl Machine {
    /// Update direct manipulation from one tracked move or release.
    fn motion(
        &self,
        state: State,
        contact: Contact,
        event: PointerEvent,
    ) -> Step<State, Event, Effect> {
        let touch = event.kind() == PointerKind::Touch;
        let delta = Point::new(
            (event.window_pos().x - contact.origin.x) * self.geometry.sign,
            event.window_pos().y - contact.origin.y,
        );
        let (mut axis, mut latch) = match state {
            State::Gesture(SwitchGesture::Dragging { axis, latch, .. })
            | State::Obscured(SwitchGesture::Dragging { axis, latch, .. }) => {
                (Some(axis.direction()), latch)
            }
            _ => (None, Latch::Initial),
        };
        let acquired = axis.is_none().then(|| crossed_axis(delta)).flatten();
        let suppress = acquired == Some(DragAxis::Horizontal);
        axis = axis.or(acquired);
        if axis == Some(DragAxis::Horizontal) {
            let toward = if contact.started_on {
                -delta.x
            } else {
                delta.x
            };
            let side = if contact.started_on {
                self.geometry.on_side
            } else {
                self.geometry.off_side
            };
            latch = latch.update(toward, toward >= self.geometry.width - side);
        }
        let (step, next_value) = if matches!(event.phase(), PointerPhase::Up(_)) {
            self.release(contact, axis, latch, touch)
        } else if let Some(axis) = axis {
            let axis = match axis {
                DragAxis::Horizontal => {
                    let start = if contact.started_on {
                        self.geometry.travel()
                    } else {
                        0.0
                    };
                    Axis::Horizontal {
                        offset: self
                            .geometry
                            .band(start + delta.x, latch.on(contact.started_on))
                            * self.geometry.sign,
                    }
                }
                DragAxis::Vertical => Axis::Vertical,
            };
            (
                Step::to(State::Gesture(SwitchGesture::Dragging {
                    contact,
                    axis,
                    latch,
                })),
                None,
            )
        } else {
            (Step::stay(), None)
        };
        let step = if suppress {
            step.with(Effect::Suppress(event))
        } else {
            step
        };
        if let Some(next) = next_value {
            step.with(Effect::Set(next))
        } else {
            step
        }
    }
    /// Decide the accepted value and delayed touch lift when a contact ends.
    fn release(
        &self,
        contact: Contact,
        axis: Option<DragAxis>,
        latch: Latch,
        touch: bool,
    ) -> (Step<State, Event, Effect>, Option<bool>) {
        if axis == Some(DragAxis::Vertical) || (!touch && axis != Some(DragAxis::Horizontal)) {
            (idle(), None)
        } else {
            let on = if latch.flipped() {
                latch.on(contact.started_on)
            } else {
                !contact.started_on
            };
            let step = if touch {
                Step::to(State::Released {
                    on,
                    kind: contact.kind,
                    lift: Lift::Raised,
                })
                .after(
                    Duration::from_millis(235).saturating_sub(Duration::from_nanos(
                        self.now.saturating_sub(contact.started),
                    )),
                    Event::Settle,
                )
            } else {
                Step::to(State::Released {
                    on,
                    kind: contact.kind,
                    lift: Lift::Settled,
                })
                .cancel(Event::Settle)
            };
            (step, (self.value != on).then_some(on))
        }
    }
}

/// Claim the dominant direct-manipulation axis only after crossing its slop.
fn crossed_axis(delta: Point) -> Option<DragAxis> {
    if DragAxis::Horizontal
        .past_slop(delta, SWITCH_SLOP.next_down())
        .is_some()
        && delta.x.abs() > delta.y.abs()
    {
        Some(DragAxis::Horizontal)
    } else if DragAxis::Vertical
        .past_slop(delta, SWITCH_SLOP.next_down())
        .is_some()
        && delta.y.abs() >= delta.x.abs()
    {
        Some(DragAxis::Vertical)
    } else {
        None
    }
}
