//! Role: The DateRangeInput builder: a typed start and end with a range calendar.

use super::calendar::{Calendar, calendar};
use super::field::{calendar_popup, date_text_field, field_strip, picker_button, status};
use super::text::{date_text, error_message};
use kivact::control::{Node, NodeBuilder, icon, node, text};
use kivact::i18n::{CalendarDate, message};
use kivact::interaction::{PopupCloseReason, use_popup};
use kivact::primitive::NodeExt;
use kivact::style::{ColorToken, Direction, style};
use kivact::view::{
    IntoText, IntoValue, IntoView, LiveRegion, Role, View, callback, node_ref, show,
};
use kivact::{Binding, Value, memo, signal};
use std::rc::Rc;

/// Two editable dates with one calendar to choose both from. Build it with
/// [`date_range_input`].
///
/// Each field behaves like a [`DateInput`]. In the calendar, the first day
/// chosen sets the start and the end to that day and waits for the end; the
/// second one sets the end, or, when it is before the start, the start (the
/// two are swapped). The range is drawn in the calendar, and the calendar
/// closes after the second choice. A start after the end that is typed makes
/// both fields invalid, shows `pop.date.error.order` (`The end date is before the
/// start date`) and [`valid`](DateRangeInput::valid) `false`.
/// The popup announces `pop.date.range.pick_start` (`Pick a start date`) and
/// `pop.date.range.pick_end` (`Pick an end date`) politely. Its `pop.date.range.close`
/// (`Close`) button dismisses it and returns focus to the start input.
///
/// ```no_run
/// use kivact::i18n::CalendarDate;
/// use kivact::prelude::*;
/// use kivact_ui_pop as ui;
///
/// fn stay() -> impl IntoView {
///     let today = CalendarDate::new(2026, 10, 3);
///     let (from, to) = (signal(today), signal(today.add_days(2)));
///     ui::date_range_input(("From", "To"), (from, to), today)
/// }
/// ```
pub struct DateRangeInput {
    root: Node,
    start_label: Value<String>,
    end_label: Value<String>,
    start: Binding<CalendarDate>,
    end: Binding<CalendarDate>,
    today: CalendarDate,
    valid: Option<Binding<bool>>,
    calendar: Option<Rc<dyn Fn(Calendar) -> Calendar>>,
}

/// Two editable dates over `start` and `end`. The application supplies `today`.
#[track_caller]
pub fn date_range_input(
    labels: (impl IntoText, impl IntoText),
    values: (
        impl Into<Binding<CalendarDate>>,
        impl Into<Binding<CalendarDate>>,
    ),
    today: CalendarDate,
) -> DateRangeInput {
    let (start_label, end_label) = labels;
    let (start, end) = values;
    DateRangeInput {
        root: node().style(style().direction(Direction::Column)),
        start_label: start_label.into_text_value(),
        end_label: end_label.into_text_value(),
        start: start.into(),
        end: end.into(),
        today,
        valid: None,
        calendar: None,
    }
}

impl DateRangeInput {
    /// Whether both texts are dates and the start is not after the end. It is
    /// written as the texts are edited.
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

impl NodeBuilder for DateRangeInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for DateRangeInput {
    fn into_view(self) -> View {
        let DateRangeInput {
            root,
            start_label,
            end_label,
            start,
            end,
            today,
            valid,
            calendar: configure,
        } = self;
        let (start_value, end_value) = (start.into_value(), end.into_value());
        let (start_text, end_text) = (date_text(start_value), date_text(end_value));
        let (parsed_start, parsed_end) = (start_text.parsed, end_text.parsed);
        let (start_error, end_error) = (start_text.shown, end_text.shown);
        let (start_draft, end_draft) = (start_text.draft, end_text.draft);
        let reversed =
            memo(move || matches!((parsed_start.get(), parsed_end.get()), (Ok(a), Ok(b)) if a > b));
        let start_bad = memo(move || start_error.get().is_some() || reversed.get());
        let end_bad = memo(move || end_error.get().is_some() || reversed.get());
        let any_bad = memo(move || start_bad.get() || end_bad.get());
        if let Some(valid) = valid {
            kivact::effect(move || {
                valid.set(parsed_start.get().is_ok() && parsed_end.get().is_ok() && !reversed.get())
            });
        }
        let range = memo(move || {
            let (from, to) = (start_value.get(), end_value.get());
            (from <= to).then_some((from, to))
        });
        let cursor = signal(start_value.get_untracked());
        let waiting_for_end = signal(false);
        let open = signal(false);
        let anchor = node_ref();
        let popup = use_popup(open);
        let (start_message, end_message, order_message) = (node_ref(), node_ref(), node_ref());
        let picked = callback(move |date| {
            let (from, to) = if waiting_for_end.get() {
                let first = start_value.get();
                (first.min(date), first.max(date))
            } else {
                (date, date)
            };
            start.set(from);
            end.set(to);
            start_draft.show(from);
            end_draft.show(to);
            if waiting_for_end.get() {
                popup.close(PopupCloseReason::Selection);
            }
            waiting_for_end.update(|waiting| *waiting = !*waiting);
        });
        let start_error_shown = memo(move || start_error.get().is_some());
        let end_error_shown = memo(move || end_error.get().is_some());
        root.child(field_strip(
            anchor,
            any_bad,
            (
                icon("calendar").style(style().color(ColorToken::TextMuted)),
                date_text_field(
                    start_label,
                    start_text,
                    start,
                    start_bad,
                    &[start_message, order_message],
                ),
                text("–").style(style().color(ColorToken::TextMuted)),
                date_text_field(
                    end_label,
                    end_text,
                    end,
                    end_bad,
                    &[end_message, order_message],
                ),
                picker_button(popup, move || {
                    waiting_for_end.set(false);
                    cursor.set(start_value.get_untracked());
                }),
            ),
        ))
        .child(status(
            start_error_shown,
            error_message(start_error),
            start_message,
        ))
        .child(status(
            end_error_shown,
            error_message(end_error),
            end_message,
        ))
        .child(status(
            reversed,
            message(
                "pop.date.error.order",
                "The end date is before the start date",
            ),
            order_message,
        ))
        .child(show(popup.shown(), move || {
            let content = calendar(cursor, today)
                .range(range)
                .on_activate(move |value| picked.try_call(value));
            let content = match &configure {
                Some(configure) => configure(content),
                None => content,
            };
            let pick_start = message("pop.date.range.pick_start", "Pick a start date");
            let pick_end = message("pop.date.range.pick_end", "Pick an end date");
            let instructions = text(move || {
                if waiting_for_end.get() {
                    pick_end.get()
                } else {
                    pick_start.get()
                }
            })
            .role(Role::Status)
            .live_region(LiveRegion::Polite);
            let close =
                kivact::ui::button(message("pop.date.range.close", "Close")).on_press(move || {
                    popup.close(PopupCloseReason::Programmatic);
                    start_text.node.focus();
                });
            calendar_popup(
                anchor,
                popup,
                start_label,
                node()
                    .style(style().direction(Direction::Column).gap(8))
                    .child(instructions)
                    .child(content)
                    .child(close),
            )
        }))
        .into_view()
    }
}
