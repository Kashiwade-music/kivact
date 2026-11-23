//! Role: The date-time field's component, its controller wiring and desktop picker.

use crate::date_time_input::look::DateTimeInputTheme;
use crate::date_time_input::model::{DateTimeFields, field_bindings};
use crate::{DatePresentation, button, calendar, date_input, dialog};
use kivact_core::custom::Caller;
use kivact_core::i18n::{DateStyle, DateTime, TimeStyle, Weekday, message};
use kivact_core::state::{IntoValue, Signal, Value};
use kivact_core::view::{Element, IntoView, Node, column, node, row};
use kivact_interaction::{DatePanel, DatePicker, DatePickerApi, use_date_picker};
use kivact_macros::component;

/// Edit an optional local date and time. A Gregorian date and wall-clock time,
/// with no time zone conversion.
///
/// Setters: `label` (the name of the date and time group and of its picker,
/// default "Choose date and time"; the date and time fields are named by
/// their own messages), `picker_label` (the separate picker button's name),
/// `format` (the date text format), `min` and `max` (the accepted date and
/// time), `week_start` (the first weekday column), `presentation` (the picker
/// surface) and `time_format` (the wall-clock precision, independent of the
/// date format).
#[component]
pub fn date_time_input(
    value: Signal<Option<DateTime>>,
    label: Option<Value<String>>,
    picker_label: Option<Value<String>>,
    #[prop(default = DateStyle::Iso)] format: Value<DateStyle>,
    #[prop(default = None::<DateTime>)] min: Value<Option<DateTime>>,
    #[prop(default = None::<DateTime>)] max: Value<Option<DateTime>>,
    week_start: Option<Value<Weekday>>,
    #[prop(default = DatePresentation::Adaptive)] presentation: DatePresentation,
    time_format: Option<Value<TimeStyle>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let label = label.unwrap_or_else(|| message("date_time.open", "Choose date and time"));
    let seconds: Value<bool> = match time_format {
        Some(format) => (move || format.get().seconds).into_value(),
        None => false.into_value(),
    };
    if presentation.sheet() {
        return crate::date_time_input::sheet::render(crate::date_time_input::sheet::Inputs {
            root: node(),
            label,
            picker_label,
            value,
            format,
            min,
            max,
            week_start,
            seconds,
            variants,
        });
    }
    let fields = field_bindings(value, min, max);
    let editors = desktop_editors(
        DateTimeFields {
            date: fields.date,
            time: fields.time,
        },
        (format, seconds),
        min,
        max,
        week_start,
        presentation,
        &variants,
    );
    let (picker, open) = picker_controller();
    let trigger =
        button(picker_label.unwrap_or_else(|| message("date_time.open", "Choose date and time")));
    let content = desktop_content(fields, min, max, week_start, presentation, seconds, picker);
    let root = crate::layer::with_base_recipe_style(
        node(),
        |r: &DateTimeInputTheme| r.root_recipe(),
        &variants,
        kivact_core::custom::PartState::default,
    );
    root.name(label).child(column((
        editors,
        dialog(label, open)
            .with_variants(&variants)
            .mode(presentation.mode())
            .content(move || content.call(()))
            .attach(trigger),
    )))
}

/// The picker's open state and panel, opened from the dialog's trigger.
fn picker_controller() -> (DatePickerApi, Signal<bool>) {
    let picker = use_date_picker();
    let invoker = kivact_core::view::node_ref();
    (picker, picker.open(DatePanel::Date, invoker))
}

/// Switch the canonical date/time picker panel from its visible tab actions.
fn picker_tabs(picker: DatePickerApi) -> Node {
    row((
        button(message("date_time.date", "Date"))
            .on_press(move || picker.show_panel(DatePanel::Date)),
        button(message("date_time.time", "Time"))
            .on_press(move || picker.show_panel(DatePanel::Time)),
    ))
}

fn desktop_editors(
    fields: DateTimeFields,
    formats: (Value<DateStyle>, Value<bool>),
    min: Value<Option<DateTime>>,
    max: Value<Option<DateTime>>,
    week_start: Option<Value<Weekday>>,
    presentation: DatePresentation,
    variants: &kivact_core::view::Variants,
) -> Node {
    let DateTimeFields { date, time } = fields;
    let (format, seconds) = formats;
    let mut date_field = date_input(date)
        .label(message("date_time.date", "Date"))
        .format(format)
        .min(move || min.get().map(|value| value.date))
        .max(move || max.get().map(|value| value.date))
        .presentation(presentation)
        .with_variants(variants);
    if let Some(day) = week_start {
        date_field = date_field.week_start(day);
    }
    let no_bounds: Value<Option<kivact_core::i18n::TimeOfDay>> = None.into_value();
    let time_field = crate::time_input::time_field(
        time,
        message("date_time.time", "Time"),
        None,
        seconds,
        (no_bounds, no_bounds),
        presentation,
        variants,
    );
    row((date_field, time_field))
}

fn desktop_content(
    fields: DateTimeFields,
    min: Value<Option<DateTime>>,
    max: Value<Option<DateTime>>,
    week_start: Option<Value<Weekday>>,
    presentation: DatePresentation,
    seconds: Value<bool>,
    picker: DatePickerApi,
) -> kivact_core::view::ViewFn {
    let DateTimeFields { date, time } = fields;
    kivact_core::view::ViewFn::from(move |()| {
        let tabs = picker_tabs(picker);
        let body = kivact_core::view::dynamic(
            move || picker.state().get(),
            move |phase| match phase {
                DatePicker::Open {
                    panel: DatePanel::Date,
                    ..
                } => {
                    let mut picker = calendar(date)
                        .min(move || min.get().map(|value| value.date))
                        .max(move || max.get().map(|value| value.date));
                    if let Some(day) = week_start {
                        picker = picker.week_start(day);
                    }
                    if presentation.sheet() {
                        crate::date_parts::sheet::panel(date, picker).into_view()
                    } else {
                        picker.into_view()
                    }
                }
                DatePicker::Open {
                    panel: DatePanel::Time,
                    ..
                } => crate::date_parts::wheel::time_panel(time, seconds).into_view(),
                DatePicker::Closed => node().into_view(),
            },
        );
        column((
            tabs,
            body,
            button(message("date_time.done", "Done")).on_press(move || picker.hide()),
        ))
    })
}
