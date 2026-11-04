//! Role: Compute pointer corridor transitions and progress deadlines.
use super::geometry::Edge;
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
    pub sample_cache: PointerSampleCache,
}
/// Last input sample, retained across the Holding variant to compare forward progress.
#[derive(Clone, PartialEq)]
pub(super) struct PointerSampleCache(pub Point);
impl Default for PointerSampleCache {
    fn default() -> Self {
        Self(Point::new(0.0, 0.0))
    }
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
                sample_cache: state.sample_cache.clone(),
            });
        };
        let idle = || {
            Step::to(State {
                corridor: Corridor::Idle,
                sample_cache: PointerSampleCache(at.unwrap_or(state.sample_cache.0)),
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
                sample_cache: PointerSampleCache(at),
            })
            .cancel(Event::Expire);
        }
        if surface.contains(at) {
            return idle();
        }
        let (origin, previous) = match state.corridor {
            Corridor::Idle => return idle(),
            Corridor::Tracking { origin, previous } => (origin, previous),
            Corridor::Holding { origin } => (origin, state.sample_cache.0),
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
            sample_cache: PointerSampleCache(at),
        })
        .after(self.timeout, Event::Expire)
        .with(Effect::Progress)
    }
}
