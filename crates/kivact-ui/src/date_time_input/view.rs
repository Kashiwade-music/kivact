//! Role: The date-time field's builder, its controller wiring and desktop picker.

use crate::date_time_input::look::DateTimeInputTheme;
use crate::date_time_input::model::{DateTimeFields, field_bindings};
use crate::{DatePresentation, button, calendar, date_input, dialog, time_input};
use kivact_core::i18n::{DateStyle, DateTime, TimeStyle, Weekday, message};
use kivact_core::state::{IntoText, IntoValue, Signal, Value};
use kivact_core::view::{Element, IntoView, Node, View, column, node, row};
use kivact_interaction::{DatePanel, DatePicker, DatePickerApi, use_date_picker};
// xtask-allow(state): legacy: picker_label, week_start
/// A Gregorian date and wall-clock time, with no time zone conversion.
pub struct DateTimeInput {
    root: Node,
    label: Value<String>,
    picker_label: Option<Value<String>>,
    value: Signal<Option<DateTime>>,
    format: Value<DateStyle>,
    min: Value<Option<DateTime>>,
    max: Value<Option<DateTime>>,
    week_start: Option<Value<Weekday>>,
    presentation: DatePresentation,
    seconds: Value<bool>,
}
/// Edit an optional local date and time.
#[track_caller]
pub fn date_time_input(value: Signal<Option<DateTime>>) -> DateTimeInput {
    DateTimeInput {
        root: node(),
        label: message("date_time.open", "Choose date and time"),
        picker_label: None,
        value,
        format: DateStyle::Iso.into_value(),
        min: None.into_value(),
        max: None.into_value(),
        week_start: None,
        presentation: DatePresentation::Adaptive,
        seconds: false.into_value(),
    }
}
impl DateTimeInput {
    /// The name of the date and time group and of its picker (default
    /// "Choose date and time"). The date and the time fields are named by
    /// their own messages.
    #[track_caller]
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = label.into_text_value();
        self
    }

    /// Name the separate picker button independently of the text editor.
    #[track_caller]
    pub fn picker_label(mut self, label: impl IntoText) -> Self {
        self.picker_label = Some(label.into_text_value());
        self
    }

    /// Date text format.
    #[track_caller]
    pub fn format(mut self, format: impl IntoValue<DateStyle>) -> Self {
        self.format = format.into_value();
        self
    }
    /// Earliest accepted date and time.
    #[track_caller]
    pub fn min(mut self, min: impl IntoValue<Option<DateTime>>) -> Self {
        self.min = min.into_value();
        self
    }
    /// Latest accepted date and time.
    #[track_caller]
    pub fn max(mut self, max: impl IntoValue<Option<DateTime>>) -> Self {
        self.max = max.into_value();
        self
    }
    /// First weekday column.
    #[track_caller]
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
    #[track_caller]
    pub fn time_format(mut self, format: impl IntoValue<TimeStyle>) -> Self {
        let format = format.into_value();
        self.seconds = (move || format.get().seconds).into_value();
        self
    }
}
impl Element for DateTimeInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for DateTimeInput {
    fn into_view(self) -> View {
        let variants = self.root.authored().variants();
        if self.presentation.sheet() {
            return crate::date_time_input::sheet::render(crate::date_time_input::sheet::Inputs {
                root: self.root,
                label: self.label,
                picker_label: self.picker_label,
                value: self.value,
                format: self.format,
                min: self.min,
                max: self.max,
                week_start: self.week_start,
                seconds: self.seconds,
                variants,
            });
        }
        let Self {
            root,
            label,
            picker_label,
            value,
            format,
            min,
            max,
            week_start,
            presentation,
            seconds,
        } = self;
        let variants = root.authored().variants();
        let DateTimeFields { date, time } = field_bindings(value, min, max);
        let editors = desktop_editors(
            DateTimeFields { date, time },
            (format, seconds),
            min,
            max,
            week_start,
            presentation,
            &variants,
        );
        let (picker, open) = picker_controller();
        let trigger = button(
            picker_label.unwrap_or_else(|| message("date_time.open", "Choose date and time")),
        );
        let content = desktop_content(
            DateTimeFields { date, time },
            min,
            max,
            week_start,
            presentation,
            seconds,
            picker,
        );
        let root = crate::layer::with_base_recipe_style(
            root,
            |r: &DateTimeInputTheme| r.root_recipe(),
            &variants,
            kivact_core::custom::PartState::default,
        );
        root.name(label)
            .child(column((
                editors,
                dialog(label, open)
                    .with_variants(&variants)
                    .mode(presentation.mode())
                    .content(move || content.call(()))
                    .attach(trigger),
            )))
            .into_view()
    }
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
    let time_field = time_input(time)
        .label(message("date_time.time", "Time"))
        .format(move || TimeStyle {
            seconds: seconds.get(),
        })
        .presentation(presentation)
        .with_variants(variants);
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
