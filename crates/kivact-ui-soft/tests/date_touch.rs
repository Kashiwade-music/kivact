//! Role: Verify forced date and time sheets commit immediately and Save closes.
use kivact::i18n::CalendarDate;
use kivact::signal;
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, render_with_handle};
use kivact::view::{Key, Role};
use kivact_ui_soft::{
    DatePresentation, DateTime, TimeOfDay, date_input, date_time_input, theme, time_input,
};
use std::time::Duration;

fn options() -> RenderOptions {
    RenderOptions {
        theme: theme(ThemeMode::Light),
        ..RenderOptions::from_env()
    }
}

#[test]
fn date_sheet_uses_calendar_and_month_wheels_without_staging_the_date() {
    let start = CalendarDate::new(2026, 10, 14);
    let (screen, selected) = render_with_handle(options(), move || {
        let selected = signal(Some(start));
        (
            selected,
            date_input("Due", selected).presentation(DatePresentation::BottomSheet),
        )
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Due");
    trigger.press_key(Key::ArrowDown);
    screen.advance(Duration::from_millis(600));
    screen.get_by_role(Role::Dialog, "Choose date");
    screen.get_by_role(Role::Button, "October 2026").click();
    screen
        .get_by_role(Role::SpinButton, "Month")
        .press_key(Key::ArrowDown);
    assert_eq!(selected.get_untracked(), Some(start));
    screen.get_by_role(Role::Button, "Done").click();
    screen.get_by_role(Role::GridCell, "2026-11-15").click();
    assert_eq!(
        selected.get_untracked(),
        Some(CalendarDate::new(2026, 11, 15))
    );
    screen.get_by_role(Role::Dialog, "Choose date");
    screen.get_by_role(Role::Button, "Save").click();
    screen.advance(Duration::from_millis(600));
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn time_sheet_wheels_commit_each_step_before_save() {
    let (screen, selected) = render_with_handle(options(), || {
        let selected = signal(TimeOfDay::new(12, 59, 58));
        (
            selected,
            time_input("Reminder", selected)
                .show_seconds(true)
                .presentation(DatePresentation::BottomSheet),
        )
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Reminder");
    trigger.press_key(Key::ArrowDown);
    screen.advance(Duration::from_millis(600));
    screen
        .get_by_role(Role::SpinButton, "Minute")
        .press_key(Key::ArrowDown);
    assert_eq!(selected.get_untracked(), TimeOfDay::new(12, 0, 58));
    screen
        .get_by_role(Role::SpinButton, "Second")
        .press_key(Key::ArrowDown);
    assert_eq!(selected.get_untracked(), TimeOfDay::new(12, 0, 59));
    screen.get_by_role(Role::Button, "Save").click();
    screen.advance(Duration::from_millis(600));
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn combined_sheet_advances_from_date_to_time_and_keeps_the_committed_date() {
    let start = CalendarDate::new(2026, 10, 14);
    let (screen, selected) = render_with_handle(options(), move || {
        let selected = signal(Some(DateTime {
            date: start,
            time: TimeOfDay::new(12, 15, 0).unwrap(),
        }));
        (
            selected,
            date_time_input("Starts at", selected).presentation(DatePresentation::BottomSheet),
        )
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Date");
    trigger.press_key(Key::ArrowDown);
    screen.advance(Duration::from_millis(600));
    screen.get_by_role(Role::GridCell, "2026-10-15").click();
    assert_eq!(
        selected.get_untracked().unwrap().date,
        CalendarDate::new(2026, 10, 15)
    );
    screen.get_by_role(Role::Button, "Save date").click();
    screen.advance(Duration::from_millis(400));
    screen
        .get_by_role(Role::SpinButton, "Hour")
        .press_key(Key::ArrowDown);
    assert_eq!(
        selected.get_untracked().unwrap().time,
        TimeOfDay::new(13, 15, 0).unwrap()
    );
    screen.get_by_role(Role::Button, "Save").click();
    screen.advance(Duration::from_millis(600));
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
