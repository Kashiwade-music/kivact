//! Role: Pure pointer drag lifecycle, threshold, axis, and edit ownership transitions.
use super::data::{Active, AxisLock, DragOptions, EditOwnership, Velocity};
use crate::DragVelocity;
use kivact_core::geometry::Point;
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::PointerId;
use std::rc::Rc;
use std::time::Duration;

pub(super) enum State<T> {
    Idle,
    Pending {
        origin: Point,
        pointer: PointerId,
        active: Active<T>,
    },
    Active(Active<T>),
    Releasing {
        edit: EditOwnership,
    },
}
impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Idle => Self::Idle,
            Self::Pending {
                origin,
                pointer,
                active,
            } => Self::Pending {
                origin: *origin,
                pointer: *pointer,
                active: active.clone(),
            },
            Self::Active(active) => Self::Active(active.clone()),
            Self::Releasing { edit } => Self::Releasing { edit: *edit },
        }
    }
}
impl<T> PartialEq for State<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Idle, Self::Idle) => true,
            (Self::Active(a), Self::Active(b))
            | (Self::Pending { active: a, .. }, Self::Pending { active: b, .. }) => {
                let velocity_equal = match (&a.velocity, &b.velocity) {
                    (Velocity::Disabled, Velocity::Disabled) => true,
                    (Velocity::Tracking(a), Velocity::Tracking(b)) => Rc::ptr_eq(a, b),
                    _ => false,
                };
                a.source == b.source
                    && a.last == b.last
                    && a.edit == b.edit
                    && a.axis == b.axis
                    && Rc::ptr_eq(&a.start, &b.start)
                    && velocity_equal
            }
            (Self::Releasing { edit: a }, Self::Releasing { edit: b }) => a == b,
            _ => false,
        }
    }
}
pub(super) struct Machine<T: 'static> {
    options: DragOptions<T>,
}
impl<T: 'static> Machine<T> {
    pub(super) fn new(options: DragOptions<T>) -> Self {
        Self { options }
    }
}
pub(super) enum Event<T> {
    Begin(Active<T>, u64),
    Position(Point, u64),
    OwnEdit,
    Release(EditOwnership),
    Stop,
    Disabled(bool),
}
pub(super) enum Effect {
    Commit,
}

pub(super) fn position<T>(active: &Active<T>, pos: Point, now: u64, lock: bool) -> Active<T> {
    let mut active = active.clone();
    active.last = pos;
    let delta = Point::new(pos.x - active.start.pos.x, pos.y - active.start.pos.y);
    if lock && active.axis == AxisLock::Free && (delta.x != 0.0 || delta.y != 0.0) {
        active.axis = if delta.x.abs() >= delta.y.abs() {
            AxisLock::Horizontal
        } else {
            AxisLock::Vertical
        };
    }
    if let Velocity::Tracking(samples) = &active.velocity {
        let mut samples = (**samples).clone();
        samples.record(now, pos);
        active.velocity = Velocity::Tracking(Rc::new(samples));
    }
    active
}
impl<T: 'static> MachineTrait for Machine<T> {
    type State = State<T>;
    type Event = Event<T>;
    type Effect = Effect;
    fn transition(&self, state: &State<T>, event: Event<T>) -> Step<State<T>, Event<T>, Effect> {
        match (state, event) {
            (State::Idle, Event::Begin(mut active, now)) => {
                if self.options.track_velocity {
                    let mut samples = DragVelocity::new(Duration::from_millis(100));
                    samples.reset(now, active.start.pos);
                    active.velocity = Velocity::Tracking(Rc::new(samples));
                }
                if self.options.slop > 0.0 {
                    Step::to(State::Pending {
                        origin: active.start.pos,
                        pointer: active.start.pointer,
                        active,
                    })
                } else {
                    Step::to(State::Active(active))
                }
            }
            (
                State::Pending {
                    origin,
                    pointer,
                    active,
                },
                Event::Position(pos, now),
            ) => {
                let delta = Point::new(pos.x - origin.x, pos.y - origin.y);
                if delta.x.hypot(delta.y) > self.options.slop {
                    Step::to(State::Active(position(
                        active,
                        pos,
                        now,
                        self.options.axis_lock,
                    )))
                } else {
                    // Keep sampling within the threshold without selecting an axis.
                    Step::to(State::Pending {
                        origin: *origin,
                        pointer: *pointer,
                        active: position(active, pos, now, false),
                    })
                }
            }
            (State::Active(active), Event::Position(pos, now)) => Step::to(State::Active(
                position(active, pos, now, self.options.axis_lock),
            )),
            (State::Active(active), Event::OwnEdit) => {
                let mut active = active.clone();
                active.edit = EditOwnership::Owned;
                Step::to(State::Active(active))
            }
            (State::Releasing { .. }, Event::OwnEdit) => Step::to(State::Releasing {
                edit: EditOwnership::Owned,
            }),
            (_, Event::Release(edit)) => Step::to(State::Releasing { edit }),
            (_, Event::Stop) => Step::to(State::Idle),
            (State::Active(active), Event::Disabled(true)) => {
                let step = Step::to(State::Idle);
                if active.edit == EditOwnership::Owned {
                    step.with(Effect::Commit)
                } else {
                    step
                }
            }
            (State::Pending { .. }, Event::Disabled(true)) => Step::to(State::Idle),
            _ => Step::stay(),
        }
    }
}
