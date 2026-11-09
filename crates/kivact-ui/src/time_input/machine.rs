//! Role: Validate time text and store exactly one draft phase.
use crate::time_parse::parse_time;
use kivact_core::i18n::TimeOfDay;
use kivact_core::state::{Machine, Step};
#[derive(Clone, PartialEq)]
pub(super) enum TimeDraft {
    Empty,
    Valid { text: String, value: TimeOfDay },
    Invalid(String),
}
#[derive(Clone, PartialEq)]
pub(super) enum TimeInputState {
    Closed(TimeDraft),
    Open {
        draft: TimeDraft,
        invoker: kivact_core::view::NodeRef,
    },
}
impl TimeInputState {
    pub(super) fn draft(&self) -> &TimeDraft {
        match self {
            Self::Closed(draft) | Self::Open { draft, .. } => draft,
        }
    }
    pub(super) fn is_open(&self) -> bool {
        matches!(self, Self::Open { .. })
    }
}
pub(super) enum Event {
    Text(String),
    Commit(String),
    Open(kivact_core::view::NodeRef),
    Close,
    External(Option<TimeOfDay>),
}
pub(super) struct Snapshot {
    // xtask-allow(state-fields): passive record - seconds
    pub(super) seconds: bool,
    // xtask-allow(state-fields): passive record - min
    pub(super) min: Option<TimeOfDay>,
    // xtask-allow(state-fields): passive record - max
    pub(super) max: Option<TimeOfDay>,
}
fn key(time: TimeOfDay) -> (u8, u8, u8) {
    (time.hour, time.minute, time.second)
}
impl Machine for Snapshot {
    type State = TimeInputState;
    type Event = Event;
    type Effect = Option<TimeOfDay>;
    fn transition(
        &self,
        state: &TimeInputState,
        event: Event,
    ) -> Step<TimeInputState, Event, Option<TimeOfDay>> {
        match event {
            Event::External(value) => {
                let draft = value.map_or(TimeDraft::Empty, |value| TimeDraft::Valid {
                    text: String::new(),
                    value,
                });
                Step::to(if state.is_open() {
                    TimeInputState::Open {
                        draft,
                        invoker: match state {
                            TimeInputState::Open { invoker, .. } => *invoker,
                            _ => unreachable!("invariant: only open drafts preserve invokers"),
                        },
                    }
                } else {
                    TimeInputState::Closed(draft)
                })
            }
            Event::Open(invoker) => Step::to(TimeInputState::Open {
                draft: state.draft().clone(),
                invoker,
            }),
            Event::Close => Step::to(TimeInputState::Closed(state.draft().clone())),
            Event::Text(text) => self.edit(state, text, false),
            Event::Commit(text) => self.edit(state, text, true),
        }
    }
}

impl Snapshot {
    fn edit(
        &self,
        state: &TimeInputState,
        text: String,
        commit: bool,
    ) -> Step<TimeInputState, Event, Option<TimeOfDay>> {
        let parsed = parse_time(&text, self.seconds).filter(|time| {
            self.min.is_none_or(|min| key(*time) >= key(min))
                && self.max.is_none_or(|max| key(*time) <= key(max))
        });
        let empty = text.trim().is_empty();
        let draft = if empty {
            TimeDraft::Empty
        } else {
            parsed.map_or_else(
                || TimeDraft::Invalid(text.clone()),
                |value| TimeDraft::Valid {
                    text: text.clone(),
                    value,
                },
            )
        };
        let next = Step::to(if state.is_open() && !commit {
            TimeInputState::Open {
                draft,
                invoker: match state {
                    TimeInputState::Open { invoker, .. } => *invoker,
                    _ => unreachable!("invariant: only open drafts preserve invokers"),
                },
            }
        } else {
            TimeInputState::Closed(draft)
        });
        if commit && (parsed.is_some() || empty) {
            next.with(parsed)
        } else {
            next
        }
    }
}
