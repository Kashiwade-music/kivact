//! Role: Validate time text and store exactly one draft phase.
use kivact_core::i18n::TimeOfDay;
use kivact_core::state::{Machine, Step};
fn parse_time(input: &str, seconds: bool) -> Option<TimeOfDay> {
    let lower = input.trim().to_ascii_lowercase();
    let (input, meridiem) = if let Some(time) = lower
        .strip_suffix("am")
        .or_else(|| lower.strip_suffix("a.m."))
    {
        (time.trim_end(), Some(false))
    } else if let Some(time) = lower
        .strip_suffix("pm")
        .or_else(|| lower.strip_suffix("p.m."))
    {
        (time.trim_end(), Some(true))
    } else {
        (lower.as_str(), None)
    };
    if input.is_empty() || !input.bytes().all(|b| b.is_ascii_digit() || b == b':') {
        return None;
    }
    let fields = input.split(':').collect::<Vec<_>>();
    let (hour, minute, second) = match fields.as_slice() {
        [hour] if hour.len() <= 2 => (*hour, "0", "0"),
        [compact] if compact.len() == 3 || compact.len() == 4 => {
            let (hour, minute) = compact.split_at(compact.len() - 2);
            (hour, minute, "0")
        }
        [hour, minute] => (*hour, *minute, "0"),
        [hour, minute, second] if seconds => (*hour, *minute, *second),
        _ => return None,
    };
    let mut hour = hour.parse::<u8>().ok()?;
    if let Some(pm) = meridiem {
        if !(1..=12).contains(&hour) {
            return None;
        }
        hour = hour % 12 + if pm { 12 } else { 0 };
    }
    {
        let minute = minute.parse::<u8>().ok()?;
        let second = second.parse::<u8>().ok()?;
        (hour < 24 && minute < 60 && second < 60).then_some(TimeOfDay {
            hour,
            minute,
            second,
        })
    }
}

#[derive(Clone, PartialEq)]
pub(super) enum TimeDraft {
    Empty,
    Valid { text: String, value: TimeOfDay },
    Invalid(String),
}
#[derive(Clone, PartialEq)]
pub(super) enum TimeInputState {
    Closed(TimeDraft),
    Open(TimeDraft),
}
impl TimeInputState {
    pub(super) fn draft(&self) -> &TimeDraft {
        match self {
            Self::Closed(draft) | Self::Open(draft) => draft,
        }
    }
    pub(super) fn is_open(&self) -> bool {
        matches!(self, Self::Open(_))
    }
}
pub(super) enum Event {
    Text(String),
    Commit(String),
    Pick(Option<TimeOfDay>),
    Open,
    Close,
    External(Option<TimeOfDay>),
}
// xtask-allow(state-fields): Event snapshot contains independent inclusive bounds and formatting precision.
pub(super) struct Snapshot {
    pub(super) seconds: bool,
    pub(super) min: Option<TimeOfDay>,
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
        let valid = |time: TimeOfDay| {
            self.min.is_none_or(|min| key(time) >= key(min))
                && self.max.is_none_or(|max| key(time) <= key(max))
        };
        match event {
            Event::External(value) => {
                let draft = value.map_or(TimeDraft::Empty, |value| TimeDraft::Valid {
                    text: String::new(),
                    value,
                });
                Step::to(if state.is_open() {
                    TimeInputState::Open(draft)
                } else {
                    TimeInputState::Closed(draft)
                })
            }
            Event::Open => Step::to(TimeInputState::Open(state.draft().clone())),
            Event::Close => Step::to(TimeInputState::Closed(state.draft().clone())),
            Event::Pick(None) => Step::to(TimeInputState::Closed(TimeDraft::Empty)).with(None),
            Event::Pick(Some(value)) if valid(value) => {
                Step::to(TimeInputState::Closed(TimeDraft::Valid {
                    text: String::new(),
                    value,
                }))
                .with(Some(value))
            }
            Event::Pick(_) => Step::stay(),
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
            TimeInputState::Open(draft)
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
