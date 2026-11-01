# Pick dates and ranges

[日本語](../../../jp/pop/how-to-guide/dates-and-calendars.md)

Use this guide to let people type or pick a date, a date range, or a day on a
calendar. All three components take their values as signals, so the dates stay
in your state.

## Ask for one date

`pop::date_input(label, value, today)` is a text field with a calendar popup.
`value` is a `Signal<CalendarDate>` (or any binding of one) and you give
`today` yourself.

```rust,no_run
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn due_date() -> impl IntoView {
    let today = CalendarDate::today();
    let due = signal(today.add_days(7));
    let valid = signal(true);
    column((
        pop::date_input("Due date", due, today).valid(valid),
        button("Save")
            .disabled(move || !valid.get())
            .on_press(move || println!("due {}", due.get())),
    ))
    .style(style().gap(12).padding(24))
}
# fn main() {}
```

The label names the text field for screen readers; draw it yourself if you
want it visible (wrap the input in `field("Due date", ..)` or put a `text`
above it). The button that opens the calendar is named "Choose a date".

What the text field does:

- It shows the date as `YYYY-MM-DD`. People can also type `YYYY/MM/DD`: four
  digits for the year, two for the month and two for the day.
- The signal changes only when the text is a whole date that exists.
  While someone types `2027-1`, your state still holds the previous date.
- A text that is not a date turns the field red and shows "Use YYYY-MM-DD" or,
  for a date such as `2027-02-30`, "This date does not exist". The start of a
  date is not an error until the field loses focus or Enter is pressed.
- `.valid(signal)` writes `true` while the text is a date and `false` otherwise.
  Use it to block saving; the component never blocks anything itself.
- Choosing a day in the calendar writes the text and the signal and closes
  the popup. Escape or a click outside closes it without a change.

Rules that depend on your data (a date in the past, a weekend, a booked day)
are yours. Check the signal and show an error beside the input, as the
[tutorial](../getting-started/03-validate-and-submit.md) does. Only check the
rule while `valid` is `true`, because the signal keeps the last good date.

### Make the date optional

A date input always holds a date. For "no date", keep a switch beside it
and derive the optional value:

```rust,no_run
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn deadline() -> impl IntoView {
    let today = CalendarDate::today();
    let enabled = signal(false);
    let date = signal(today);
    let deadline = memo(move || enabled.get().then(|| date.get()));
    column((
        checkbox("Set a deadline", enabled),
        show(enabled, move || pop::date_input("Deadline", date, today)),
        text(move || match deadline.get() {
            Some(day) => format!("Deadline: {day}"),
            None => "No deadline".to_owned(),
        }),
    ))
    .style(style().gap(12).padding(24))
}
# fn main() {}
```

## Ask for a range

`pop::date_range_input((start_label, end_label), (start, end), today)` shows
two fields over one calendar.

```rust,no_run
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn stay() -> impl IntoView {
    let today = CalendarDate::today();
    let from = signal(today);
    let to = signal(today.add_days(2));
    let valid = signal(true);
    column((
        pop::date_range_input(("Check-in", "Check-out"), (from, to), today).valid(valid),
        text(move || format!("{} nights", to.get().to_epoch_days() - from.get().to_epoch_days())),
    ))
    .style(style().gap(12).padding(24))
}
# fn main() {}
```

Each field follows the rules above. In the calendar, the first day you choose
sets both ends and waits for the end; the second sets the end, or swaps the
two when it is before the start, then the popup closes. The range is shaded.
If someone types a start after the end, both fields turn red, the message
"The end date is before the start date" appears and `valid` becomes `false`.
While the popup is open it announces "Pick a start date" and "Pick an end
date" politely, and its Close button returns focus to the start field.

## Show a calendar on its own

`pop::calendar(selected, today)` is the month grid without the text field. Use
it for a schedule or a booking screen. These builders cover most needs:

```rust,no_run
use kivact::i18n::{CalendarDate, Weekday};
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn schedule() -> impl IntoView {
    let today = CalendarDate::today();
    let day = signal(today);
    let events = vec![
        pop::CalendarEvent { date: today, color: rgb(0x2b8bd8) },
        pop::CalendarEvent { date: today.add_days(3), color: rgb(0xef4458) },
    ];
    let trip = Some((today.add_days(5), today.add_days(9)));
    column((
        pop::calendar(day, today)
            .week_starts_on(Weekday::Monday)
            .year_navigation(true)
            .events(events)
            .range(trip)
            .weekend(|weekday| match weekday {
                Weekday::Saturday | Weekday::Sunday => Some(pop::Hue::Pink),
                _ => None,
            })
            .on_activate(|date| println!("opened {date}")),
        text(move || format!("Selected: {}", day.get())),
    ))
    .style(style().gap(12).padding(24))
}
# fn main() {}
```

- `.week_starts_on(Weekday)` sets the first column. Without it the grid starts
  on the first day of the week of the App's locale.
- `.year_navigation(true)` adds previous and next year buttons next to the
  month buttons; Shift with PageUp or PageDown does the same from the
  keyboard. `.year_month_menus(true)` replaces the title with a year select
  and a month select, covering 50 years either side.
- `.events(..)` draws up to three colored dots per day. The calendar only
  draws them; what an event is stays in your model. Pass a signal of
  `Vec<pop::CalendarEvent>` to update the dots.
- `.range(Some((first, last)))` shades the days from `first` to `last`,
  ends included.
- `.weekend(..)` colors weekdays. By default every day has the same color.
- `.on_activate(..)` runs when someone clicks a day or presses Enter on it,
  even when that day is already selected.
- `.month_title(|first_of_month| String)` replaces the title text.

Every builder that takes a flag or a list also accepts a signal.

## Language and texts

The weekday headings, month buttons, the "Choose a date" button and the date
errors are English until you replace them. Set the App's locale and messages;
the keys start with `pop.`:

```rust,no_run
use kivact::i18n::{CalendarDate, Locale, Messages};
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

fn picker() -> impl IntoView {
    let today = CalendarDate::today();
    pop::date_input("Date", signal(today), today)
}

fn main() -> kivact::Result<()> {
    let messages = Messages::new()
        .with("pop.date.open", "日付を選択")
        .with("pop.date.error.format", "YYYY-MM-DD の形式で入力してください")
        .with("pop.date.error.invalid", "存在しない日付です")
        .with("pop.date.error.order", "終了日が開始日より前です")
        .with("pop.calendar.title", "{year}.{month}");
    App::new("Dates")
        .locale(Locale::new("ja-JP"))
        .messages(messages)
        .theme(pop::theme(ThemeMode::Light))
        .run(picker)
}
```

Besides the keys above, the calendar reads `pop.calendar.sun` through
`pop.calendar.sat` (weekday headings), `pop.calendar.previous`, `.next`,
`.previous_year`, `.next_year`, `.year` and `.month`, and the range popup
reads `pop.date.range.pick_start`, `pop.date.range.pick_end` and
`pop.date.range.close`. The locale also decides the first day of the week.

## See also

- [Set up the Pop theme and tokens](theme-and-tokens.md) for the colors of the
  calendar and its events
- [Validate and submit](../getting-started/03-validate-and-submit.md) in the
  tutorial, which combines dates with field errors
- `kivact::i18n::CalendarDate` (`new`, `checked`, `today`, `add_days`,
  `add_months`, `weekday`) in `cargo doc -p kivact --open`, and the rest of
  the Pop API in `cargo doc -p kivact-ui-pop --open`
