//! Role: Core calendar selection, typed date/time editing and touch wheel scenarios.
use crate::support::mount;
use kivact::i18n::{CalendarDate, CalendarMonth, DateRange, TimeOfDay};
use kivact::prelude::*;
use kivact::ui::{self, DatePresentation};
fn day(d: u8) -> CalendarDate {
    CalendarDate::new(2026, 10, d)
}
#[test]
fn calendar_selects_a_date_with_the_keyboard() {
    let (screen, value) = mount(|| {
        let value = signal(Some(day(14)));
        (
            value,
            ui::calendar(value)
                .month(signal(CalendarMonth::new(2026, 10)))
                .autofocus(true),
        )
    });
    screen
        .get_by_role(Role::GridCell, "2026-10-14")
        .press_key(Key::ArrowRight);
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), Some(day(15)));
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-15")
            .is_selected()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn calendar_range_picks_start_then_end() {
    let (screen, value) = mount(|| {
        let value = signal(None::<DateRange>);
        (
            value,
            ui::calendar(value).default_month(CalendarMonth::new(2026, 10)),
        )
    });
    screen.get_by_role(Role::GridCell, "2026-10-20").click();
    screen.get_by_role(Role::GridCell, "2026-10-14").click();
    assert_eq!(
        value.get_untracked(),
        Some(DateRange::new(day(14), day(20)))
    );
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-17")
            .is_selected()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn date_input_parses_typed_text_and_opens_the_calendar() {
    let (screen, value) = mount(|| {
        let value = signal(None::<CalendarDate>);
        (
            value,
            ui::date_input("Birthday", value).presentation(DatePresentation::Desktop),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Birthday");
    field.click();
    field.type_text("2026-10-14");
    field.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), Some(day(14)));
    screen.get_by_role(Role::Button, "Choose date").click();
    screen.get_by_role(Role::Grid, "2026-10");
    screen.get_by_role(Role::GridCell, "2026-10-20").click();
    assert_eq!(value.get_untracked(), Some(day(20)));
    assert!(screen.query_by_role(Role::Grid, "2026-10").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn date_input_bottom_sheet_picks_with_the_wheel() {
    let (screen, value) = mount(|| {
        let value = signal(Some(day(14)));
        (
            value,
            ui::date_input("Birthday", value).presentation(DatePresentation::BottomSheet),
        )
    });
    screen.get_by_role(Role::Button, "Choose date").click();
    let wheel = screen.get_by_role(Role::SpinButton, "Day");
    wheel.click();
    wheel.press_key(Key::ArrowUp);
    screen.get_by_role(Role::Button, "Apply date").click();
    assert_eq!(value.get_untracked(), Some(day(15)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn time_input_accepts_hours_and_minutes() {
    let (screen, value) = mount(|| {
        let value = signal(None::<TimeOfDay>);
        (
            value,
            ui::time_input("Meeting", value).presentation(DatePresentation::Desktop),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Meeting");
    field.click();
    field.type_text("13:45");
    field.press_key(Key::Enter);
    assert_eq!(
        value.get_untracked(),
        Some(TimeOfDay {
            hour: 13,
            minute: 45,
            second: 0
        })
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
