//! Role: Catalog page of Calendar: single, range, disabled dates and dropdown caption.

use crate::model::{Example, Page, PreviewAlign};
use kivact::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "calendar",
        name: "Calendar",
        description: "月の表から日付や期間を選ぶ部品です。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("calendar.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "single",
                "Single",
                "日付の binding を渡します。矢印キーで日、PageUp と PageDown で月を移り、Enter で選びます。",
                single,
            ),
            Example::new(
                "range",
                "Range",
                "calendar_range は二回の押下で期間を選びます。number_of_months で複数の月を並べます。",
                range,
            ),
            Example::new(
                "disabled",
                "Disabled dates",
                "disabled_dates に述語を渡すと、その日はフォーカスも選択もできません。ここでは週末です。",
                disabled_dates,
            ),
            Example::new(
                "dropdown",
                "Dropdown caption",
                "caption の Dropdown は、月と年を選択欄で選びます。",
                dropdown,
            ),
        ],
    }
}

// example: single
fn single() -> impl IntoView {
    let date = signal(Some(CalendarDate::new(2026, 10, 14)));
    basic::calendar(date).default_month(CalendarMonth::new(2026, 10))
}
// end example

// example: range
fn range() -> impl IntoView {
    let value = signal(Some(DateRange::new(
        CalendarDate::new(2026, 10, 5),
        CalendarDate::new(2026, 10, 9),
    )));
    basic::calendar_range(value)
        .default_month(CalendarMonth::new(2026, 10))
        .number_of_months(2)
}
// end example

// example: disabled
fn disabled_dates() -> impl IntoView {
    basic::calendar(signal(None))
        .default_month(CalendarMonth::new(2026, 10))
        .week_start(Weekday::Monday)
        .disabled_dates(|d| matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday))
}
// end example

// example: dropdown
fn dropdown() -> impl IntoView {
    basic::calendar(signal(None))
        .default_month(CalendarMonth::new(2026, 10))
        .caption(basic::CalendarCaption::Dropdown)
        .show_outside_days(false)
}
// end example
