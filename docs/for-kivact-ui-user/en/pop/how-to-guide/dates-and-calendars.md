# Pick dates and ranges

[日本語](../../../jp/pop/how-to-guide/dates-and-calendars.md)

Install the Pop theme and use the standard `kivact::ui` calendar and date fields. The application owns an `Option<CalendarDate>` or `Option<DateRange>` binding; `None` represents an empty field.

## Edit dates

```rust,no_run
use kivact::i18n::{CalendarDate, DateRange};
use kivact::prelude::*;

fn dates() -> impl IntoView {
    let today = CalendarDate::today();
    let due = signal(Some(today));
    let stay = signal(Some(DateRange::new(today, today.add_days(2))));
    column((
        kivact::ui::date_input("Due date", due)
            .calendar(move |calendar| calendar.today(today)),
        kivact::ui::date_range_input("Stay", stay)
            .start_label("Check-in")
            .end_label("Check-out")
            .calendar(move |calendar| calendar.today(today).number_of_months(1usize)),
    )).style(style().gap(12))
}
```

The text field publishes a parsed value and `.valid(binding)` reports draft validity. A partial draft remains visible while editing. `.format(DateStyle)` and `.format_with(..)` control presentation. `.min(..)`, `.max(..)`, `.week_start(..)` and `.calendar(..)` configure constraints and the shared calendar. The range field preserves custom endpoint labels through `.start_label(..)` and `.end_label(..)`.

A calendar range commits to the caller only after both endpoints are chosen. Closing its popup earlier keeps the original range. The calendar announces “Pick a start date” and “Pick an end date” from its range owner. `.presentation(DatePresentation::Desktop)` selects a desktop popup; `BottomSheet` selects a touch sheet and `Adaptive` uses platform capabilities.

## Show a calendar

```rust,no_run
use kivact::i18n::{CalendarDate, DateRange, Weekday};
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn schedule() -> impl IntoView {
    let today = CalendarDate::today();
    let day = signal(Some(today));
    kivact::ui::calendar(day)
        .today(today)
        .week_start(Weekday::Monday)
        .year_navigation(true)
        .caption(kivact::ui::CalendarCaption::Dropdown)
        .month_title(|date| format!("{}-{:02}", date.year(), date.month()))
        .events(vec![kivact::ui::CalendarEvent {date: today, color: rgb(0x2b8bd8)}])
        .range(Some(DateRange::new(today.add_days(5), today.add_days(9))))
        .variant(pop::WeekendHue::new(|day| match day {
            Weekday::Saturday | Weekday::Sunday => Some(pop::Hue::Pink),
            _ => None,
        }))
        .on_activate(|date| println!("opened {date}"))
}
```

`.week_start(..)` sets the first column; the locale supplies the default. `.caption(CalendarCaption::Dropdown)` replaces the month title with month and year menus. `.year_navigation(true)` adds year buttons; Shift with PageUp or PageDown also changes the year. `.month_title(..)` customizes the text title.

`.events(..)` draws up to three event dots per date. `.range(Some(DateRange::new(first, last)))` adds a range highlight. `pop::WeekendHue` preserves every Pop hue, including reactive application values read by the weekday callback. The canonical `.weekend(..)` and `.marks(..)` accept semantic `Tone` values. `.on_activate(..)` runs even if the activated date was already selected.

## Language

Month and weekday names follow the locale. Supply canonical message keys such as `date.open`, `date.range_open`, `date.start`, `date.end`, `calendar.previous`, `calendar.next`, `calendar.previous_year`, `calendar.next_year`, `calendar.range.pick_start` and `calendar.range.pick_end`.

```rust,no_run
use kivact::i18n::{CalendarDate, Locale, Messages};
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;
fn picker() -> impl IntoView { kivact::ui::date_input("Date", signal(Some(CalendarDate::today()))) }
fn main() -> kivact::Result<()> {
    App::new("Dates")
        .locale(Locale::new("ja-JP"))
        .messages(Messages::new().with("date.open", "日付を選択").with("date.range_open", "期間を選択"))
        .theme(pop::theme(ThemeMode::Light))
        .run(picker)
}
```

See [theme and tokens](theme-and-tokens.md) for appearance and [validation](../getting-started/03-validate-and-submit.md) for application-owned rules.
