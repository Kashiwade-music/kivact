//! Role: The calendar, the date input and the date range input.

use kivact::i18n::CalendarDate;
use kivact::prelude::*;

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(year, month, day)
}

#[test]
fn the_date_range_writes_only_a_complete_range() {
    let (screen, (start, end)) = kivact::test::render_with_handle(
        crate::support::options().messages(
            kivact::i18n::Messages::new()
                .with("date.open", "Choose endpoint date")
                .with("date.range_open", "Choose a date"),
        ),
        || {
            let today = date(2026, 10, 3);
            let (start, end) = (signal(today), signal(date(2026, 10, 5)));
            ((start, end), {
                let (start_label, end_label) = ("From", "To");
                let start_label = kivact::state::IntoText::into_text_value(start_label);
                let end_label = kivact::state::IntoText::into_text_value(end_label);
                let (start, end) = (start, end);
                let start: kivact::state::Binding<kivact::i18n::CalendarDate> =
                    kivact::state::IntoBinding::into_binding(start);
                let end: kivact::state::Binding<kivact::i18n::CalendarDate> =
                    kivact::state::IntoBinding::into_binding(end);
                let range = kivact::state::binding(
                    move || Some(kivact::i18n::DateRange::new(start.get(), end.get())),
                    move |range: Option<kivact::i18n::DateRange>| {
                        if let Some(range) = range {
                            start.set(range.start());
                            end.set(range.end());
                        }
                    },
                );
                kivact::ui::date_range_input(
                    move || format!("{} – {}", start_label.get(), end_label.get()),
                    range,
                )
                .start_label(start_label)
                .end_label(end_label)
                .calendar(move |calendar| {
                    calendar
                        .today(today)
                        .number_of_months(1usize)
                        .month_title(|date| format!("{}-{:02}", date.year(), date.month()))
                })
            })
        },
    );
    let opener = screen.get_by_role(Role::Button, "Choose a date");
    opener.click();
    screen.get_by_role(Role::GridCell, "2026-10-10").click();
    screen.press_key(Key::Escape);
    assert_eq!(
        (start.get_untracked(), end.get_untracked()),
        (date(2026, 10, 3), date(2026, 10, 5)),
        "closing before the second day leaves both dates"
    );
    opener.click();
    screen.get_by_text("Pick a start date");
    screen.get_by_role(Role::GridCell, "2026-10-15").click();
    screen.get_by_role(Role::GridCell, "2026-10-12").click();
    assert_eq!(
        (start.get_untracked(), end.get_untracked()),
        (date(2026, 10, 12), date(2026, 10, 15))
    );
    assert_eq!(opener.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_date_picker_writes_the_binding_and_closes() {
    let (screen, date_value) = kivact::test::render_with_handle(
        crate::support::options()
            .messages(kivact::i18n::Messages::new().with("date.open", "Choose a date")),
        || {
            let value = signal(date(2026, 10, 3));
            (value, {
                let value: kivact::state::Binding<kivact::i18n::CalendarDate> =
                    kivact::state::IntoBinding::into_binding(value);
                let value = kivact::state::binding(
                    move || Some(value.get()),
                    move |day| {
                        if let Some(day) = day {
                            value.set(day);
                        }
                    },
                );
                kivact::ui::date_input("Date", value).calendar(move |calendar| {
                    calendar
                        .today(date(2026, 10, 3))
                        .month_title(|date| format!("{}-{:02}", date.year(), date.month()))
                })
            })
        },
    );
    let opener = screen.get_by_role(Role::Button, "Choose a date");
    assert_eq!(opener.is_expanded(), Some(false));
    opener.click();
    assert_eq!(opener.is_expanded(), Some(true));
    screen.get_by_role(Role::GridCell, "2026-10-17").click();
    assert_eq!(date_value.get_untracked(), date(2026, 10, 17));
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Date").value(),
        Some("2026-10-17".to_string())
    );
    assert_eq!(opener.is_expanded(), Some(false));
    opener.click();
    screen.get_by_role(Role::GridCell, "2026-10-17").click();
    assert_eq!(opener.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
