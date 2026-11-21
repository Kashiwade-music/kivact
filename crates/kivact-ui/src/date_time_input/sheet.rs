//! Role: Route touch date and time fields through one picker controller.
use kivact_core::i18n::{
    CalendarDate, DateStyle, DateTime, TimeStyle, Weekday, message, use_locale,
};
use kivact_core::state::{Signal, Value, binding};
use kivact_core::view::{
    Element, Handled, IntoView, Key, Node, View, column, dynamic, node_ref, row,
};
use kivact_interaction::{DatePanel, DatePicker, DatePickerApi, use_date_picker};
pub(crate) struct Inputs {
    pub root: Node,
    pub label: Value<String>,
    pub picker_label: Option<Value<String>>,
    pub value: Signal<Option<DateTime>>,
    pub format: Value<DateStyle>,
    pub min: Value<Option<DateTime>>,
    pub max: Value<Option<DateTime>>,
    pub week_start: Option<Value<Weekday>>,
    pub seconds: Value<bool>,
    pub variants: kivact_core::view::Variants,
}
pub(crate) fn render(i: Inputs) -> View {
    let button_ref = node_ref();
    let picker = use_date_picker();
    let open = picker.open(DatePanel::Date, button_ref);
    let restore = picker.return_focus(button_ref);
    let fields = crate::date_time_input::model::field_bindings(i.value, i.min, i.max);
    let date = fields.date;
    let time = fields.time;
    let editors = editors(date, time, i.format, i.seconds, open, picker);
    let trigger = crate::button(
        i.picker_label
            .unwrap_or_else(|| message("date_time.open", "Choose date and time")),
    )
    .node_ref(button_ref);
    let content = picker_content(i.min, i.max, i.week_start, i.seconds, date, time, picker);
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
    date: Signal<Option<CalendarDate>>,
    time: Signal<Option<kivact_core::i18n::TimeOfDay>>,
    picker: DatePickerApi,
) -> kivact_core::view::ViewFn {
    kivact_core::view::ViewFn::from(move |()| {
        dynamic(
            move || picker.state().get(),
            move |phase| match phase {
                DatePicker::Open {
                    panel: DatePanel::Date,
                    ..
                } => {
                    let mut calendar = crate::calendar(date)
                        .min(move || min.get().map(|v| v.date))
                        .max(move || max.get().map(|v| v.date))
                        .autofocus(true);
                    if let Some(day) = week_start {
                        calendar = calendar.week_start(day);
                    }
                    column((
                        crate::date_parts::sheet::panel(date, calendar),
                        crate::button(message("date_time.save_date", "Save date"))
                            .on_press(move || picker.show_panel(DatePanel::Time)),
                    ))
                    .into_view()
                }
                DatePicker::Open {
                    panel: DatePanel::Time,
                    ..
                } => column((
                    crate::date_parts::wheel::time_panel(time, seconds),
                    crate::button(message("date_time.save", "Save"))
                        .on_press(move || picker.hide()),
                ))
                .into_view(),
                DatePicker::Closed => kivact_core::view::node().into_view(),
            },
        )
    })
}

fn editors(
    date: Signal<Option<CalendarDate>>,
    time: Signal<Option<kivact_core::i18n::TimeOfDay>>,
    format: Value<DateStyle>,
    seconds: Value<bool>,
    open: Signal<bool>,
    picker: DatePickerApi,
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
    let field = crate::text_input(draft)
        .label(message("date_time.date", "Date"))
        .node_ref(date_ref)
        .with_node(|n| n.expanded(open))
        .forward_keys(vec![Key::ArrowDown])
        .on_key(move |event: kivact_core::view::KeyEvent| {
            if event.key() == Key::ArrowDown {
                picker.show(DatePanel::Date, date_ref);
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
            if let Some(parsed) =
                crate::time_input::parse::parse_time(&text, seconds.get_untracked())
            {
                time.set(Some(parsed));
            }
        },
    );
    let time_label = crate::text_input(time_draft)
        .label(message("date_time.time", "Time"))
        .node_ref(time_ref)
        .with_node(|n| n.expanded(open))
        .forward_keys(vec![Key::ArrowDown])
        .on_key(move |event: kivact_core::view::KeyEvent| {
            if event.key() == Key::ArrowDown {
                picker.show(DatePanel::Time, time_ref);
                Handled::Yes
            } else {
                Handled::No
            }
        });
    row((field, time_label))
}
