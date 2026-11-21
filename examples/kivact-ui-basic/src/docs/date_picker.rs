//! Catalog page of Date Picker: single, format, range and options.

use catalog_shell::{Example, Page};
use kivact::i18n::{CalendarDate, DateRange};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("date-picker", "Date Picker", "Components", include_str!("date_picker.rs"))
        .description("Chooses a date or a range of dates from a calendar. The width of the button is the width of the text it shows, so it has no fixed width; set one with style when you need it. The text and its format are set by the button passed to attach.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pressing the button opens a calendar popover; picking a day closes it and shows the value. The width is 280 through style.",
            basic,
        ))
        .example(Example::new(
            "format",
            "Format",
            "format changes the display. placeholder is the text shown while nothing is picked.",
            format,
        ))
        .example(Example::new(
            "range",
            "Range",
            "date_range_picker closes on the second press and shows 'start - end'.",
            range,
        ))
        .example(Example::new(
            "weekdays",
            "Weekdays only",
            "calendar sets the calendar it opens. Here Sundays and Saturdays cannot be chosen.",
            weekdays,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let date = signal(None::<CalendarDate>);
    let locale = kivact::i18n::use_locale();
    kivact::ui::date_input(date)
        .label("Date of birth")
        .presentation(kivact::ui::DatePresentation::Desktop)
        .variant(basic::DatePickerVariant::Flush)
        .calendar(|c| c.variable_rows(true))
        .attach(
            button(move || {
                date.get().map_or_else(
                    || "Pick a date".to_owned(),
                    |date| {
                        locale
                            .get()
                            .format_date(date, kivact::i18n::DateStyle::Long)
                    },
                )
            })
            .name("Date of birth")
            .icon(icon("calendar"))
            .variant(kivact::ui::ButtonVariant::Outline)
            .style(
                style()
                    .width(280)
                    .justify_content(Justify::Start)
                    .font_weight(400),
            ),
        )
}
// end example

// example: format
fn format() -> impl IntoView {
    let date = signal(Some(CalendarDate::new(2026, 10, 3)));
    kivact::ui::date_input(date)
        .label("Due date")
        .presentation(kivact::ui::DatePresentation::Desktop)
        .variant(basic::DatePickerVariant::Flush)
        .calendar(|c| c.variable_rows(true))
        .attach(
            button(move || {
                date.get().map_or_else(
                    || "Pick a due date".to_owned(),
                    |d| format!("{:04}/{:02}/{:02}", d.year(), d.month(), d.day()),
                )
            })
            .name("Due date")
            .icon(icon("calendar"))
            .variant(kivact::ui::ButtonVariant::Outline),
        )
}
// end example

// example: range
fn range() -> impl IntoView {
    let value = signal(None::<DateRange>);
    let locale = kivact::i18n::use_locale();
    kivact::ui::date_range_input(value)
        .label("Stay")
        .presentation(kivact::ui::DatePresentation::Desktop)
        .variant(basic::DatePickerVariant::Flush)
        .calendar(|c| c.number_of_months(2).variable_rows(true))
        .attach(
            button(move || {
                value.get().map_or_else(
                    || "Pick a date".to_owned(),
                    |range| {
                        format!(
                            "{} - {}",
                            locale
                                .get()
                                .format_date(range.start(), kivact::i18n::DateStyle::Long),
                            locale
                                .get()
                                .format_date(range.end(), kivact::i18n::DateStyle::Long)
                        )
                    },
                )
            })
            .name("Stay")
            .icon(icon("calendar"))
            .variant(kivact::ui::ButtonVariant::Outline)
            .style(style().width(300)),
        )
}
// end example

// example: weekdays
fn weekdays() -> impl IntoView {
    use kivact::i18n::Weekday;
    let date = signal(None::<CalendarDate>);
    kivact::ui::date_input(date)
        .label("Meeting")
        .presentation(kivact::ui::DatePresentation::Desktop)
        .variant(basic::DatePickerVariant::Flush)
        .calendar(|c| {
            c.variable_rows(true)
                .disabled_dates(|d| matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday))
        })
        .attach(
            button("Pick a date")
                .name("Meeting")
                .icon(icon("calendar"))
                .variant(kivact::ui::ButtonVariant::Outline),
        )
}
// end example
