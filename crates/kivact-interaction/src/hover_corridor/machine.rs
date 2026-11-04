//! Role: Compute pointer corridor transitions and progress deadlines.
use super::api::Edge;
use kivact_core::geometry::{Point, Rect};
use kivact_core::state::{Machine as MachineTrait, Step};
use std::time::Duration;
#[derive(Clone, PartialEq)]
pub(super) enum Corridor {
    Idle,
    Tracking { origin: Point, previous: Point },
    Holding { origin: Point },
}
#[derive(Clone, PartialEq)]
pub(super) struct State {
    pub corridor: Corridor,
    pub sample: Option<Point>,
}
pub(super) enum Event {
    Sample {
        geometry: Option<(Rect, Rect)>,
        at: Option<Point>,
        available: bool,
    },
    Expire,
}
pub(super) enum Effect {
    Progress,
}
pub(super) struct Machine {
    pub timeout: Duration,
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        let Event::Sample {
            geometry,
            at,
            available,
        } = event
        else {
            return Step::to(State {
                corridor: Corridor::Idle,
                sample: state.sample,
            });
        };
        let idle = || {
            Step::to(State {
                corridor: Corridor::Idle,
                sample: at,
            })
            .cancel(Event::Expire)
        };
        let (true, Some((anchor, surface)), Some(at)) = (available, geometry, at) else {
            return idle();
        };
        if anchor.contains(at) {
            return Step::to(State {
                corridor: Corridor::Tracking {
                    origin: at,
                    previous: at,
                },
                sample: Some(at),
            })
            .cancel(Event::Expire);
        }
        if surface.contains(at) {
            return idle();
        }
        let (origin, previous) = match state.corridor {
            Corridor::Idle => return idle(),
            Corridor::Tracking { origin, previous } => (origin, previous),
            Corridor::Holding { origin } => (origin, state.sample.unwrap_or(origin)),
        };
        let Some(edge) = Edge::facing(origin, surface) else {
            return idle();
        };
        if !edge.contains(at, origin) || edge.distance(at) > edge.distance(previous) {
            return idle();
        }
        if edge.distance(at) == edge.distance(previous) {
            return Step::stay();
        }
        Step::to(State {
            corridor: Corridor::Holding { origin },
            sample: Some(at),
        })
        .after(self.timeout, Event::Expire)
        .with(Effect::Progress)
    }
}
