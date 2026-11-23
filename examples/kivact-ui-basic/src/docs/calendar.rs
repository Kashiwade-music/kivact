//! Catalog page of Calendar: single, range, disabled dates and dropdown caption.

use catalog_shell::{Example, Page};
use kivact::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page::new("calendar", "Calendar", "Components", include_str!("calendar.rs"))
        .description("A month grid for choosing a date or a range of dates.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "single",
            "Single",
            "Pass a binding for the date. The arrow keys move by day, PageUp and PageDown move by month, and Enter selects.",
            single,
        ))
        .example(Example::new(
            "range",
            "Range",
            "calendar_range selects a range with two presses. number_of_months shows several months side by side.",
            range,
        ))
        .example(Example::new(
            "disabled",
            "Disabled dates",
            "A predicate passed to disabled_dates makes those days unfocusable and unselectable. Here the weekends are disabled.",
            disabled_dates,
        ))
        .example(Example::new(
            "dropdown",
            "Dropdown caption",
            "The Dropdown in the caption chooses the month and the year.",
            dropdown,
        ))
}

// example: single
fn single() -> impl IntoView {
    let date = signal(Some(CalendarDate::new(2026, 10, 14)));
    kivact::ui::calendar(date)
        .variable_rows(true)
        .default_month(CalendarMonth::new(2026, 10))
}
// end example

// example: range
fn range() -> impl IntoView {
    let value = signal(Some(DateRange::new(
        CalendarDate::new(2026, 10, 5),
        CalendarDate::new(2026, 10, 9),
    )));
    kivact::ui::calendar(value)
        .variable_rows(true)
        .default_month(CalendarMonth::new(2026, 10))
        .number_of_months(2)
}
// end example

// example: disabled
fn disabled_dates() -> impl IntoView {
    kivact::ui::calendar(signal(None::<CalendarDate>))
        .variable_rows(true)
        .default_month(CalendarMonth::new(2026, 10))
        .week_start(Weekday::Monday)
        .disabled_dates(|d| matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday))
}
// end example

// example: dropdown
fn dropdown() -> impl IntoView {
    kivact::ui::calendar(signal(None::<CalendarDate>))
        .variable_rows(true)
        .default_month(CalendarMonth::new(2026, 10))
        .caption(kivact::ui::CalendarCaption::Dropdown)
        .show_outside_days(false)
}
// end example
