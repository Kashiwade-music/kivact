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
    RenderOptions::from_env().theme(theme(ThemeMode::Light))
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

#[test]
fn date_sheet_swipe_pages_without_selecting_the_contact_day() {
    use kivact::view::{PointerKind, ReadingDirection, provide_direction};
    let start = CalendarDate::new(2026, 10, 14);
    let bound = CalendarDate::new(2026, 11, 30);
    for direction in [ReadingDirection::Ltr, ReadingDirection::Rtl] {
        let (screen, selected) = render_with_handle(options(), move || {
            let selected = signal(Some(start));
            (
                selected,
                provide_direction(direction, move || {
                    date_input("Due", selected)
                        .presentation(DatePresentation::BottomSheet)
                        .calendar(move |calendar| calendar.bounds(Some(start), Some(bound)))
                }),
            )
        });
        screen
            .get_by_role(Role::ComboBox, "Due")
            .press_key(Key::ArrowDown);
        let day = screen.get_by_role(Role::GridCell, "2026-10-20");
        let bounds = day.accessibility_bounds().unwrap();
        assert!((bounds.width - 44.0).abs() < 1.0);
        assert!((bounds.height - 44.0).abs() < 1.0);
        let x = bounds.x + bounds.width / 2.0;
        let y = bounds.y + bounds.height / 2.0;
        let dx = if direction == ReadingDirection::Ltr {
            -48.0
        } else {
            48.0
        };
        let finger = screen.contact_down(PointerKind::Touch, x, y);
        screen.contact_move(finger, x + dx, y + 24.0);
        screen.contact_up(finger);
        assert_eq!(selected.get_untracked(), Some(start));
        let next = screen.get_by_role(Role::GridCell, "2026-11-15");
        screen.advance(std::time::Duration::from_millis(500));
        next.tap();
        assert_eq!(
            selected.get_untracked(),
            Some(CalendarDate::new(2026, 11, 15))
        );
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn time_wheel_flick_projects_to_a_detent_and_exposes_increment_actions() {
    use kivact::view::{A11yAction, PointerKind};
    let (screen, selected) = render_with_handle(options(), || {
        let selected = signal(TimeOfDay::new(12, 20, 0));
        (
            selected,
            time_input("Reminder", selected).presentation(DatePresentation::BottomSheet),
        )
    });
    screen
        .get_by_role(Role::ComboBox, "Reminder")
        .press_key(Key::ArrowDown);
    screen.advance(Duration::from_millis(600));
    let minute = screen.get_by_role(Role::SpinButton, "Minute");
    assert_eq!(minute.value_range(), (Some(0.0), Some(59.0), Some(20.0)));
    let bounds = minute.accessibility_bounds().unwrap();
    let x = bounds.x + bounds.width / 2.0;
    let y = bounds.y + bounds.height / 2.0;
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.advance(Duration::from_millis(16));
    screen.contact_move(contact, x, y - 35.0);
    assert_eq!(selected.get_untracked(), TimeOfDay::new(12, 21, 0));
    screen.advance(Duration::from_millis(16));
    screen.contact_up(contact);
    let snapped = selected.get_untracked().unwrap().minute();
    assert!(
        snapped > 21,
        "release should advance beyond the crossed detent"
    );
    screen.advance(Duration::from_secs(1));
    assert_eq!(selected.get_untracked().unwrap().minute(), snapped);
    minute.a11y(A11yAction::Increment);
    assert_eq!(selected.get_untracked().unwrap().minute(), snapped + 1);
    minute.press_key(Key::PageUp);
    assert_eq!(selected.get_untracked().unwrap().minute(), snapped - 9);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
