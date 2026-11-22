//! Role: Decide a swipe toward an edge and whether its release dismisses.
use super::types::{SwipeDismissPolicy, SwipePhase};
use crate::DragVelocity;
use kivact_core::geometry::{Edge, Point, Rect};
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::{MouseButton, PointerEvent, PointerId, PointerKind, PointerPhase};
use std::time::Duration;
/// How much more the motion toward the edge must be than the motion across it.
const AXIS_RATIO: f64 = 1.2;
/// The recent motion the release speed is measured over.
const VELOCITY_WINDOW: Duration = Duration::from_millis(100);

#[derive(Clone, PartialEq)]
pub(super) struct Contact {
    pub(super) pointer: PointerId,
    origin: Point,
    size: f64,
    velocity: DragVelocity,
}
#[derive(Clone, PartialEq)]
pub(super) enum Swipe {
    Rest,
    Pressed(Contact),
    Dragging { contact: Contact, offset: f64 },
    Blocked { pointer: PointerId },
    Released { travel: f64, size: f64 },
}
impl Swipe {
    pub(super) fn phase(&self) -> SwipePhase {
        match self {
            Self::Rest | Self::Blocked { .. } => SwipePhase::Rest,
            Self::Pressed(_) => SwipePhase::Pressed,
            Self::Dragging { contact, offset } => SwipePhase::Dragging {
                offset: *offset,
                size: contact.size,
            },
            Self::Released { travel, size } => SwipePhase::Dismissed {
                travel: *travel,
                size: *size,
            },
        }
    }
}
pub(super) enum Event {
    Pointer(PointerEvent),
    Protect(PointerId),
}
pub(super) enum Effect {
    Dismiss,
}
pub(super) struct Machine {
    pub(super) policy: SwipeDismissPolicy,
    pub(super) rect: Option<Rect>,
    pub(super) now: u64,
}

