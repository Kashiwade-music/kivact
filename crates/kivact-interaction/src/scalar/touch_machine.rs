//! Role: Decide touch press, slop tracking, release coast and edit effects atomically.
use super::touch_data::{Coast, Contact, TouchSliderGesture, Travel};
use super::track_data::{Geometry, Grip, Target, nearest};
use crate::{DragAxis, project_release};
use kivact_core::geometry::Point;
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::{PointerId, PointerPhase};
use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub(super) enum State {
    Idle,
    Pressed { contact: Contact },
    Tracking { contact: Contact, anchor: f64 },
    Coasting { coast: Coast },
}
pub(super) enum Event {
    Press(Contact),
    Pointer {
        id: PointerId,
        phase: PointerPhase,
        position: Point,
        now: u64,
    },
    Finish(bool),
    Frame(u64),
    Release {
        index: usize,
        now: u64,
        units: f64,
        sign: f64,
        speed_px: f64,
    },
}
pub(super) enum Effect {
    Begin([f64; 2]),
    Write(Target, f64),
    Commit,
    Cancel,
    Tap(usize, f64, [f64; 2]),
    Release {
        index: usize,
        now: u64,
        units: f64,
        sign: f64,
        speed_px: f64,
    },
}
pub(super) struct Machine {
    pub(super) config: TouchSliderGesture,
    pub(super) geometry: Option<Geometry>,
    pub(super) thumbs: Vec<Target>,
    pub(super) snapshot: [f64; 2],
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match event {
            Event::Press(contact) => Step::to(State::Pressed { contact }),
            Event::Finish(restore) => {
                let step = Step::to(State::Idle);
                if matches!(state, State::Tracking { .. } | State::Coasting { .. }) {
                    step.with(if restore {
                        Effect::Cancel
                    } else {
                        Effect::Commit
                    })
                } else {
                    step
                }
            }
            Event::Frame(now) => self.frame(state, now),
            Event::Release {
                index,
                now,
                units,
                sign,
                speed_px,
            } => {
                let Some(target) = self.thumbs.get(index).copied() else {
                    return Step::to(State::Idle).with(Effect::Commit);
                };
                let from = target.position(target.binding.get_untracked());
                let (min, max) = target.position_limits();
                let travel = project_release(0.0, speed_px, self.config.projection);
                let goal = (from + travel * units * sign).clamp(min, max);
                if !self.config.response.is_zero()
                    && goal != from
                    && ((goal - from).abs() >= 0.5 * units || speed_px.abs() >= 0.001)
                {
                    Step::to(State::Coasting {
                        coast: Coast {
                            index,
                            start: now,
                            from,
                            target: goal,
                            velocity: speed_px * 1000.0 * units * sign,
                            units_per_pixel: units,
                        },
                    })
                } else {
                    Step::to(State::Idle).with(Effect::Commit)
                }
            }
            Event::Pointer {
                id,
                phase,
                position,
                now,
            } => {
                let (contact, anchor) = match state {
                    State::Pressed { contact } => (contact, None),
                    State::Tracking { contact, anchor } => (contact, Some(*anchor)),
                    _ => return Step::stay(),
                };
                if contact.id != id {
                    return Step::stay();
                }
                if matches!(phase, PointerPhase::Cancel) {
                    return self.transition(state, Event::Finish(true));
                }
                if !matches!(phase, PointerPhase::Move | PointerPhase::Up(_)) {
                    return Step::stay();
                }
                self.position(contact.clone(), anchor, phase, position, now)
            }
        }
    }
}
impl Machine {
    fn position(
        &self,
        mut contact: Contact,
        mut anchor: Option<f64>,
        phase: PointerPhase,
        position: Point,
        now: u64,
    ) -> Step<State, Event, Effect> {
        let Some(geometry) = self.geometry else {
            return Step::stay();
        };
        let along = geometry.along(position);
        let delta = Point::new(position.x - contact.origin.x, position.y - contact.origin.y);
        let slop = self.config.slop_px.max(0.0);
        if delta.x.hypot(delta.y) > slop {
            contact.travel = Travel::Moved;
        }
        let mut velocity = (*contact.velocity).clone();
        velocity.record(now, position);
        contact.velocity = Rc::new(velocity);
        let mut effects = Vec::new();
        let mut release = None;
        if let Grip::Thumb(index) = contact.grip
            && let Some(rect) = geometry.rect.get()
            && geometry.length(rect) > 0.0
            && let Some(target) = self.thumbs.get(index).copied()
        {
            let units = 1.0 / geometry.length(rect);
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
            if anchor.is_none()
                && matches!(phase, PointerPhase::Move)
                && let Some(delta) = axis.past_slop(delta, slop)
            {
                anchor = Some(geometry.along(contact.origin) + delta.signum() * slop);
                effects.push(Effect::Begin(self.snapshot));
            }
            if let Some(anchor) = anchor {
                let value = contact.initial + (along - anchor) * units * sign;
                effects.push(Effect::Write(target, value));
                if matches!(phase, PointerPhase::Up(_)) {
                    // The writer may project/reject the final position. Read the
                    // adopted value only after this write effect has run.
                    release = Some((index, now, units, sign));
                }
            }
        }
        let next = self.position_state(
            &contact,
            anchor,
            phase,
            along,
            release.is_some(),
            &mut effects,
        );
        let mut step = Step::to(next);
        for effect in effects {
            step = step.with(effect);
        }
        if let Some((index, now, units, sign)) = release {
            step = step.with(Effect::Release {
                index,
                now,
                units,
                sign,
                speed_px: geometry.along(contact.velocity.pixels_per_ms(now)),
            });
        }
        step
    }
    fn position_state(
        &self,
        contact: &Contact,
        anchor: Option<f64>,
        phase: PointerPhase,
        along: f64,
        releasing: bool,
        effects: &mut Vec<Effect>,
    ) -> State {
        let geometry = self
            .geometry
            .expect("invariant: a positioned touch has geometry");
        if releasing {
            State::Tracking {
                contact: contact.clone(),
                anchor: anchor.expect("invariant: a release has a tracking anchor"),
            }
        } else if matches!(phase, PointerPhase::Up(_)) {
            if contact.travel == Travel::Still
                && !matches!(contact.grip, Grip::Thumb(_))
                && let Some(first) = self.thumbs.first()
            {
                let index = nearest(
                    &self.thumbs,
                    geometry.fraction_at(along, first.length(geometry)),
                );
                effects.push(Effect::Tap(index, along, self.snapshot));
            }
            if anchor.is_some() {
                effects.push(Effect::Commit);
            }
            State::Idle
        } else if let Some(anchor) = anchor {
            State::Tracking {
                contact: contact.clone(),
                anchor,
            }
        } else {
            State::Pressed {
                contact: contact.clone(),
            }
        }
    }
    fn frame(&self, state: &State, now: u64) -> Step<State, Event, Effect> {
        let State::Coasting { coast: run } = state else {
            return Step::stay();
        };
        let elapsed = now.saturating_sub(run.start) as f64 / 1_000_000_000.0;
        let response = self.config.response.as_secs_f64();
        let (position, speed) = coast_sample(run.from, run.target, run.velocity, response, elapsed);
        let end = elapsed >= response * 4.0
            || ((position - run.target).abs() < run.units_per_pixel * 0.5
                && speed.abs() < run.units_per_pixel);
        if let Some(target) = self.thumbs.get(run.index).copied() {
            let (min, max) = target.position_limits();
            let bounded = position.clamp(min, max);
            if !end && position == bounded {
                return Step::stay().with(Effect::Write(target, bounded));
            }
            Step::to(State::Idle)
                .with(Effect::Write(
                    target,
                    if end {
                        run.target.clamp(min, max)
                    } else {
                        bounded
                    },
                ))
                .with(Effect::Commit)
        } else {
            Step::to(State::Idle).with(Effect::Commit)
        }
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
