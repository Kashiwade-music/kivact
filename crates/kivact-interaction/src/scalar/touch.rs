//! Role: Direct manipulation and release coast for opted-in touch sliders.

use super::track::{Grip, Pointer, nearest};
use crate::edit::EditSession;
use crate::{DragAxis, DragVelocity, project_release};
use kivact_core::custom::clock_now;
use kivact_core::geometry::Point;
use kivact_core::motion::{FrameLoop, FrameRate, frame_loop};
use kivact_core::state::{Signal, Value, Var, signal, var};
use kivact_core::view::{Handled, PointerEvent, PointerId, PointerKind, PointerPhase};
use std::time::Duration;

/// Geometry and release timing for a scalar's opt-in touch path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TouchSliderGesture {
    /// Axis displacement required before a thumb begins tracking.
    pub slop_px: f64,
    /// Velocity multiplier for the coast destination.
    pub projection: Duration,
    /// Natural response time of the critically damped release spring.
    pub response: Duration,
}

#[derive(Clone, Copy)]
struct Contact {
    id: PointerId,
    grip: Grip,
    origin: Point,
    anchor: Option<f64>,
    initial: f64,
    moved: bool,
}

#[derive(Clone, Copy, PartialEq)]
struct Coast {
    index: usize,
    start: u64,
    from: f64,
    target: f64,
    velocity: f64,
    units_per_pixel: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct Touch {
    config: TouchSliderGesture,
    pointer: Pointer,
    session: EditSession<[f64; 2]>,
    snapshot: Value<[f64; 2]>,
    contact: Var<Option<Contact>>,
    velocity: Var<DragVelocity>,
    lifted: Signal<Option<usize>>,
    coast: Signal<Option<Coast>>,
    clock: FrameLoop,
}

impl Touch {
    pub(crate) fn new(
        config: TouchSliderGesture,
        pointer: Pointer,
        session: EditSession<[f64; 2]>,
        snapshot: Value<[f64; 2]>,
    ) -> Self {
        let coast = signal(None::<Coast>);
        let clock = frame_loop(move |frame| {
            let Some(run) = coast.get() else {
                return;
            };
            let elapsed = (frame.now.as_nanos().min(u128::from(u64::MAX)) as u64)
                .saturating_sub(run.start) as f64
                / 1_000_000_000.0;
            let response = config.response.as_secs_f64();
            let (position, speed) =
                coast_sample(run.from, run.target, run.velocity, response, elapsed);
            let end = elapsed >= response * 4.0
                || ((position - run.target).abs() < run.units_per_pixel * 0.5
                    && speed.abs() < run.units_per_pixel);
            if let Some(target) = pointer.thumbs.get().get(run.index).copied() {
                let (min, max) = target.limits();
                let bounded = position.clamp(min, max);
                if end || position != bounded {
                    target.set(if end {
                        run.target.clamp(min, max)
                    } else {
                        bounded
                    });
                } else {
                    target.set(bounded);
                    return;
                }
            }
            coast.set(None);
            session.commit();
        });
        // A completed coast need not keep requesting frames.
        kivact_core::effect_on(
            move || coast.with(Option::is_some),
            move |running| {
                if *running {
                    clock.start(FrameRate::Display);
                } else {
                    clock.stop();
                }
            },
        );
        Self {
            config,
            pointer,
            session,
            snapshot,
            contact: var(None),
            coast,
            clock,
            velocity: var(DragVelocity::new(Duration::from_millis(100))),
            lifted: signal(None),
        }
    }

    pub(crate) fn lifted(self, index: usize) -> Value<bool> {
        kivact_core::memo(move || self.lifted.get() == Some(index))
    }

    pub(crate) fn finish(self, restore: bool) -> bool {
        let contact = self.contact.get();
        let coast = self.coast.get().is_some();
        if contact.is_none() && !coast {
            return false;
        }
        self.contact.set(None);
        self.coast.set(None);
        self.lifted.set(None);
        self.clock.stop();
        if coast || contact.is_some_and(|contact| contact.anchor.is_some()) {
            if restore {
                self.session.cancel();
            } else {
                self.session.commit();
            }
        }
        true
    }

