//! Role: Route touch date and time fields through one picker controller.
use crate::date_time_model::{Controller, DateTimeInputState, Event, Panel};
use kivact_core::i18n::{
    CalendarDate, DateStyle, DateTime, TimeStyle, Weekday, message, use_locale,
};
use kivact_core::state::{Binding, Value, binding, machine, memo};
use kivact_core::view::{
    Element, Handled, IntoView, Key, Node, View, column, dynamic, node_ref, row,
};
pub(crate) struct Inputs {
    pub root: Node,
    pub label: Value<String>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub picker_label: Option<Value<String>>,
    pub value: Binding<Option<DateTime>>,
    pub format: Value<DateStyle>,
    pub min: Value<Option<DateTime>>,
    pub max: Value<Option<DateTime>>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub week_start: Option<Value<Weekday>>,
    pub seconds: Value<bool>,
    pub variants: crate::variants::Variants,
}
pub(crate) fn render(i: Inputs) -> View {
    let button_ref = node_ref();
    let state = machine(DateTimeInputState::Closed, || Controller, |_, ()| {});
    let open = binding(
        move || matches!(state.get(), DateTimeInputState::Open { .. }),
        move |open| {
            state.send(if open {
                Event::Open(Panel::Date, button_ref)
            } else {
                Event::Close
            })
        },
    );
    let restore = memo(move || match state.get() {
        DateTimeInputState::Open { invoker, .. } => invoker,
        _ => button_ref,
    });
    let fields = crate::date_time_model::field_bindings(i.value, i.min, i.max);
    let date = fields.date;
    let time = fields.time;
    let editors = editors(date, time, i.format, i.seconds, open, state);
    let trigger = crate::button(
        i.picker_label
            .unwrap_or_else(|| message("date_time.open", "Choose date and time")),
    )
    .node_ref(button_ref);
    let content = picker_content(i.min, i.max, i.week_start, i.seconds, date, time, state);
    i.root
        .child(row((
            editors,
            crate::dialog(i.label, open)
                .with_variants(&i.variants)
                .mode(crate::DatePresentation::BottomSheet.mode())
                .return_focus(restore)
                .content(move || content.call(()))
                .attach(trigger),
        )))
        .into_view()
}

fn picker_content(
    min: Value<Option<DateTime>>,
    max: Value<Option<DateTime>>,
    week_start: Option<Value<Weekday>>,
    seconds: Value<bool>,
    date: Binding<Option<CalendarDate>>,
    time: Binding<Option<kivact_core::i18n::TimeOfDay>>,
    state: kivact_core::state::MachineHandle<Controller>,
) -> kivact_core::view::ViewFn {
    kivact_core::view::ViewFn::from(move |()| {
        dynamic(
            move || state.get(),
            move |phase| match phase {
                DateTimeInputState::Open {
                    panel: Panel::Date, ..
                } => {
                    let mut picker = crate::calendar(date)
                        .min(move || min.get().map(|v| v.date))
                        .max(move || max.get().map(|v| v.date))
                        .autofocus(true);
                    if let Some(day) = week_start {
                        picker = picker.week_start(day);
                    }
                    column((
                        crate::date_sheet::panel(date, picker),
                        crate::button(message("date_time.save_date", "Save date"))
                            .on_press(move || state.send(Event::Switch(Panel::Time))),
                    ))
                    .into_view()
                }
                DateTimeInputState::Open {
                    panel: Panel::Time, ..
                } => column((
                    crate::date_wheel::time_panel(time, seconds),
                    crate::button(message("date_time.save", "Save"))
                        .on_press(move || state.send(Event::Close)),
                ))
                .into_view(),
                DateTimeInputState::Closed => kivact_core::view::node().into_view(),
            },
        )
    })
}

fn editors(
    date: Binding<Option<CalendarDate>>,
    time: Binding<Option<kivact_core::i18n::TimeOfDay>>,
    format: Value<DateStyle>,
    seconds: Value<bool>,
    open: Binding<bool>,
    state: kivact_core::state::MachineHandle<Controller>,
) -> Node {
    let date_ref = node_ref();
    let time_ref = node_ref();
    let locale = use_locale();
    let draft = binding(
        move || {
            date.get()
                .map(|date| locale.get().format_date(date, format.get()))
                .unwrap_or_default()
        },
        move |text: String| {
            if let Some(parsed) = locale.get_untracked().parse_date(&text) {
                date.set(Some(parsed));
            }
        },
    );
    let field = crate::text_input(message("date_time.date", "Date"), draft)
        .node_ref(date_ref)
        .with_node(|n| n.expanded(open))
        .forward_keys(vec![Key::ArrowDown])
        .on_key(move |event: kivact_core::view::KeyEvent| {
            if event.key() == Key::ArrowDown {
                state.send(Event::Open(Panel::Date, date_ref));
                Handled::Yes
            } else {
                Handled::No
            }
        });
    let time_draft = binding(
        move || {
            time.get()
                .map(|time| {
                    locale.get().format_time(
                        time,
                        TimeStyle {
                            seconds: seconds.get(),
                        },
                    )
                })
                .unwrap_or_default()
        },
        move |text: String| {
            if let Some(parsed) = crate::time_parse::parse_time(&text, seconds.get_untracked()) {
                time.set(Some(parsed));
            }
        },
    );
    let time_label = crate::text_input(message("date_time.time", "Time"), time_draft)
        .node_ref(time_ref)
        .with_node(|n| n.expanded(open))
        .forward_keys(vec![Key::ArrowDown])
        .on_key(move |event: kivact_core::view::KeyEvent| {
            if event.key() == Key::ArrowDown {
                state.send(Event::Open(Panel::Time, time_ref));
                Handled::Yes
            } else {
                Handled::No
            }
        });
    row((field, time_label))
}
