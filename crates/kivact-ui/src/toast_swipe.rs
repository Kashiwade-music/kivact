//! Role: Opt-in touch and pen swipes for standard toast cards.

use kivact_core::custom::clock_now;
use kivact_core::geometry::{Edge, Point, Rect};
use kivact_core::motion::{FrameRate, Transition, frame_loop};
use kivact_core::state::{Signal, Value, Var, effect, signal, var};
use kivact_core::style::{Style, Theme, ThemeExtension, TransitionProp, pct, style};
use kivact_core::view::{Handled, MouseButton, PointerEvent, PointerId, PointerKind, PointerPhase};
use kivact_interaction::{DragAxis, ToastId, Toaster};

const SLOP: f64 = 8.0;
const VERTICAL_RATIO: f64 = 1.2;
const DISMISS_RATIO: f64 = 0.4;
const FLICK_DISTANCE: f64 = 48.0;
const FLICK_SPEED: f64 = 1.2;

/// Theme opt-in for touch and pen dismissal of standard toast cards.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ToastSwipe {
    #[default]
    Disabled,
    Enabled {
        edge: Edge,
        transition: Transition,
    },
}

impl ThemeExtension for ToastSwipe {
    fn fallback(_: &Theme) -> Self {
        Self::Disabled
    }
}

#[derive(Clone, Copy)]
struct Contact {
    id: PointerId,
    start: Point,
    start_ns: u64,
    height: f64,
    promoted: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Visual {
    Rest,
    Drag { travel: f64, height: f64 },
    DismissOn { travel: f64, height: f64 },
}

#[derive(Clone, Copy)]
pub(super) struct Swipe {
    edge: Edge,
    rect: Value<Option<Rect>>,
    held: Signal<bool>,
    contact: Var<Option<Contact>>,
    blocked: Var<Option<PointerId>>,
    visual: Signal<Visual>,
}

impl Swipe {
    pub(super) fn new(
        edge: Edge,
        rect: Value<Option<Rect>>,
        held: Signal<bool>,
        toaster: Toaster,
        id: ToastId,
    ) -> Self {
        let visual = signal(Visual::Rest);
        // Publish the exit target and restore its transition before removing
        // the card. The first display tick observes that committed paint.
        let clock = frame_loop(move |_| {
            if matches!(visual.get(), Visual::DismissOn { .. }) {
                toaster.dismiss(id);
            }
        });
        effect(move || {
            if matches!(visual.get(), Visual::DismissOn { .. }) {
                clock.start(FrameRate::Display);
            } else {
                clock.stop();
            }
        });
        Self {
            edge,
            rect,
            held,
            contact: var(None),
            blocked: var(None),
            visual,
        }
    }

    /// The built-in buttons still receive their normal pointer activation.
    pub(super) fn protect_action(self, event: PointerEvent) {
        if matches!(event.phase(), PointerPhase::Down(_)) {
            self.blocked.set(Some(event.id()));
        }
    }

    fn reset(self) {
        self.contact.set(None);
        self.visual.set(Visual::Rest);
        self.held.set(false);
    }

