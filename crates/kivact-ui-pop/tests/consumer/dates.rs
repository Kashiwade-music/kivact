//! Role: The calendar, the date input and the date range input.

use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_pop as ui;

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(year, month, day)
}

#[test]
fn the_date_range_writes_only_a_complete_range() {
    let (screen, (start, end)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let today = date(2026, 10, 3);
            let (start, end) = (signal(today), signal(date(2026, 10, 5)));
            (
                (start, end),
                ui::date_range_input(("From", "To"), (start, end), today),
            )
        });
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
    let (screen, date_value) = kivact::test::render_with_handle(crate::support::options(), || {
        let value = signal(date(2026, 10, 3));
        (value, ui::date_input("Date", value, date(2026, 10, 3)))
    });
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
