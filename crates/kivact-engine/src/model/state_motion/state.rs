//! Role: Store running state style transitions, their tracks and color channels.

use std::collections::BTreeMap;
use std::sync::Arc;

use kivact_types::motion::{SpringRun, Transition, TransitionProp};
use kivact_types::{NodeId, PaintProps, PropBasis, PropInterp, Rgba, apply_components};

/// The state motion group: the transitions of the nodes whose state
/// styles are moving (allocated only while one moves).
#[derive(Default)]
pub(crate) struct StateMotion(pub(super) BTreeMap<NodeId, StateAnimation>);

impl StateMotion {
    /// The transition of `id`, while it moves.
    pub(crate) fn get(&self, id: NodeId) -> Option<&StateAnimation> {
        self.0.get(&id)
    }

    /// Whether a transition moves.
    pub(crate) fn is_active(&self) -> bool {
        !self.0.is_empty()
    }

    /// Drop the transition of `id`.
    pub(crate) fn forget(&mut self, id: NodeId) {
        self.0.remove(&id);
    }

    /// Whether the transition of `id` is still in flight ([`StateAnimation::in_flight`]).
    pub(crate) fn in_flight(&self, id: NodeId) -> bool {
        self.get(id).is_some_and(StateAnimation::in_flight)
    }

    /// The nodes whose transitions are still in flight.
    pub(crate) fn moving(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.0
            .iter()
            .filter(|(_, a)| a.in_flight())
            .map(|(id, _)| *id)
    }

    /// Whether `id` moves its opacity or its transform now (a track of
    /// either is running): such a node is a compositor layer. Captured and
    /// held tracks do not move it.
    pub(crate) fn moves_layer(&self, id: NodeId) -> bool {
        self.0.get(&id).is_some_and(|a| {
            a.tracks.iter().any(|t| {
                matches!(t.phase, TrackPhase::Running { .. })
                    && matches!(t.prop, TransitionProp::Opacity | TransitionProp::Transform)
            })
        })
    }
}

/// The independently interpolated text color channels.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ColorChannels(u8);

impl ColorChannels {
    pub(super) const TEXT: Self = Self(1);
    pub(super) const BASE_TEXT: Self = Self(2);

    pub(super) fn contains(self, channel: Self) -> bool {
        self.0 & channel.0 != 0
    }

    pub(super) fn set(&mut self, channel: Self, enabled: bool) {
        if enabled {
            self.0 |= channel.0;
        } else {
            self.0 &= !channel.0;
        }
    }
}

/// The state animation of one node: its target paint and one track per
/// property it moves.
#[derive(Debug)]
pub(crate) struct StateAnimation {
    /// The paint the animation moves to.
    pub(super) target: PaintProps,
    /// The text color the animation moves to.
    pub(super) target_text: Rgba,
    /// One track per property that has a transition and can be interpolated.
    pub(super) tracks: Vec<PropTrack>,
    /// Which text colors the tracks move.
    pub(super) colors: ColorChannels,
}

/// The motion of one property.
#[derive(Debug)]
pub(crate) struct PropTrack {
    /// The property it moves.
    pub(super) prop: TransitionProp,
    /// How it moves.
    pub(super) transition: Arc<Transition>,
    /// Where it is in its life.
    pub(super) phase: TrackPhase,
}

/// Where a track is in its life.
#[derive(Debug)]
pub(crate) enum TrackPhase {
    /// A state change is pending: `shown` is drawn; `interp.from`/`to` are
    /// the interrupted run's endpoints (for `Retarget::Proportional`), equal
    /// to `shown` when nothing ran; `spring` keeps the interrupted velocity.
    Captured {
        /// The interrupted interpolation.
        interp: PropInterp,
        /// The numbers drawn.
        shown: Vec<f64>,
        /// The interrupted spring state.
        spring: SpringRun,
    },
    /// Moving from `interp.from` to `interp.to` since `start` (host ns).
    Running {
        /// The interpolation.
        interp: PropInterp,
        /// The numbers drawn now.
        current: Vec<f64>,
        /// When it started (host ns).
        start: u64,
        /// A spring's velocity, step time and iteration.
        spring: SpringRun,
    },
    /// The run ended away from the target (alternate with an even count):
    /// `value` is drawn until the next capture.
    Held {
        /// The finished interpolation.
        interp: PropInterp,
        /// The numbers drawn.
        value: Vec<f64>,
    },
    /// At the target value.
    Settled,
}

impl TrackPhase {
    /// The basis and numbers drawn, unless the track is at the target.
    fn drawn(&self) -> Option<(&PropBasis, &[f64])> {
        match self {
            TrackPhase::Captured { interp, shown, .. } => Some((&interp.basis, shown)),
            TrackPhase::Running {
                interp, current, ..
            } => Some((&interp.basis, current)),
            TrackPhase::Held { interp, value } => Some((&interp.basis, value)),
            TrackPhase::Settled => None,
        }
    }
}

impl StateAnimation {
    /// Whether it moves a state color or an unshadowed base color.
    pub(crate) fn colors_text(&self, inherits_base: impl FnOnce() -> bool) -> bool {
        self.colors.contains(ColorChannels::TEXT)
            || (self.colors.contains(ColorChannels::BASE_TEXT) && inherits_base())
    }

    /// The paint drawn now: the target with every unsettled track's numbers
    /// written in.
    pub(crate) fn current_paint(&self) -> PaintProps {
        self.displayed().0
    }

    /// The text color drawn now.
    pub(crate) fn current_color(&self) -> Rgba {
        self.displayed().1
    }

    /// The paint and the text color drawn now.
    pub(super) fn displayed(&self) -> (PaintProps, Rgba) {
        let mut paint = self.target.clone();
        let mut text = self.target_text;
        for track in &self.tracks {
            if let Some((basis, values)) = track.phase.drawn() {
                apply_components(basis, values, &mut paint, &mut text);
            }
        }
        (paint, text)
    }

    /// Whether any track is running.
    pub(super) fn running(&self) -> bool {
        self.tracks
            .iter()
            .any(|t| matches!(t.phase, TrackPhase::Running { .. }))
    }

    /// Whether every track is at the target.
    pub(super) fn settled(&self) -> bool {
        self.tracks
            .iter()
            .all(|t| matches!(t.phase, TrackPhase::Settled))
    }

    /// Whether any track waits for a retarget.
    pub(super) fn captured(&self) -> bool {
        self.tracks
            .iter()
            .any(|t| matches!(t.phase, TrackPhase::Captured { .. }))
    }

    /// Whether a track still moves or waits to: a held track (a run that
    /// ended away from its target) is not in flight, so a presence waits
    /// for it no longer.
    pub(super) fn in_flight(&self) -> bool {
        self.running() || self.captured()
    }
}
