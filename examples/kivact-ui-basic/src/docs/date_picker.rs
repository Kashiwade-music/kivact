//! Role: Catalog page of Date Picker: single, format, range and options.

use crate::model::{Example, Page, PreviewAlign};
use kivact::i18n::{CalendarDate, DateRange};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "date-picker",
        name: "Date Picker",
        description: "カレンダーから日付や期間を選ぶ部品です。ボタンの幅は表示する文字の幅で、固定の幅を持ちません。幅が必要なときは style で指定します。表示の形式は messages の date_picker.format（既定は {month} {day}, {year}）、期間の区切りは date_picker.range（既定は {start} - {end}）で差し替えられます。",
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
    basic::date_picker("Date of birth", date).style(style().width(280))
}
// end example

// example: format
fn format() -> impl IntoView {
    let date = signal(Some(CalendarDate::new(2026, 10, 3)));
    basic::date_picker("Due date", date)
        .placeholder("Pick a due date")
        .format(|d| format!("{:04}/{:02}/{:02}", d.year(), d.month(), d.day()))
}
// end example

// example: range
fn range() -> impl IntoView {
    let value = signal(None::<DateRange>);
    basic::date_range_picker("Stay", value)
        .style(style().width(300))
        .calendar(|c| c.number_of_months(2))
}
// end example

// example: weekdays
fn weekdays() -> impl IntoView {
    use kivact::i18n::Weekday;
    basic::date_picker("Meeting", signal(None)).calendar(|c| {
        c.disabled_dates(|d| matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday))
    })
}
// end example
