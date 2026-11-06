//! Role: Transition measured viewport changes into following and history scroll commands.
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_types::Revision;

const EPSILON: f64 = 0.5;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct State {
    follow: Follow,
    // Cache of measured inputs, distinct from the requested offset in Follow.
    sample: Option<Sample>,
}
#[derive(Clone, Copy, PartialEq)]
enum Follow {
    Following {
        expected: Option<f64>,
    },
    Detached {
        offset: f64,
    },
    AwaitingHistory {
        anchor: f64,
        revision: Revision,
        resume: HistoryResume,
    },
}
#[derive(Clone, Copy, PartialEq)]
enum HistoryResume {
    Following,
    Detached,
}
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Sample {
    pub end: f64,
    pub height: f64,
    pub offset: f64,
    pub revision: Option<Revision>,
}
pub(super) enum Event {
    Observe(Sample),
    Jump(f64),
}
pub(super) enum Effect {
    Scroll(f64),
}
pub(super) struct Machine {
    pub reattach: f64,
}
impl Default for State {
    fn default() -> Self {
        Self {
            follow: Follow::Following { expected: None },
            sample: None,
        }
    }
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        let mut next = *state;
        let target = match event {
            Event::Jump(end) => {
                next.follow = match next.follow {
                    Follow::AwaitingHistory { revision, .. } => Follow::AwaitingHistory {
                        anchor: end,
                        revision,
                        resume: HistoryResume::Following,
                    },
                    _ => Follow::Following {
                        expected: Some(end),
                    },
                };
                Some(end)
            }
            Event::Observe(sample) => self.observe(&mut next, sample),
        };
        let step = Step::to(next);
        match target {
            Some(target) => step.with(Effect::Scroll(target)),
            None => step,
        }
    }
}
impl Machine {
    fn observe(&self, state: &mut State, sample: Sample) -> Option<f64> {
        let previous = state.sample.replace(sample);
        let (mut resume, mut expected, pending) = match state.follow {
            Follow::Following { expected } => (HistoryResume::Following, expected, None),
            Follow::Detached { offset } => (HistoryResume::Detached, Some(offset), None),
            Follow::AwaitingHistory {
                anchor,
                revision,
                resume,
            } => (resume, Some(anchor), Some(revision)),
        };
        let acknowledged = expected.is_some_and(|target| (target - sample.offset).abs() <= EPSILON);
        if acknowledged {
            expected = None;
        }
        let Some(previous) = previous else {
            return Self::request(state, resume, sample.offset, sample.end);
        };
        let pending = if sample.revision != previous.revision {
            sample.revision.or(previous.revision)
        } else {
            pending
        };
        if pending.is_some() && sample.height - previous.height > EPSILON {
            let anchor = expected.unwrap_or(previous.offset);
            return Self::request(
                state,
                resume,
                sample.offset,
                (anchor + sample.height - previous.height).min(sample.end),
            );
        }
        if (sample.end - previous.end).abs() > EPSILON {
            if sample.end > previous.end
                && !acknowledged
                && sample.offset < previous.offset - EPSILON
            {
                resume = HistoryResume::Detached;
            }
            if resume == HistoryResume::Following {
                let target = Self::request(state, resume, sample.offset, sample.end);
                if let Some(revision) = pending {
                    state.follow = Follow::AwaitingHistory {
                        anchor: target.unwrap_or(sample.offset),
                        revision,
                        resume,
                    };
                }
                return target;
            }
            expected = None;
        } else if !acknowledged && (sample.offset - previous.offset).abs() > EPSILON {
            expected = None;
            if sample.end - sample.offset <= self.reattach {
                resume = HistoryResume::Following;
            } else if sample.offset < previous.offset {
                resume = HistoryResume::Detached;
            }
        }
        state.follow = match pending {
            Some(revision) => Follow::AwaitingHistory {
                anchor: expected.unwrap_or(sample.offset),
                revision,
                resume,
            },
            None => Self::follow(resume, expected, sample.offset),
        };
        None
    }
    fn follow(resume: HistoryResume, expected: Option<f64>, offset: f64) -> Follow {
        match resume {
            HistoryResume::Following => Follow::Following { expected },
            HistoryResume::Detached => Follow::Detached {
                offset: expected.unwrap_or(offset),
            },
        }
    }
    fn request(state: &mut State, resume: HistoryResume, offset: f64, target: f64) -> Option<f64> {
        let target = ((target - offset).abs() > EPSILON).then_some(target);
        state.follow = Self::follow(resume, target, offset);
        target
    }
}
