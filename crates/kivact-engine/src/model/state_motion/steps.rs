//! Role: Prepare, retarget, step and shift the transitions of state styles.

use std::sync::Arc;

use super::state::{ColorChannels, PropTrack, StateAnimation, TrackPhase};
use crate::model::Model;
use kivact_types::motion::{SpringRun, Transition, TransitionProp};
use kivact_types::{NodeId, PaintProps, Rgba, apply_components, prop_interp};

/// The properties a state animation moves, one track each.
const PROPS: [TransitionProp; 10] = [
    TransitionProp::Opacity,
    TransitionProp::Transform,
    TransitionProp::Background,
    TransitionProp::BorderColor,
    TransitionProp::BorderWidth,
    TransitionProp::Radius,
    TransitionProp::TextColor,
    TransitionProp::Shadow,
    TransitionProp::Filter,
    TransitionProp::BackdropFilter,
];

/// The transition of `prop` in `paint`: its own entry, else the `All` entry.
fn transition_for(paint: &PaintProps, prop: TransitionProp) -> Option<Arc<Transition>> {
    let list = paint.transitions.as_ref()?;
    let find = |p: TransitionProp| {
        list.iter()
            .find(|(q, _)| *q == p)
            .map(|(_, t)| Arc::new(t.clone()))
    };
    find(prop).or_else(|| find(TransitionProp::All))
}

/// A captured track of `prop` showing `paint` as it is, if it has a
/// transition and can be interpolated.
fn captured_track(prop: TransitionProp, paint: &PaintProps, text: Rgba) -> Option<PropTrack> {
    let transition = transition_for(paint, prop)?;
    let interp = prop_interp(prop, paint, paint, (text, text))?;
    Some(PropTrack {
        prop,
        transition,
        phase: TrackPhase::Captured {
            shown: interp.to.clone(),
            interp,
            spring: SpringRun::default(),
        },
    })
}

impl StateAnimation {
    /// Capture what is shown before a state changes: every track becomes
    /// `Captured` at its drawn value (a running track keeps its velocity),
    /// and a property that has none yet gets one at the target.
    fn capture(&mut self) {
        let (target, text) = (&self.target, self.target_text);
        for track in &mut self.tracks {
            track.phase = match std::mem::replace(&mut track.phase, TrackPhase::Settled) {
                TrackPhase::Running {
                    interp,
                    current,
                    spring,
                    ..
                } => TrackPhase::Captured {
                    interp,
                    shown: current,
                    spring,
                },
                TrackPhase::Held { interp, value } => TrackPhase::Captured {
                    interp,
                    shown: value,
                    spring: SpringRun::default(),
                },
                TrackPhase::Settled => match captured_track(track.prop, target, text) {
                    Some(PropTrack { phase, .. }) => phase,
                    None => TrackPhase::Settled,
                },
                captured @ TrackPhase::Captured { .. } => captured,
            };
        }
        for prop in PROPS {
            if self.tracks.iter().all(|t| t.prop != prop)
                && let Some(track) = captured_track(prop, target, text)
            {
                self.tracks.push(track);
            }
        }
    }

    /// Start the captured tracks toward `target` (and its text color
    /// `text`) at `now`; a track that cannot move, has nothing to move or
    /// is decorative under reduced motion is settled.
    fn retarget(&mut self, target: PaintProps, text: Rgba, now: u64, reduced: bool) {
        for track in &mut self.tracks {
            if !matches!(track.phase, TrackPhase::Captured { .. }) {
                continue;
            }
            let TrackPhase::Captured {
                interp: old,
                shown,
                mut spring,
            } = std::mem::replace(&mut track.phase, TrackPhase::Settled)
            else {
                unreachable!("invariant: the track is captured");
            };
            let Some(transition) = transition_for(&target, track.prop) else {
                continue;
            };
            if reduced && transition.decorative {
                continue;
            }
            // The shown value, written over the old target.
            let mut displayed = self.target.clone();
            let mut displayed_text = self.target_text;
            apply_components(&old.basis, &shown, &mut displayed, &mut displayed_text);
            let Some(new) = prop_interp(track.prop, &displayed, &target, (displayed_text, text))
            else {
                continue;
            };
            if new.to == new.from {
                continue;
            }
            let timing = if old.basis.same_shape(&new.basis) {
                transition.retargeted(&old.from, &old.to, &new.from, &new.to)
            } else {
                // The components do not correspond: a new run, at rest.
                spring.velocity.clear();
                Transition::clone(&transition)
            };
            spring.start(now, &timing, &new.from, &new.to);
            track.transition = Arc::new(timing);
            track.phase = TrackPhase::Running {
                current: new.from.clone(),
                interp: new,
                start: now,
                spring,
            };
        }
        self.target = target;
        self.target_text = text;
    }

