//! Role: Combined local date and wall-clock time editor.
use crate::date_input::date_input;
use crate::date_touch::{self, DatePresentation};
use crate::time_input::time_input;
use crate::time_of_day::TimeOfDay;
use kivact::control::{Node, NodeBuilder, node, row};
use kivact::i18n::{CalendarDate, message};
use kivact::style::{Align, style};
use kivact::view::{IntoText, IntoView, Role, View, dynamic, node_ref};
use kivact::{Binding, Value, binding, memo, signal};

/// A local Gregorian date and wall-clock time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DateTime {
    pub date: CalendarDate,
    pub time: TimeOfDay,
}

pub struct DateTimeInput {
    root: Node,
    label: Value<String>,
    presentation: DatePresentation,
    value: Binding<Option<DateTime>>,
    seconds: bool,
}

pub fn date_time_input(
    label: impl IntoText,
    value: impl Into<Binding<Option<DateTime>>>,
) -> DateTimeInput {
    let label = label.into_text_value();
    DateTimeInput {
        root: node().role(Role::Group).name(label),
        label,
        value: value.into(),
        presentation: DatePresentation::Adaptive,
        seconds: false,
    }
}

impl DateTimeInput {
    /// Choose how the picker opens.
    pub fn presentation(mut self, presentation: DatePresentation) -> Self {
        self.presentation = presentation;
        self
    }

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
            label,
            value,
            seconds,
            presentation,
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
        let date_label = message("soft.date_time.date", "Date");
        let time_label = message("soft.date_time.time", "Time");
        if presentation.sheet() {
            let open = signal(false);
            let time_panel = signal(false);
            let draft_time = signal(None::<TimeOfDay>);
            let sheet_time = binding(
                move || time.get().or_else(|| draft_time.get()),
                move |next| {
                    if value.get_untracked().is_some() {
                        time.set(next);
                    } else {
                        draft_time.set(next);
                    }
                },
            );
            let sheet_date = binding(date, move |next| {
                if let (Some(date), Some(time)) = (next, draft_time.get_untracked()) {
                    value.set(Some(DateTime { date, time }));
                    draft_time.set(None);
                } else {
                    date.set(next);
                }
            });
            let date_open = memo(move || open.get() && !time_panel.get());
            let time_open = memo(move || open.get() && time_panel.get());
            let date_trigger = date_touch::trigger(
                date_label,
                move || {
                    date.get()
                        .map(|date| date.to_string())
                        .unwrap_or_else(|| "YYYY-MM-DD".to_owned())
                },
                "calendar",
                date_open,
                node_ref(),
                move || {
                    time_panel.set(false);
                    open.set(true);
                },
            );
            let time_trigger = date_touch::trigger(
                time_label,
                move || date_touch::time_label(sheet_time.get(), seconds),
                "clock",
                time_open,
                node_ref(),
                move || {
                    time_panel.set(true);
                    open.set(true);
                },
            );
            let save_date = message("soft.date_time.save_date", "Save date");
            let save = message("soft.date_time.save", "Save");
            let action_label = memo(move || {
                if time_panel.get() {
                    save.get()
                } else {
                    save_date.get()
                }
            });
            let sheet = date_touch::sheet_with_action(
                label,
                open,
                None,
                move || {
                    let date_tab = kivact::ui::button(date_label)
                        .selected(move || !time_panel.get())
                        .on_press(move || time_panel.set(false));
                    let time_tab = kivact::ui::button(time_label)
                        .selected(time_panel)
                        .on_press(move || time_panel.set(true));
                    let panel = dynamic(
                        move || time_panel.get(),
                        move |time_open| {
                            if time_open {
                                date_touch::time_panel(sheet_time, seconds)
                            } else {
                                date_touch::date_panel(sheet_date, None)
                            }
                        },
                    );
                    kivact::control::column((
                        row((date_tab, time_tab)).style(style().gap(8)),
                        panel,
                    ))
                },
                action_label,
                move || {
                    if time_panel.get_untracked() {
                        open.set(false);
                    } else {
                        time_panel.set(true);
                    }
                },
            );
            return root
                .child(row((date_trigger, time_trigger)).style(style().gap(8).wrap(true)))
                .child(sheet)
                .into_view();
        }
        root.child(
            row((
                date_input(date_label, date).presentation(DatePresentation::Desktop),
                time_input(time_label, time)
                    .presentation(DatePresentation::Desktop)
                    .show_seconds(seconds),
            ))
            .style(style().align_items(Align::Center).gap(8)),
        )
        .into_view()
    }
}
