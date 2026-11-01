//! Role: The DateInput builder: one typed date with a calendar popup.

use super::calendar::{Calendar, calendar};
use super::field::{calendar_popup, date_text_field, field_strip, picker_button, status};
use super::text::{date_text, error_message};
use kivact::control::{Node, NodeBuilder, icon, node};
use kivact::i18n::CalendarDate;
use kivact::interaction::{PopupCloseReason, use_popup};
use kivact::style::{ColorToken, Direction, style};
use kivact::view::{IntoText, IntoValue, IntoView, View, callback, node_ref, show};
use kivact::{Binding, Value, memo, signal};
use std::rc::Rc;

/// An editable Gregorian date with a calendar to choose it from. Build it
/// with [`date_input`].
///
/// The text is `YYYY-MM-DD` (`/` is accepted as the separator as well). It
/// stays as typed while it is being edited; the value follows it only when
/// it is a whole date that exists (four digits of the year, two of the month
/// and of the day), never a text on the way such as `2026-02-3`. The start
/// of a date is not an error until the field is left or Enter is pressed. A
/// text that is not a date marks the field invalid and shows the message
/// `pop.date.error.format` (`Use YYYY-MM-DD`) or, for a whole text that is no day
/// of the calendar, `pop.date.error.invalid` (`This date does not exist`), which
/// describes the field to assistive
/// technology. [`valid`](DateInput::valid) tells the application, so that it
/// can keep a form from being saved. The button that opens the calendar is
/// named by `pop.date.open` (`Choose a date`).
///
/// ```no_run
/// use kivact::i18n::{CalendarDate, Weekday};
/// use kivact::prelude::*;
/// use kivact_ui_pop as ui;
///
/// fn due_date() -> impl IntoView {
///     let today = CalendarDate::new(2026, 10, 3);
///     let date = signal(today);
///     let valid = signal(true);
///     column((
///         ui::date_input("Due", date, today)
///             .valid(valid)
///             .calendar(|calendar| calendar.week_starts_on(Weekday::Monday)),
///         kivact::ui::button("Save").disabled(move || !valid.get()),
///     ))
/// }
/// ```
pub struct DateInput {
    root: Node,
    label: Value<String>,
    value: Binding<CalendarDate>,
    today: CalendarDate,
    valid: Option<Binding<bool>>,
    calendar: Option<Rc<dyn Fn(Calendar) -> Calendar>>,
}

/// An editable date over `value`. The application supplies `today`.
#[track_caller]
pub fn date_input(
    label: impl IntoText,
    value: impl Into<Binding<CalendarDate>>,
    today: CalendarDate,
) -> DateInput {
    DateInput {
        root: node().style(style().direction(Direction::Column)),
        label: label.into_text_value(),
        value: value.into(),
        today,
        valid: None,
        calendar: None,
    }
}

impl DateInput {
    /// Whether the text is a date: not empty, in the format, and a day that
    /// exists. It is written as the text is edited, `true` while the text
    /// follows the value.
    pub fn valid(mut self, valid: impl Into<Binding<bool>>) -> Self {
        self.valid = Some(valid.into());
        self
    }

    /// Set up the calendar that opens, for instance its week start.
    pub fn calendar(mut self, f: impl Fn(Calendar) -> Calendar + 'static) -> Self {
        self.calendar = Some(Rc::new(f));
        self
    }
}

impl NodeBuilder for DateInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for DateInput {
    fn into_view(self) -> View {
        let DateInput {
            root,
            label,
            value: state,
            today,
            valid,
            calendar: configure,
        } = self;
        let input = date_text(state.into_value());
        let draft = input.draft;
        let (parsed, error) = (input.parsed, input.shown);
        let bad = memo(move || error.get().is_some());
        if let Some(valid) = valid {
            kivact::effect(move || valid.set(parsed.get().is_ok()));
        }
        let (open, anchor, message_node) = (signal(false), node_ref(), node_ref());
        let popup = use_popup(open);
        let picked = callback(move |date| {
            draft.show(date);
            popup.close(PopupCloseReason::Selection);
        });
        root.child(field_strip(
            anchor,
            bad,
            (
                icon("calendar").style(style().color(ColorToken::TextMuted)),
                date_text_field(label, input, state, bad, &[message_node]),
                picker_button(popup, || ()),
            ),
        ))
        .child(status(bad, error_message(error), message_node))
        .child(show(popup.shown(), move || {
            let content = calendar(state, today).on_activate(move |value| picked.try_call(value));
            calendar_popup(
                anchor,
                popup,
                label,
                match &configure {
                    Some(configure) => configure(content),
                    None => content,
                },
            )
        }))
        .into_view()
    }
}