    /// Step the running tracks to `now`; one that ends at its target is
    /// settled, one that ends elsewhere (alternate, even count) is held.
    fn step(&mut self, now: u64, reduced: bool) {
        for track in &mut self.tracks {
            let phase = std::mem::replace(&mut track.phase, TrackPhase::Settled);
            track.phase = match phase {
                TrackPhase::Running {
                    interp,
                    mut current,
                    start,
                    mut spring,
                } => {
                    let done = if reduced && track.transition.decorative {
                        current.clone_from(&interp.to);
                        true
                    } else {
                        track.transition.step(
                            &interp.from,
                            &interp.to,
                            start,
                            now,
                            &mut current,
                            &mut spring,
                        )
                    };
                    if !done {
                        TrackPhase::Running {
                            interp,
                            current,
                            start,
                            spring,
                        }
                    } else if current == interp.to {
                        TrackPhase::Settled
                    } else {
                        TrackPhase::Held {
                            interp,
                            value: current,
                        }
                    }
                }
                other => other,
            };
        }
    }
}

impl Model {
    /// Capture the displayed value before the host changes a state.
    pub(crate) fn prepare_state_motion(&mut self, id: NodeId) {
        self.capture_state_motion(id, false);
    }

    /// Capture the paint `id` enters from: its target paint with its enter paint over it.
    pub(crate) fn prepare_entrance(&mut self, id: NodeId) {
        self.capture_state_motion(id, true);
    }

    fn capture_state_motion(&mut self, id: NodeId, entering: bool) {
        if let Some(a) = self.motion.0.get_mut(&id) {
            a.capture();
            return;
        }
        let (paint, text) = if entering {
            (self.enter_paint(id), self.enter_text_color(id))
        } else {
            (self.target_paint(id), self.text_color(id))
        };
        let tracks: Vec<PropTrack> = PROPS
            .into_iter()
            .filter_map(|prop| captured_track(prop, &paint, text))
            .collect();
        if tracks.is_empty() {
            return;
        }
        let colors_text = self
            .tree
            .get(id)
            .and_then(|n| n.states())
            .is_some_and(|s| s.deltas().any(|d| d.text_color.is_some()));
        let mut colors = ColorChannels::default();
        colors.set(ColorChannels::TEXT, colors_text);
        self.motion.0.insert(
            id,
            StateAnimation {
                target: paint,
                target_text: text,
                tracks,
                colors,
            },
        );
    }

    /// Retarget pending captures after input or a prop commit, not every frame.
    pub(crate) fn retarget_states(&mut self, now: u64, reduced: bool) -> bool {
        if self.motion.0.is_empty() {
            return false;
        }
        let mut animations = std::mem::take(&mut self.motion.0);
        animations.retain(|id, a| {
            if self.tree.get(*id).is_none() {
                return false;
            }
            if !a.captured() {
                return true;
            }
            let target = self.target_paint(*id);
            let c = self.text_color(*id);
            let colors_text = a.colors.contains(ColorChannels::TEXT)
                || self
                    .tree
                    .get(*id)
                    .and_then(|n| n.states())
                    .is_some_and(|s| s.deltas().any(|d| d.text_color.is_some()));
            a.colors.set(ColorChannels::TEXT, colors_text);
            a.colors.set(
                ColorChannels::BASE_TEXT,
                !colors_text && c != a.current_color(),
            );
            a.retarget(target, c, now, reduced);
            let moving = !a.current_paint().transform.is_none();
            self.refresh_place(*id, moving);
            !a.settled()
        });
        let active = !animations.is_empty();
        self.motion.0 = animations;
        active
    }

    pub(crate) fn step_states(&mut self, now: u64, reduced: bool) -> bool {
        if self.motion.0.is_empty() {
            return false;
        }
        let mut animations = std::mem::take(&mut self.motion.0);
        animations.retain(|id, a| {
            if self.tree.get(*id).is_none() {
                return false;
            }
            if !a.running() {
                return true;
            }
            a.step(now, reduced);
            let moving = !a.current_paint().transform.is_none();
            self.refresh_place(*id, moving);
            self.repaint(*id);
            if a.target.transform != PaintProps::default().transform
                || self.tree.get(*id).is_some_and(|n| n.place.transformed)
            {
                self.a11y.subtree(*id);
                self.changes
                    .layout
                    .insert(crate::model::changes::LayoutReasons::PASS);
                self.changes.moved();
            }
            !a.settled()
        });
        let active = animations.values().any(StateAnimation::running);
        self.motion.0 = animations;
        active
    }

    pub(crate) fn shift_state_times(&mut self, paused: u64, now: u64) {
        for a in self.motion.0.values_mut() {
            for track in &mut a.tracks {
                if let TrackPhase::Running { start, spring, .. } = &mut track.phase {
                    *start = if *start <= paused {
                        start.saturating_add(now.saturating_sub(paused))
                    } else {
                        now
                    };
                    spring.last_ns = now;
                }
            }
        }
    }
}