    pub(super) fn event(self, event: PointerEvent) -> Handled {
        if !matches!(event.kind(), PointerKind::Touch | PointerKind::Pen)
            || matches!(self.visual.get(), Visual::DismissOn { .. })
        {
            return Handled::No;
        }
        let pos = event.window_pos();
        if !pos.x.is_finite() || !pos.y.is_finite() {
            self.reset();
            return Handled::No;
        }
        if let PointerPhase::Down(press) = event.phase() {
            if !event.is_primary() && event.kind() == PointerKind::Touch {
                self.reset();
                return Handled::No;
            }
            if self.blocked.get() == Some(event.id()) {
                self.blocked.set(None);
                return Handled::No;
            }
            if press.button == MouseButton::Primary
                && self.contact.get().is_none()
                && let Some(rect) = self.rect.get()
            {
                self.held.set(true);
                self.contact.set(Some(Contact {
                    id: event.id(),
                    start: pos,
                    start_ns: clock_now(),
                    height: rect.height.max(1.0),
                    promoted: false,
                }));
            }
            return Handled::No;
        }
        let Some(mut contact) = self
            .contact
            .get()
            .filter(|contact| contact.id == event.id())
        else {
            return Handled::No;
        };
        let delta = Point::new(pos.x - contact.start.x, pos.y - contact.start.y);
        let sign = edge_sign(self.edge);
        let travel = (delta.y * sign).max(0.0);
        match event.phase() {
            PointerPhase::Move => {
                if !contact.promoted {
                    match intent(delta, sign) {
                        Intent::Pending => return Handled::No,
                        Intent::Reject => {
                            self.reset();
                            return Handled::No;
                        }
                        Intent::Promote => {
                            contact.promoted = true;
                            self.contact.set(Some(contact));
                        }
                    }
                }
                self.visual.set(Visual::Drag {
                    travel,
                    height: contact.height,
                });
                Handled::Yes
            }
            PointerPhase::Up(_) => {
                let elapsed_ms = clock_now().saturating_sub(contact.start_ns) as f64 / 1_000_000.0;
                if contact.promoted && qualifies(travel, contact.height, elapsed_ms) {
                    self.contact.set(None);
                    self.visual.set(Visual::DismissOn {
                        travel,
                        height: contact.height,
                    });
                } else {
                    self.reset();
                }
                if contact.promoted {
                    Handled::Yes
                } else {
                    Handled::No
                }
            }
            PointerPhase::Cancel => {
                self.reset();
                Handled::No
            }
            _ => Handled::No,
        }
    }

    pub(super) fn style(self, transition: &Transition) -> Style {
        let visual = self.visual.get();
        let timing = if matches!(visual, Visual::Drag { .. }) {
            Transition::tween(0.0, kivact_core::motion::Easing::Linear)
        } else {
            transition.clone()
        };
        let base = style().transition(TransitionProp::Paint, timing);
        match visual {
            Visual::Rest => base,
            Visual::Drag { travel, height } | Visual::DismissOn { travel, height } => {
                let progress = (travel / height).clamp(0.0, 1.0);
                let card = base
                    .translate_y(travel * edge_sign(self.edge))
                    .opacity(1.0 - 0.4 * progress)
                    .scale(1.0 - 0.02 * progress);
                if matches!(visual, Visual::DismissOn { .. }) {
                    card.exit(
                        style()
                            .opacity(0.0)
                            .translate_y(pct(120.0 * edge_sign(self.edge)))
                            .scale(1.0),
                    )
                } else {
                    card
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Intent {
    Pending,
    Reject,
    Promote,
}

fn intent(delta: Point, sign: f64) -> Intent {
    if delta.x.abs() > SLOP && delta.x.abs() > delta.y.abs() {
        return Intent::Reject;
    }
    let Some(dy) = DragAxis::Vertical.past_slop(delta, SLOP) else {
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

fn qualifies(travel: f64, height: f64, elapsed_ms: f64) -> bool {
    travel >= (height * DISMISS_RATIO).max(FLICK_DISTANCE)
        || (travel >= FLICK_DISTANCE && travel / elapsed_ms.max(1.0) > FLICK_SPEED)
}

fn edge_sign(edge: Edge) -> f64 {
    match edge {
        Edge::Top | Edge::Left => -1.0,
        Edge::Bottom | Edge::Right => 1.0,
    }
}

#[cfg(test)]
mod tests {
    //! Role: Check swipe intent and dismissal thresholds.
    use super::*;

    #[test]
    fn intent_and_flick_thresholds_match_the_edge() {
        assert_eq!(intent(Point::new(0.0, 8.0), 1.0), Intent::Pending);
        assert_eq!(intent(Point::new(12.0, 10.0), 1.0), Intent::Reject);
        assert_eq!(intent(Point::new(10.0, 12.0), 1.0), Intent::Pending);
        assert_eq!(intent(Point::new(10.0, 13.0), 1.0), Intent::Promote);
        assert_eq!(intent(Point::new(0.0, -12.0), 1.0), Intent::Reject);
        assert_eq!(intent(Point::new(0.0, -12.0), -1.0), Intent::Promote);
        assert!(!qualifies(47.0, 200.0, 1.0));
        assert!(!qualifies(48.0, 200.0, 40.0));
        assert!(qualifies(48.0, 200.0, 39.0));
        assert!(qualifies(80.0, 200.0, 1000.0));
    }
}
