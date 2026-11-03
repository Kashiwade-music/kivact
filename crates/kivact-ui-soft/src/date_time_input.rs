//! Role: Combined local date and wall-clock time editor.
use crate::date_input::date_input;
use crate::time_input::{TimeOfDay, time_input};
use kivact::control::{Node, NodeBuilder, node, row};
use kivact::i18n::{CalendarDate, message};
use kivact::style::{Align, style};
use kivact::view::{IntoText, IntoView, Role, View};
use kivact::{Binding, binding};

/// A local Gregorian date and wall-clock time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DateTime {
    pub date: CalendarDate,
    pub time: TimeOfDay,
}

pub struct DateTimeInput {
    root: Node,
    value: Binding<Option<DateTime>>,
    seconds: bool,
}

pub fn date_time_input(
    label: impl IntoText,
    value: impl Into<Binding<Option<DateTime>>>,
) -> DateTimeInput {
    DateTimeInput {
        root: node().role(Role::Group).name(label),
        value: value.into(),
        seconds: false,
    }
}

impl DateTimeInput {
    pub fn show_seconds(mut self, show: bool) -> Self {
        self.seconds = show;
        self
    }
}

impl NodeBuilder for DateTimeInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for DateTimeInput {
    fn into_view(self) -> View {
        let Self {
            root,
            value,
            seconds,
        } = self;
        let date = binding(
            move || value.get().map(|date_time| date_time.date),
            move |date: Option<CalendarDate>| {
                let current = value.get_untracked();
                value.set(date.map(|date| {
                    DateTime {
                        date,
                        time: current
                            .map(|v| v.time)
                            .unwrap_or_else(|| TimeOfDay::new(0, 0, 0).expect("midnight is valid")),
                    }
                }));
            },
        );
        let time = binding(
            move || value.get().map(|date_time| date_time.time),
            move |time: Option<TimeOfDay>| {
                if let Some(current) = value.get_untracked() {
                    value.set(time.map(|time| DateTime { time, ..current }));
                }
            },
        );
        root.child(
            row((
                date_input(message("soft.date_time.date", "Date"), date),
                time_input(message("soft.date_time.time", "Time"), time).show_seconds(seconds),
            ))
            .style(style().align_items(Align::Center).gap(8)),
        )
        .into_view()
    }
}
