//! Role: Inline swipe ownership and month snapping for the touch calendar.
use kivact::custom::clock_now;
use kivact::geometry::Point;
use kivact::i18n::CalendarMonth;
use kivact::interaction::DragVelocity;
use kivact::state::Binding;
use kivact::state::Signal;
use kivact::state::Var;
use kivact::state::signal;
use kivact::state::var;
use kivact::view::Handled;
use kivact::view::PointerEvent;
use kivact::view::PointerId;
use kivact::view::PointerKind;
use kivact::view::PointerPhase;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    Pending,
    Inline,
    Block,
}

#[derive(Clone, Copy)]
struct Contact {
    id: PointerId,
    start: Point,
    axis: Axis,
}

#[derive(Clone, Copy)]
pub(super) struct MonthSwipe {
    month: Binding<CalendarMonth>,
    min: CalendarMonth,
    max: CalendarMonth,
    sign: f64,
    contact: Var<Option<Contact>>,
    velocity: Var<DragVelocity>,
    pub(super) shift: Signal<f64>,
    pub(super) entering: Signal<f64>,
    pub(super) suppress_choice: Signal<bool>,
}

impl MonthSwipe {
    pub(super) fn new(
        month: Binding<CalendarMonth>,
        min: CalendarMonth,
        max: CalendarMonth,
        rtl: bool,
    ) -> Self {
        Self {
            month,
            min,
            max,
            sign: if rtl { -1.0 } else { 1.0 },
            contact: var(None),
            velocity: var(DragVelocity::new(Duration::from_millis(100))),
            shift: signal(0.0),
            entering: signal(0.0),
            suppress_choice: signal(false),
        }
    }

    pub(super) fn pointer(self, event: PointerEvent) -> Handled {
        if event.kind() != PointerKind::Touch || !event.is_primary() {
            return Handled::No;
        }
        let position = event.window_pos();
        let now = clock_now();
        if matches!(event.phase(), PointerPhase::Down(_)) {
            self.suppress_choice.set(false);
            self.entering.set(0.0);
            self.shift.set(0.0);
            self.velocity
                .update(|velocity| velocity.reset(now, position));
            self.contact.set(Some(Contact {
                id: event.id(),
                start: position,
                axis: Axis::Pending,
            }));
            return Handled::No;
        }
        let Some(mut contact) = self
            .contact
            .get()
            .filter(|contact| contact.id == event.id())
        else {
            return Handled::No;
        };
        if event.phase() == PointerPhase::Cancel {
            self.finish();
            return Handled::No;
        }
        if !matches!(event.phase(), PointerPhase::Move | PointerPhase::Up(_)) {
            return Handled::No;
        }
        let (dx, dy) = (position.x - contact.start.x, position.y - contact.start.y);
        if contact.axis == Axis::Pending && dx.hypot(dy) >= 6.0 {
            contact.axis = if dy.abs() <= 2.0 * dx.abs() {
                Axis::Inline
            } else {
                Axis::Block
            };
            self.contact.set(Some(contact));
            self.suppress_choice.set(true);
        }
        if contact.axis != Axis::Inline {
            if matches!(event.phase(), PointerPhase::Up(_)) {
                self.finish();
            }
            return Handled::No;
        }
        self.velocity
            .update(|velocity| velocity.record(now, position));
        if matches!(event.phase(), PointerPhase::Up(_)) {
            let speed = self.velocity.with(|velocity| velocity.pixels_per_ms(now).x);
            let projected = (dx + 499.0 * speed) * self.sign;
            let pages = if dx.abs() >= 32.0 {
                let direction = (-dx * self.sign).signum() as i32;
                let distance = (projected.abs() / 308.0).round().clamp(1.0, 3.0) as i32;
                direction * distance
            } else {
                0
            };
            let current = self.month.get_untracked();
            let next = current.add_months(pages).clamp(self.min, self.max);
            if current != next {
                self.entering
                    .set(if kivact::platform::reduced_motion().get() {
                        0.0
                    } else {
                        pages.signum() as f64 * self.sign * 44.0
                    });
                self.month.set(next);
            }
            self.finish();
        } else {
            let current = self.month.get_untracked();
            let toward = dx * self.sign;
            let edge =
                (toward > 0.0 && current <= self.min) || (toward < 0.0 && current >= self.max);
            self.shift
                .set(dx.clamp(-154.0, 154.0) * if edge { 0.15 } else { 1.0 });
        }
        Handled::Yes
    }

    fn finish(self) {
        self.contact.set(None);
        self.shift.set(0.0);
        self.clear_after_input();
    }

    pub(super) fn clear_after_input(self) {
        if self.suppress_choice.get_untracked() {
            // Native activation may be delivered after the pointer-up event.
            // Keep its binding write suppressed until the next input turn.
            kivact::task::spawn(async move {
                kivact::task::sleep(Duration::from_millis(1)).await;
                if self.contact.get().is_none() {
                    self.suppress_choice.set(false);
                }
            });
        }
    }
}