impl MachineTrait for Machine {
    type State = Swipe;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &Swipe, event: Event) -> Step<Swipe, Event, Effect> {
        let event = match event {
            Event::Protect(pointer) => return Step::to(Swipe::Blocked { pointer }),
            Event::Pointer(event) => event,
        };
        if !self.policy.inputs.accepts(event.kind()) || matches!(state, Swipe::Released { .. }) {
            return Step::stay();
        }
        let pos = event.window_pos();
        if !pos.x.is_finite() || !pos.y.is_finite() {
            return Step::to(Swipe::Rest);
        }
        if let PointerPhase::Down(press) = event.phase() {
            return self.press(state, &event, press.button, pos);
        }
        let (contact, promoted) = match state {
            Swipe::Pressed(contact) => (contact.clone(), false),
            Swipe::Dragging { contact, .. } => (contact.clone(), true),
            _ => return Step::stay(),
        };
        if contact.pointer != event.id() {
            return Step::stay();
        }
        self.follow(contact, promoted, &event, pos)
    }
}
impl Machine {
    fn press(
        &self,
        state: &Swipe,
        event: &PointerEvent,
        button: MouseButton,
        pos: Point,
    ) -> Step<Swipe, Event, Effect> {
        if !event.is_primary() && event.kind() == PointerKind::Touch {
            return Step::to(Swipe::Rest);
        }
        if matches!(*state, Swipe::Blocked { pointer } if pointer == event.id()) {
            return Step::to(Swipe::Rest);
        }
        if button == MouseButton::Primary
            && matches!(state, Swipe::Rest | Swipe::Blocked { .. })
            && let Some(rect) = self.rect
        {
            let mut velocity = DragVelocity::new(VELOCITY_WINDOW);
            velocity.reset(self.now, pos);
            return Step::to(Swipe::Pressed(Contact {
                pointer: event.id(),
                origin: pos,
                size: extent(rect, self.policy.edge).max(1.0),
                velocity,
            }));
        }
        Step::stay()
    }
    fn follow(
        &self,
        mut contact: Contact,
        promoted: bool,
        event: &PointerEvent,
        pos: Point,
    ) -> Step<Swipe, Event, Effect> {
        let delta = Point::new(pos.x - contact.origin.x, pos.y - contact.origin.y);
        let travel = toward(self.policy.edge, delta).max(0.0);
        contact.velocity.record(self.now, pos);
        match event.phase() {
            PointerPhase::Move => {
                if !promoted {
                    match intent(delta, self.policy.edge, self.policy.slop) {
                        Intent::Pending => return Step::to(Swipe::Pressed(contact)),
                        Intent::Reject => return Step::to(Swipe::Rest),
                        Intent::Promote => {}
                    }
                }
                Step::to(Swipe::Dragging {
                    contact,
                    offset: travel,
                })
            }
            PointerPhase::Up(_) => {
                let speed = toward(self.policy.edge, contact.velocity.pixels_per_ms(self.now));
                if promoted && self.policy.dismisses(travel, contact.size, speed) {
                    Step::to(Swipe::Released {
                        travel,
                        size: contact.size,
                    })
                    .with(Effect::Dismiss)
                } else {
                    Step::to(Swipe::Rest)
                }
            }
            PointerPhase::Cancel => Step::to(Swipe::Rest),
            _ => Step::stay(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Intent {
    Pending,
    Reject,
    Promote,
}
/// The motion toward the edge (along) and across it.
fn split(edge: Edge, delta: Point) -> (f64, f64) {
    match edge {
        Edge::Top | Edge::Bottom => (delta.y, delta.x),
        Edge::Left | Edge::Right => (delta.x, delta.y),
    }
}
pub(super) fn sign(edge: Edge) -> f64 {
    match edge {
        Edge::Top | Edge::Left => -1.0,
        Edge::Bottom | Edge::Right => 1.0,
    }
}
/// How far `delta` goes toward `edge`.
pub(super) fn toward(edge: Edge, delta: Point) -> f64 {
    split(edge, delta).0 * sign(edge)
}
fn extent(rect: Rect, edge: Edge) -> f64 {
    match edge {
        Edge::Top | Edge::Bottom => rect.height,
        Edge::Left | Edge::Right => rect.width,
    }
}
pub(super) fn intent(delta: Point, edge: Edge, slop: f64) -> Intent {
    let (along, across) = split(edge, delta);
    if across.abs() > slop && across.abs() > along.abs() {
        return Intent::Reject;
    }
    if along.abs() <= slop || along.abs() <= across.abs() * AXIS_RATIO {
        return Intent::Pending;
    }
    if along * sign(edge) > 0.0 {
        Intent::Promote
    } else {
        Intent::Reject
    }
}
#[cfg(test)]
mod tests {
    //! Role: Check swipe intent and dismissal thresholds.
    use super::{Intent, intent};
    use crate::SwipeDismissPolicy;
    use kivact_core::geometry::{Edge, Point};
    #[test]
    fn intent_and_flick_thresholds_match_the_edge() {
        assert_eq!(
            intent(Point::new(0.0, 8.0), Edge::Bottom, 8.0),
            Intent::Pending
        );
        assert_eq!(
            intent(Point::new(12.0, 10.0), Edge::Bottom, 8.0),
            Intent::Reject
        );
        assert_eq!(
            intent(Point::new(10.0, 12.0), Edge::Bottom, 8.0),
            Intent::Pending
        );
        assert_eq!(
            intent(Point::new(10.0, 13.0), Edge::Bottom, 8.0),
            Intent::Promote
        );
        assert_eq!(
            intent(Point::new(0.0, -12.0), Edge::Bottom, 8.0),
            Intent::Reject
        );
        assert_eq!(
            intent(Point::new(0.0, -12.0), Edge::Top, 8.0),
            Intent::Promote
        );
        assert_eq!(
            intent(Point::new(12.0, 0.0), Edge::Right, 8.0),
            Intent::Promote
        );
        let toast = SwipeDismissPolicy::toast(Edge::Bottom, 8.0);
        assert!(!toast.dismisses(47.0, 200.0, 100.0));
        assert!(!toast.dismisses(48.0, 200.0, 1.2));
        assert!(toast.dismisses(48.0, 200.0, 1.3));
        assert!(toast.dismisses(80.0, 200.0, 0.0));
    }
}
