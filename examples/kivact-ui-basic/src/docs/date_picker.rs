//! Role: Catalog page of Date Picker: single, format, range and options.

use crate::model::{Example, Page, PreviewAlign};
use kivact::i18n::{CalendarDate, DateRange};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "date-picker",
        name: "Date Picker",
        description: "カレンダーから日付や期間を選ぶ部品です。ボタンの幅は表示する文字の幅で、固定の幅を持ちません。幅が必要なときは style で指定します。表示する文字と書式は attach に渡すボタンで指定します。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("date_picker.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "ボタンを押すとカレンダーのポップオーバーが開き、日を選ぶと閉じて値を表示します。幅は style で 280 にしています。",
                basic,
            ),
            Example::new(
                "format",
                "Format",
                "format で表示を変えます。placeholder は未選択の文言です。",
                format,
            ),
            Example::new(
                "range",
                "Range",
                "date_range_picker は二回目の押下で閉じ、「開始 - 終了」を表示します。",
                range,
            ),
            Example::new(
                "weekdays",
                "Weekdays only",
                "calendar で開くカレンダーを設定します。ここでは日曜と土曜を選べなくします。",
                weekdays,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let date = signal(None::<CalendarDate>);
    let locale = kivact::i18n::use_locale();
    kivact::ui::date_input("Date of birth", date)
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
                    .weight(400),
            ),
        )
}
// end example

// example: format
fn format() -> impl IntoView {
    let date = signal(Some(CalendarDate::new(2026, 10, 3)));
    kivact::ui::date_input("Due date", date)
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
    kivact::ui::date_range_input("Stay", value)
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
    kivact::ui::date_input("Meeting", date)
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
