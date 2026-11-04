//! Role: Parses and formats typed dates and names why a text is not a date.

use crate::form::Draft;
use kivact::i18n::{CalendarDate, message};
use kivact::state::{Signal, Value, memo, signal};
use kivact::view::{NodeRef, node_ref};

/// Why the text of a date field is not a date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DateError {
    /// The start of a date, which more characters can complete (also empty).
    Incomplete,
    /// Not, and not the start of, `YYYY-MM-DD`.
    Format,
    /// Written in full, but no such day in the calendar.
    Missing,
}

/// The date `input` stands for: the full form `YYYY-MM-DD` or `YYYY/MM/DD`
/// (four digits of the year, two of the month and the day), nothing less.
fn parse_date(input: &str) -> Result<CalendarDate, DateError> {
    let bytes = input.as_bytes();
    if bytes.len() > 10 {
        return Err(DateError::Format);
    }
    for (at, byte) in bytes.iter().enumerate() {
        let fits = match at {
            4 | 7 => matches!(byte, b'-' | b'/'),
            _ => byte.is_ascii_digit(),
        };
        if !fits {
            return Err(DateError::Format);
        }
    }
    let digits = |from: usize, to: usize| {
        bytes
            .get(from..to.min(bytes.len()).max(from))
            .unwrap_or_default()
            .iter()
            .fold(0u32, |n, digit| n * 10 + u32::from(digit - b'0'))
    };
    // A month starts with 0 or 1 and is 01 through 12; a day starts with 0
    // through 3: past that, no further character makes a date.
    if bytes.len() > 5 && !matches!(bytes[5], b'0' | b'1') {
        return Err(DateError::Format);
    }
    if bytes.len() > 6 && !(1..=12).contains(&digits(5, 7)) {
        return Err(DateError::Format);
    }
    if bytes.len() > 8 && bytes[8] > b'3' {
        return Err(DateError::Format);
    }
    if bytes.len() < 10 {
        return Err(DateError::Incomplete);
    }
    let (year, month, day) = (digits(0, 4) as i32, digits(5, 7) as u8, digits(8, 10) as u8);
    CalendarDate::checked(year, month, day).ok_or(DateError::Missing)
}

/// `date` as shown in a date field: `YYYY-MM-DD`.
fn format_date(date: CalendarDate) -> String {
    date.to_string()
}

/// The text of one date field and what is wrong with it.
#[derive(Clone, Copy)]
pub(super) struct DateText {
    /// The text, following the value (see [`Draft`]).
    pub(super) draft: Draft<CalendarDate>,
    /// What the text stands for.
    pub(super) parsed: Value<Result<CalendarDate, DateError>>,
    /// What to tell the user: a text that is only the start of a date is
    /// not wrong until the field is left or Enter is pressed.
    pub(super) shown: Value<Option<DateError>>,
    /// The user left the field or pressed Enter since the last edit.
    pub(super) checked: Signal<bool>,
    pub(super) node: NodeRef,
}

pub(super) fn date_text(value: Value<CalendarDate>) -> DateText {
    let draft = Draft::new(value, |input| parse_date(input).ok(), format_date);
    let (checked, node) = (signal(false), node_ref());
    let parsed = memo(move || parse_date(&draft.text().get()));
    let shown = memo(move || match parsed.get() {
        Ok(_) => None,
        Err(DateError::Incomplete) => checked.get().then_some(DateError::Format),
        Err(error) => Some(error),
    });
    // Leaving the field checks it.
    kivact::state::effect_on(node.focused(), move |&now| {
        if !now {
            checked.set(true);
        }
    });
    DateText {
        draft,
        parsed,
        shown,
        checked,
        node,
    }
}

/// The error text of a field whose text is wrong in the way `error` says.
pub(super) fn error_message(error: Value<Option<DateError>>) -> Value<String> {
    let format = message("pop.date.error.format", "Use YYYY-MM-DD");
    let missing = message("pop.date.error.invalid", "This date does not exist");
    memo(move || match error.get() {
        Some(DateError::Missing) => missing.get(),
        Some(_) => format.get(),
        None => String::new(),
    })
}