    pub(crate) fn event(self, grip: Grip, event: PointerEvent) -> Handled {
        if event.kind() != PointerKind::Touch {
            if matches!(event.phase(), PointerPhase::Down(_)) {
                self.finish(false);
            }
            return Handled::No;
        }
        let Some(geometry) = self.pointer.geometry.get() else {
            return Handled::No;
        };
        let position = event.window_pos();
        if !position.x.is_finite() || !position.y.is_finite() {
            self.finish(true);
            return Handled::Yes;
        }
        let along = geometry.along(position);
        let now = clock_now();
        if matches!(event.phase(), PointerPhase::Down(_)) {
            if !event.is_primary() {
                self.finish(false);
                return Handled::Yes;
            }
            self.finish(false);
            if self.session.is_active() {
                return Handled::Yes;
            }
            let initial = match grip {
                Grip::Thumb(index) => {
                    let Some(target) = self.pointer.thumbs.get().get(index).copied() else {
                        return Handled::Yes;
                    };
                    self.lifted.set(Some(index));
                    target.binding.get_untracked()
                }
                _ => 0.0,
            };
            self.contact.set(Some(Contact {
                id: event.id(),
                grip,
                origin: position,
                anchor: None,
                initial,
                moved: false,
            }));
            self.velocity
                .update(|velocity| velocity.reset(now, position));
            return Handled::Yes;
        }
        let Some(mut contact) = self
            .contact
            .get()
            .filter(|contact| contact.id == event.id())
        else {
            return Handled::Yes;
        };
        if matches!(event.phase(), PointerPhase::Cancel) {
            self.finish(true);
            return Handled::Yes;
        }
        if matches!(event.phase(), PointerPhase::Move | PointerPhase::Up(_)) {
            let dx = position.x - contact.origin.x;
            let dy = position.y - contact.origin.y;
            let slop = self.config.slop_px.max(0.0);
            contact.moved |= dx.hypot(dy) > slop;
            self.velocity
                .update(|velocity| velocity.record(now, position));
            if let Grip::Thumb(index) = contact.grip
                && let Some(rect) = geometry.rect.get()
                && geometry.length(rect) > 0.0
                && let Some(target) = self.pointer.thumbs.get().get(index).copied()
            {
                let range = target.range.get_untracked();
                let units = (range.max() - range.min()) / geometry.length(rect);
                let sign = if geometry.rtl || !geometry.axis.is_horizontal() {
                    -1.0
                } else {
                    1.0
                };
                let axis = if geometry.axis.is_horizontal() {
                    DragAxis::Horizontal
                } else {
                    DragAxis::Vertical
                };
                if contact.anchor.is_none()
                    && matches!(event.phase(), PointerPhase::Move)
                    && let Some(delta) = axis.past_slop(Point::new(dx, dy), slop)
                {
                    contact.anchor = Some(geometry.along(contact.origin) + delta.signum() * slop);
                    self.session.begin(self.snapshot.get_untracked());
                }
                if let Some(anchor) = contact.anchor {
                    target.set(contact.initial + (along - anchor) * units * sign);
                }
                self.contact.set(Some(contact));
                if matches!(event.phase(), PointerPhase::Up(_)) && contact.anchor.is_some() {
                    self.contact.set(None);
                    self.lifted.set(None);
                    let speed_px = geometry.along(self.velocity.with(|v| v.pixels_per_ms(now)));
                    let travel = project_release(0.0, speed_px, self.config.projection);
                    let from = target.binding.get_untracked();
                    let (min, max) = target.limits();
                    let goal = (from + travel * units * sign).clamp(min, max);
                    if !self.config.response.is_zero()
                        && goal != from
                        && ((goal - from).abs() >= 0.5 * units || speed_px.abs() >= 0.001)
                    {
                        self.coast.set(Some(Coast {
                            index,
                            start: now,
                            from,
                            target: goal,
                            velocity: speed_px * 1000.0 * units * sign,
                            units_per_pixel: units,
                        }));
                    } else {
                        self.session.commit();
                    }
                }
            }
            if matches!(event.phase(), PointerPhase::Up(_)) && self.contact.get().is_some() {
                if !contact.moved && !matches!(contact.grip, Grip::Thumb(_)) {
                    let thumbs = self.pointer.thumbs.get();
                    if let Some(first) = thumbs.first() {
                        let index =
                            nearest(&thumbs, geometry.fraction_at(along, first.length(geometry)));
                        self.session.once(self.snapshot.get_untracked(), || {
                            self.pointer.write(index, along)
                        });
                    }
                }
                self.finish(false);
            } else if matches!(event.phase(), PointerPhase::Move) {
                self.contact.set(Some(contact));
            }
        }
        Handled::Yes
    }
}

fn coast_sample(from: f64, target: f64, velocity: f64, response: f64, elapsed: f64) -> (f64, f64) {
    let omega = std::f64::consts::TAU / response;
    let start = from - target;
    let slope = velocity + omega * start;
    let decay = (-omega * elapsed).exp();
    let displacement = (start + slope * elapsed) * decay;
    (
        target + displacement,
        (slope - omega * (start + slope * elapsed)) * decay,
    )
}

#[cfg(test)]
mod tests {
    //! Role: Check touch-slider release calculations.

    use super::coast_sample;

    #[test]
    fn critically_damped_coast_starts_with_the_release_velocity() {
        let (position, velocity) = coast_sample(20.0, 60.0, 400.0, 0.4, 0.0);
        assert!((position - 20.0).abs() < 1e-9);
        assert!((velocity - 400.0).abs() < 1e-9);
        let (position, _) = coast_sample(20.0, 60.0, 400.0, 0.4, 0.1);
        assert!(position > 20.0 && position < 60.0);
        let (position, velocity) = coast_sample(20.0, 60.0, 400.0, 0.4, 1.0);
        assert!((position - 60.0).abs() < 0.01);
        assert!(velocity.abs() < 0.01);
    }
}
