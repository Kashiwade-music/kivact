//! Role: Decide raw touch swipe intent and release outcomes.
use kivact_core::geometry::{Edge, Point, Rect};
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::{MouseButton, PointerEvent, PointerId, PointerKind, PointerPhase};
use kivact_interaction::DragAxis;
const VERTICAL_RATIO: f64 = 1.2;
const DISMISS_RATIO: f64 = 0.4;
const FLICK_DISTANCE: f64 = 48.0;
const FLICK_SPEED: f64 = 1.2;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Contact {
    pub pointer: PointerId,
    pub origin: Point,
    pub started: u64,
    pub height: f64,
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Swipe {
    Rest,
    Pressed(Contact),
    Dragging { contact: Contact, offset: f64 },
    Blocked { pointer: PointerId },
    Released { travel: f64, height: f64 },
}
pub(super) enum Event {
    Pointer(PointerEvent),
    Protect(PointerId),
}
pub(super) enum Effect {
    Hold(bool),
    Dismiss,
}
pub(super) struct Machine {
    pub edge: Edge,
    pub rect: Option<Rect>,
    pub now: u64,
    pub slop: f64,
}
impl MachineTrait for Machine {
    type State = Swipe;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &Swipe, event: Event) -> Step<Swipe, Event, Effect> {
        if let Event::Protect(pointer) = event {
            return Step::to(Swipe::Blocked { pointer });
        }
        let Event::Pointer(event) = event else {
            return Step::stay();
        };
        if !matches!(event.kind(), PointerKind::Touch | PointerKind::Pen)
            || matches!(state, Swipe::Released { .. })
        {
            return Step::stay();
        }
        let reset = || Step::to(Swipe::Rest).with(Effect::Hold(false));
        let pos = event.window_pos();
        if !pos.x.is_finite() || !pos.y.is_finite() {
            return reset();
        }
        if let PointerPhase::Down(press) = event.phase() {
            if !event.is_primary() && event.kind() == PointerKind::Touch {
                return reset();
            }
            if matches!(*state, Swipe::Blocked { pointer } if pointer == event.id()) {
                return reset();
            }
            if press.button == MouseButton::Primary
                && matches!(*state, Swipe::Rest | Swipe::Blocked { .. })
                && let Some(rect) = self.rect
            {
                return Step::to(Swipe::Pressed(Contact {
                    pointer: event.id(),
                    origin: pos,
                    started: self.now,
                    height: rect.height.max(1.0),
                }))
                .with(Effect::Hold(true));
            }
            return Step::stay();
        }
        let (contact, promoted) = match *state {
            Swipe::Pressed(contact) => (contact, false),
            Swipe::Dragging { contact, .. } => (contact, true),
            _ => return Step::stay(),
        };
        if contact.pointer != event.id() {
            return Step::stay();
        }
        let delta = Point::new(pos.x - contact.origin.x, pos.y - contact.origin.y);
        let sign = edge_sign(self.edge);
        let travel = (delta.y * sign).max(0.0);
        match event.phase() {
            PointerPhase::Move => {
                if !promoted {
                    match intent(delta, sign, self.slop) {
                        Intent::Pending => return Step::stay(),
                        Intent::Reject => return reset(),
                        Intent::Promote => {}
                    }
                }
                Step::to(Swipe::Dragging {
                    contact,
                    offset: travel,
                })
            }
            PointerPhase::Up(_) => {
                let elapsed = self.now.saturating_sub(contact.started) as f64 / 1_000_000.0;
                if promoted && qualifies(travel, contact.height, elapsed) {
                    Step::to(Swipe::Released {
                        travel,
                        height: contact.height,
                    })
                    .with(Effect::Dismiss)
                } else {
                    reset()
                }
            }
            PointerPhase::Cancel => reset(),
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

pub(super) fn intent(delta: Point, sign: f64, slop: f64) -> Intent {
    if DragAxis::Horizontal.past_slop(delta, slop).is_some() && delta.x.abs() > delta.y.abs() {
        return Intent::Reject;
    }
    let Some(dy) = DragAxis::Vertical.past_slop(delta, slop) else {
        return Intent::Pending;
    };
    if dy.abs() <= delta.x.abs() * VERTICAL_RATIO {
        Intent::Pending
    } else if dy * sign > 0.0 {
        Intent::Promote
    } else {
        Intent::Reject
    }
}

pub(super) fn qualifies(travel: f64, height: f64, elapsed_ms: f64) -> bool {
    travel >= (height * DISMISS_RATIO).max(FLICK_DISTANCE)
        || (travel >= FLICK_DISTANCE && travel / elapsed_ms.max(1.0) > FLICK_SPEED)
}

pub(super) fn edge_sign(edge: Edge) -> f64 {
    match edge {
        Edge::Top | Edge::Left => -1.0,
        Edge::Bottom | Edge::Right => 1.0,
    }
}

#[cfg(test)]
mod tests {
    //! Role: Check swipe intent and dismissal thresholds.
    use super::{Intent, intent, qualifies};
    use kivact_core::geometry::Point;

    #[test]
    fn intent_and_flick_thresholds_match_the_edge() {
        assert_eq!(intent(Point::new(0.0, 8.0), 1.0, 8.0), Intent::Pending);
        assert_eq!(intent(Point::new(12.0, 10.0), 1.0, 8.0), Intent::Reject);
        assert_eq!(intent(Point::new(10.0, 12.0), 1.0, 8.0), Intent::Pending);
        assert_eq!(intent(Point::new(10.0, 13.0), 1.0, 8.0), Intent::Promote);
        assert_eq!(intent(Point::new(0.0, -12.0), 1.0, 8.0), Intent::Reject);
        assert_eq!(intent(Point::new(0.0, -12.0), -1.0, 8.0), Intent::Promote);
        assert!(!qualifies(47.0, 200.0, 1.0));
        assert!(!qualifies(48.0, 200.0, 40.0));
        assert!(qualifies(48.0, 200.0, 39.0));
        assert!(qualifies(80.0, 200.0, 1000.0));
    }
}
