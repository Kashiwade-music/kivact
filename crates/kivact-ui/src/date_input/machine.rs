//! Role: Date draft, validation and picker lifetime transitions.
use kivact_core::i18n::{CalendarDate, Locale};
use kivact_core::state::{Machine, Step};

/// Which control opened the picker, so closing returns the focus to it.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum OpenedBy {
    Input,
    Button,
}

/// Whether a typed date has been checked against the locale and bounds.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DateValidation {
    Unchecked,
    Checked,
}

/// The editor's authoritative typed date: the caller's date, or the pending
/// text with its validation.
#[derive(Clone, PartialEq)]
pub(crate) enum TypedDate {
    Following(CalendarDate),
    Editing {
        text: String,
        validation: DateValidation,
    },
}

impl TypedDate {
    /// The typed date for a caller's value: following it, or an empty edit.
    pub(crate) fn initial(value: Option<CalendarDate>) -> Self {
        value.map_or_else(
            || Self::Editing {
                text: String::new(),
                validation: DateValidation::Unchecked,
            },
            Self::Following,
        )
    }
}

/// Whether the picker is shown and, while it is, which control opened it.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Picker {
    Closed,
    Open { opened_by: OpenedBy },
}

/// One machine's state: the typed date and the picker it shows.
#[derive(Clone, PartialEq)]
pub(crate) struct DateField {
    pub(crate) typed: TypedDate,
    pub(crate) picker: Picker,
}

impl DateField {
    /// Whether the picker is shown.
    pub(crate) fn is_open(&self) -> bool {
        matches!(self.picker, Picker::Open { .. })
    }
}

pub(crate) enum Event {
    Open(OpenedBy),
    Close,
    Edit(String),
    Commit(String),
    Pick(CalendarDate),
    SheetPick(CalendarDate),
    Clear,
    External(Option<CalendarDate>),
}

pub(crate) enum Effect {
    /// Publish this date to the caller and announce it.
    Value(Option<CalendarDate>),
    /// Publish whether the typed draft is a valid optional date.
    Validity(bool),
}

pub(crate) struct Snapshot {
    pub(crate) locale: Locale,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(crate) min: Option<CalendarDate>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(crate) max: Option<CalendarDate>,
}

impl Snapshot {
    fn accepts(&self, date: CalendarDate) -> bool {
        self.min.is_none_or(|min| date >= min) && self.max.is_none_or(|max| date <= max)
    }
    fn valid_text(&self, text: &str) -> bool {
        text.trim().is_empty()
            || self
                .locale
                .parse_date(text)
                .is_some_and(|date| self.accepts(date))
    }
}

impl Machine for Snapshot {
    type State = DateField;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &DateField, event: Event) -> Step<DateField, Event, Effect> {
        let picker = state.picker;
        let with = |typed: TypedDate| DateField { typed, picker };
        match event {
            Event::External(value) => {
                Step::to(with(TypedDate::initial(value))).with(Effect::Validity(true))
            }
            Event::Open(opened_by) => Step::to(DateField {
                typed: state.typed.clone(),
                picker: Picker::Open { opened_by },
            }),
            Event::Close => Step::to(DateField {
                typed: state.typed.clone(),
                picker: Picker::Closed,
            }),
            Event::Edit(text) => {
                let valid = self.valid_text(&text);
                Step::to(with(TypedDate::Editing {
                    text,
                    validation: DateValidation::Unchecked,
                }))
                .with(Effect::Validity(valid))
            }
            Event::Commit(text) if text.trim().is_empty() => Step::to(DateField {
                typed: TypedDate::Editing {
                    text: String::new(),
                    validation: DateValidation::Unchecked,
                },
                picker: Picker::Closed,
            })
            .with(Effect::Validity(true))
            .with(Effect::Value(None)),
            Event::Commit(text) => match self
                .locale
                .parse_date(&text)
                .filter(|date| self.accepts(*date))
            {
                Some(date) => Step::to(DateField {
                    typed: TypedDate::Following(date),
                    picker: Picker::Closed,
                })
                .with(Effect::Validity(true))
                .with(Effect::Value(Some(date))),
                None => Step::to(DateField {
                    typed: TypedDate::Editing {
                        text,
                        validation: DateValidation::Checked,
                    },
                    picker,
                })
                .with(Effect::Validity(false))
                .with(Effect::Value(None)),
            },
            Event::Pick(date) if self.accepts(date) => Step::to(DateField {
                typed: TypedDate::Following(date),
                picker: Picker::Closed,
            })
            .with(Effect::Value(Some(date))),
            Event::Pick(_) => Step::stay(),
            Event::SheetPick(date) if self.accepts(date) => {
                Step::to(with(TypedDate::Following(date))).with(Effect::Value(Some(date)))
            }
            Event::SheetPick(_) => Step::stay(),
            Event::Clear => Step::to(DateField {
                typed: TypedDate::Editing {
                    text: String::new(),
                    validation: DateValidation::Unchecked,
                },
                picker: Picker::Closed,
            })
            .with(Effect::Value(None)),
        }
    }
}
