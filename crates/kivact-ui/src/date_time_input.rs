//! Role: Date and time editing with one explicit active picker panel.
use crate::{DatePresentation, button, calendar, date_input, dialog, time_input};
use kivact_core::i18n::{
    CalendarDate, DateStyle, DateTime, TimeOfDay, TimeStyle, Weekday, message,
};
use kivact_core::state::{
    Binding, IntoBinding, IntoText, IntoValue, Machine, Step, Value, binding, machine,
};
use kivact_core::view::{IntoView, Node, NodeBuilder, View, column, node, row};
#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Date,
    Time,
}
#[derive(Clone, Copy, PartialEq)]
enum DateTimeInputState {
    Closed,
    Open { panel: Panel },
}
enum Event {
    Open(Panel),
    Close,
}
struct Controller;
impl Machine for Controller {
    type State = DateTimeInputState;
    type Event = Event;
    type Effect = ();
    fn transition(
        &self,
        _: &DateTimeInputState,
        event: Event,
    ) -> Step<DateTimeInputState, Event, ()> {
        Step::to(match event {
            Event::Open(panel) => DateTimeInputState::Open { panel },
            Event::Close => DateTimeInputState::Closed,
        })
    }
}
/// A Gregorian date and wall-clock time, with no time zone conversion.
pub struct DateTimeInput {
    root: Node,
    label: Value<String>,
    value: Binding<Option<DateTime>>,
    format: Value<DateStyle>,
    min: Value<Option<DateTime>>,
    max: Value<Option<DateTime>>,
    week_start: Option<Value<Weekday>>,
    presentation: DatePresentation,
    seconds: Value<bool>,
}
/// Edit an optional local date and time.
pub fn date_time_input(
    label: impl IntoText,
    value: impl IntoBinding<Option<DateTime>>,
) -> DateTimeInput {
    DateTimeInput {
        root: node(),
        label: label.into_text_value(),
        value: value.into_binding(),
        format: DateStyle::Iso.into_value(),
        min: None.into_value(),
        max: None.into_value(),
        week_start: None,
        presentation: DatePresentation::Adaptive,
        seconds: false.into_value(),
    }
}
impl DateTimeInput {
    /// Date text format.
    pub fn format(mut self, format: impl IntoValue<DateStyle>) -> Self {
        self.format = format.into_value();
        self
    }
    /// Earliest accepted date and time.
    pub fn min(mut self, min: impl IntoValue<Option<DateTime>>) -> Self {
        self.min = min.into_value();
        self
    }
    /// Latest accepted date and time.
    pub fn max(mut self, max: impl IntoValue<Option<DateTime>>) -> Self {
        self.max = max.into_value();
        self
    }
    /// First weekday column.
    pub fn week_start(mut self, day: impl IntoValue<Weekday>) -> Self {
        self.week_start = Some(day.into_value());
        self
    }
    /// Picker surface selection.
    pub fn presentation(mut self, presentation: DatePresentation) -> Self {
        self.presentation = presentation;
        self
    }
    /// Format wall-clock precision independently of the date format.
    pub fn time_format(mut self, format: impl IntoValue<TimeStyle>) -> Self {
        let format = format.into_value();
        self.seconds = (move || format.get().seconds).into_value();
        self
    }
}
impl NodeBuilder for DateTimeInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
fn key(value: DateTime) -> (CalendarDate, u8, u8, u8) {
    (
        value.date,
        value.time.hour,
        value.time.minute,
        value.time.second,
    )
}
impl IntoView for DateTimeInput {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            value,
            format,
            min,
            max,
            week_start,
            presentation,
            seconds,
        } = self;
        let accepted = binding(value, move |next: Option<DateTime>| {
            if next.is_none_or(|next| {
                min.get().is_none_or(|min| key(next) >= key(min))
                    && max.get().is_none_or(|max| key(next) <= key(max))
            }) {
                value.set(next);
            }
        });
        let date = binding(
            move || value.get().map(|value| value.date),
            move |next| {
                accepted.set(next.map(|date| DateTime {
                    date,
                    time: value.get_untracked().map_or(
                        TimeOfDay {
                            hour: 0,
                            minute: 0,
                            second: 0,
                        },
                        |value| value.time,
                    ),
                }))
            },
        );
        let time = binding(
            move || value.get().map(|value| value.time),
            move |next| {
                accepted.set(next.map(|time| {
                    DateTime {
                        time,
                        date: value
                            .get_untracked()
                            .map_or_else(CalendarDate::today, |value| value.date),
                    }
                }))
            },
        );
        let mut date_field = date_input(message("date_time.date", "Date"), date)
            .format(format)
            .min(move || min.get().map(|value| value.date))
            .max(move || max.get().map(|value| value.date))
            .presentation(presentation);
        if let Some(day) = week_start {
            date_field = date_field.week_start(day);
        }
        let time_field = time_input(message("date_time.time", "Time"), time)
            .format(move || TimeStyle {
                seconds: seconds.get(),
            })
            .presentation(presentation);
        let state = machine(DateTimeInputState::Closed, || Controller, |_, ()| {});
        let open = binding(
            move || matches!(state.get(), DateTimeInputState::Open { .. }),
            move |open| {
                state.send(if open {
                    Event::Open(Panel::Date)
                } else {
                    Event::Close
                })
            },
        );
        let trigger = button(message("date_time.open", "Choose date and time"));
        let content = move || {
            let tabs = row((
                button(message("date_time.date", "Date"))
                    .on_press(move || state.send(Event::Open(Panel::Date))),
                button(message("date_time.time", "Time"))
                    .on_press(move || state.send(Event::Open(Panel::Time))),
            ));
            let body = kivact_core::view::dynamic(
                move || state.get(),
                move |phase| match phase {
                    DateTimeInputState::Open { panel: Panel::Date } => {
                        let mut picker = calendar(date)
                            .min(move || min.get().map(|value| value.date))
                            .max(move || max.get().map(|value| value.date));
                        if let Some(day) = week_start {
                            picker = picker.week_start(day);
                        }
                        if presentation.sheet() {
                            crate::date_wheel::date_panel(date, picker).into_view()
                        } else {
                            picker.into_view()
                        }
                    }
                    DateTimeInputState::Open { panel: Panel::Time } => {
                        crate::date_wheel::time_panel(time, seconds).into_view()
                    }
                    DateTimeInputState::Closed => node().into_view(),
                },
            );
            column((
                tabs,
                body,
                button(message("date_time.done", "Done"))
                    .on_press(move || state.send(Event::Close)),
            ))
        };
        let root = crate::layer::with_base_recipe_style(
            root,
            |r| r.date_time_input_recipes().root_recipe(),
            &crate::variants::Variants::default(),
            kivact_core::custom::PartState::default,
        );
        root.name(label)
            .child(column((
                row((date_field, time_field)),
                dialog(label, open)
                    .mode(presentation.mode())
                    .content(content)
                    .attach(trigger),
            )))
            .into_view()
    }
}
