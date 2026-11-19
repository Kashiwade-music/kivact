//! Role: The number field's typed-text draft machine.

use kivact_core::i18n::{FractionDigits, Grouping, Locale, NumberStyle};
use kivact_core::state::{Machine, Step};
use kivact_interaction::ScalarRange;
/// What a number field shows: the value formatted, or what the user typed
/// while the field has the focus.
#[derive(Clone, PartialEq)]
pub(super) enum NumberText {
    Value(String),
    /// `produced` is the stored value the last parsable text produced
    /// (`None` until one parses).
    Typed {
        text: String,
        produced: Option<f64>,
    },
}

impl NumberText {
    pub(super) fn text(&self) -> &str {
        match self {
            NumberText::Value(text) | NumberText::Typed { text, .. } => text,
        }
    }
}

pub(super) enum NumberEvent {
    ValueChanged,
    Typed(String),
    FocusOut,
}

pub(super) struct NumberInputSnapshot {
    pub(super) value: f64,
    pub(super) range: ScalarRange,
    pub(super) locale: Locale,
}

impl Machine for NumberInputSnapshot {
    type State = NumberText;
    type Event = NumberEvent;
    type Effect = ();
    fn transition(
        &self,
        state: &NumberText,
        event: NumberEvent,
    ) -> Step<NumberText, NumberEvent, ()> {
        match event {
            NumberEvent::ValueChanged
                if self.locale.parse_number(state.text()) == Some(self.value)
                    || matches!(state, NumberText::Typed { produced: Some(p), .. } if *p == self.value) =>
            {
                Step::stay()
            }
            NumberEvent::ValueChanged | NumberEvent::FocusOut => Step::to(NumberText::Value(
                format_value(self.value, self.range, &self.locale),
            )),
            NumberEvent::Typed(text) => {
                let parsed = self.locale.parse_number(&text);
                let produced = match parsed {
                    Some(_) => Some(self.value),
                    None => match state {
                        NumberText::Typed { produced, .. } => *produced,
                        NumberText::Value(_) => None,
                    },
                };
                Step::to(NumberText::Typed { text, produced })
            }
        }
    }
}

pub(super) fn format_value(value: f64, range: ScalarRange, locale: &Locale) -> String {
    let places = range.step().map_or_else(
        || places(value),
        |step| {
            places(step)
                .max(places(range.min()))
                .max(places(range.max()))
        },
    );
    locale.format_number(
        value,
        NumberStyle {
            fraction_digits: FractionDigits::Range {
                min: 0,
                max: places.min(255) as u8,
            },
            grouping: Grouping::None,
        },
    )
}

/// Decimal places required by the step, including a decimal exponent.
pub(super) fn places(step: f64) -> usize {
    let text = step.to_string();
    let (mantissa, exponent) =
        text.split_once('e')
            .map_or((text.as_str(), 0), |(mantissa, exponent)| {
                (
                    mantissa,
                    exponent
                        .parse::<i32>()
                        .expect("invariant: f64 exponent is an integer"),
                )
            });
    let decimals = mantissa
        .split_once('.')
        .map_or(0, |(_, decimals)| decimals.len());
    (decimals as i32 - exponent).max(0) as usize
}
