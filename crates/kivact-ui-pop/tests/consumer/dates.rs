//! Role: The calendar, the date input and the date range input.

use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_pop as ui;

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(year, month, day)
}

#[test]
fn the_date_picker_writes_the_binding_and_closes() {
    let (screen, date_value) = kivact::test::render_with_handle(crate::options(), || {
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
