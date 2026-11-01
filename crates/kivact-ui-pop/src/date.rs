//! Role: Calendar and date input components.
//!
//! Calendar dates: the month [`Calendar`] and the text fields over it,
//! [`DateInput`] and [`DateRangeInput`].

use crate::Tokens;
use crate::form::{Draft, input_style};
use kivact::control::{Node, NodeBuilder, icon, node, row, text};
use kivact::draw::Shadow;
use kivact::i18n::{CalendarDate, message};
use kivact::interaction::{Popup, PopupCloseReason, use_popup};
use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::style::{Align, Color, ColorToken, Direction, style};
use kivact::ui::{ButtonVariant, button, text_input};
use kivact::view::{
    A11yAction, IntoText, IntoValue, IntoView, LiveRegion, NodeRef, Overlay, Placement, Role, View,
    callback, node_ref, overlay, show,
};
use kivact::{Binding, Signal, Value, memo, signal};
use std::rc::Rc;

mod calendar;
pub use calendar::{Calendar, CalendarEvent, calendar};

/// Why the text of a date field is not a date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DateError {
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
struct DateText {
    /// The text, following the value (see [`Draft`]).
    draft: Draft<CalendarDate>,
    /// What the text stands for.
    parsed: Value<Result<CalendarDate, DateError>>,
    /// What to tell the user: a text that is only the start of a date is
    /// not wrong until the field is left or Enter is pressed.
    shown: Value<Option<DateError>>,
    /// The user left the field or pressed Enter since the last edit.
    checked: Signal<bool>,
    node: NodeRef,
}

fn date_text(value: Value<CalendarDate>) -> DateText {
    let draft = Draft::new(value, |input| parse_date(input).ok(), format_date);
    let (checked, node) = (signal(false), node_ref());
    let parsed = memo(move || parse_date(&draft.text().get()));
    let shown = memo(move || match parsed.get() {
        Ok(_) => None,
        Err(DateError::Incomplete) => checked.get().then_some(DateError::Format),
        Err(error) => Some(error),
    });
    let (focused, was_focused) = (node.focused(), signal(false));
    kivact::effect(move || {
        if focused.get() {
            was_focused.set(true);
        } else if was_focused.get_untracked() {
            was_focused.set(false);
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
fn error_message(error: Value<Option<DateError>>) -> Value<String> {
    let format = message("pop.date.error.format", "Use YYYY-MM-DD");
    let missing = message("pop.date.error.invalid", "This date does not exist");
    memo(move || match error.get() {
        Some(DateError::Missing) => missing.get(),
        Some(_) => format.get(),
        None => String::new(),
    })
}

/// An error line that is there while `shown`, announced politely. `node` is
/// bound to it, for `described_by`.
fn status(shown: Value<bool>, content: Value<String>, node: NodeRef) -> impl IntoView {
    show(shown, move || {
        text(content)
            .role(Role::Status)
            .live_region(LiveRegion::Polite)
            .node_ref(node)
            .style(style().font_size(12).color(ColorToken::Danger))
    })
}

/// The text field of one date, shared by the single and the range input:
/// the text follows `text`, and a date it completes is written to `value`.
/// `bad` marks the field invalid.
fn date_text_field(
    label: Value<String>,
    text: DateText,
    value: Binding<CalendarDate>,
    bad: Value<bool>,
    described_by: &[NodeRef],
) -> kivact::ui::TextInput {
    let DateText {
        draft,
        checked,
        node,
        ..
    } = text;
    let typed = draft.text();
    let edited = kivact::binding(typed, move |text: String| {
        typed.set(text.clone());
        checked.set(false);
        if let Some(date) = draft.parse(&text) {
            value.set(date);
        }
    });
    let mut field = text_input(label, edited)
        .node_ref(node)
        .invalid(bad)
        .on_submit(move || checked.set(true))
        .style(
            style()
                .grow(1)
                .height(34)
                .min_width(80)
                .padding_x(0)
                .border(0, Color::TRANSPARENT)
                .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT)),
        );
    for node in described_by {
        field = field.described_by(*node);
    }
    field
}

/// The bordered strip around the fields, the calendar icon and the button.
/// It is the anchor of the calendar and turns red while `bad`.
fn field_strip(anchor: NodeRef, bad: Value<bool>, children: impl IntoView) -> Node {
    row(children).node_ref(anchor).style(move || {
        let theme = &current_theme();
        let strip = input_style(theme)
            .height(36)
            .padding_left(12)
            .padding_right(4)
            .gap(8)
            .align_items(Align::Center);
        if bad.get() {
            strip.border_color(theme.get::<Tokens>().red)
        } else {
            strip
        }
    })
}

/// The button that opens the calendar; `on_open` runs before it opens.
fn picker_button(popup: Popup, on_open: impl Fn() + 'static) -> Node {
    let prepare = Rc::new(move || {
        if !popup.shown().get_untracked() {
            on_open();
        }
    });
    button(message("pop.date.open", "Choose a date"))
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(icon("calendar"))
        .part(popup.trigger())
        .on_press({
            let prepare = prepare.clone();
            move || prepare()
        })
        .on_a11y(A11yAction::Expand, move || prepare())
        .style(style().size(28, 28))
        .into_node()
}

/// `content` in a panel below `anchor`, closed by Escape or a press outside.
fn calendar_popup(
    anchor: NodeRef,
    popup: Popup,
    name: Value<String>,
    content: impl IntoView,
) -> Overlay {
    overlay(
        anchor,
        Placement::Below,
        node()
            .child(content)
            .style(
                style()
                    .background(crate::Color::Surface)
                    .border(1, ColorToken::Border)
                    .radius(10)
                    .shadow(Shadow::new(0, 10, 15, ColorToken::Shadow)),
            )
            .style(style().width(320).padding(16)),
    )
    .offset(4)
    .part(popup.content().role(Role::Dialog).name(name))
}

/// An editable Gregorian date with a calendar to choose it from. Build it
/// with [`date_input`].
///
/// The text is `YYYY-MM-DD` (`/` is accepted as the separator as well). It
/// stays as typed while it is being edited; the value follows it only when
/// it is a whole date that exists (four digits of the year, two of the month
/// and of the day), never a text on the way such as `2026-02-3`. The start
/// of a date is not an error until the field is left or Enter is pressed. A
/// text that is not a date marks the field invalid and shows the message
/// `date.error.format` (`Use YYYY-MM-DD`) or, for a whole text that is no day
/// of the calendar, `date.error.invalid` (`This date does not exist`), which
/// describes the field to assistive
/// technology. [`valid`](DateInput::valid) tells the application, so that it
/// can keep a form from being saved. The button that opens the calendar is
/// named by `date.open` (`Choose a date`).
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

/// Two editable dates with one calendar to choose both from. Build it with
/// [`date_range_input`].
///
/// Each field behaves like a [`DateInput`]. In the calendar, the first day
/// chosen sets the start and the end to that day and waits for the end; the
/// second one sets the end, or, when it is before the start, the start (the
/// two are swapped). The range is drawn in the calendar, and the calendar
/// closes after the second choice. A start after the end that is typed makes
/// both fields invalid, shows `date.error.order` (`The end date is before the
/// start date`) and [`valid`](DateRangeInput::valid) `false`.
/// The popup announces `date.range.pick_start` (`Pick a start date`) and
/// `date.range.pick_end` (`Pick an end date`) politely. Its `date.range.close`
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
