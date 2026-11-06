//! Role: Check APG date-picker focus, labels and announcements.
use crate::support::mount;
use kivact::i18n::{CalendarDate, CalendarMonth};
use kivact::prelude::*;
use kivact::ui::{self, DatePresentation};
#[test]
fn date_picker_selection_returns_focus_to_the_picker_button() {
    let (screen, value) = mount(|| {
        let value = signal(Some(CalendarDate::new(2026, 10, 14)));
        (
            value,
            ui::date_input("Date", value).presentation(DatePresentation::Desktop),
        )
    });
    let button = screen.get_by_role(Role::Button, "Choose date");
    button.click();
    let selected = screen.get_by_role(Role::GridCell, "2026-10-14");
    assert!(selected.is_focused());
    selected.press_key(Key::ArrowRight);
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), Some(CalendarDate::new(2026, 10, 15)));
    assert!(button.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn date_picker_weekday_headers_publish_full_names_and_month_changes_are_announced() {
    let (screen, month) = mount(|| {
        let month = signal(CalendarMonth::new(2026, 10));
        (
            month,
            ui::calendar(signal(None::<CalendarDate>)).month(month),
        )
    });
    screen.get_by_role(Role::ColumnHeader, "Monday");
    screen.get_by_role(Role::Button, "Next month").click();
    assert_eq!(month.get_untracked(), CalendarMonth::new(2026, 11));
    screen.get_by_role(Role::Status, "Showing November 2026");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn date_picker_keyboard_invoker_is_restored_and_outside_click_keeps_focus() {
    let (screen, value) = mount(|| {
        let value = signal(Some(CalendarDate::new(2026, 10, 14)));
        (
            value,
            column((
                ui::date_input("Date", value).presentation(DatePresentation::Desktop),
                spacer().style(style().height(400.0)),
                ui::button("Outside"),
            )),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Date");
    input.press_key_with(
        Key::ArrowDown,
        Modifiers {
            alt: true,
            ..Modifiers::default()
        },
    );
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-14")
            .is_focused()
    );
    screen.press_key(Key::Escape);
    assert!(input.is_focused());
    input.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), Some(CalendarDate::new(2026, 10, 15)));
    assert!(input.is_focused());
    screen.get_by_role(Role::Button, "Choose date").click();
    screen.get_by_role(Role::Button, "Outside").click();
    assert!(screen.get_by_role(Role::Button, "Outside").is_focused());
    assert!(screen.query_by_role(Role::Grid, "2026-10").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn time_picker_spinbutton_uses_scalar_key_and_assistive_directions() {
    let (screen, value) = mount(|| {
        let value = signal(Some(kivact::i18n::TimeOfDay {
            hour: 12,
            minute: 20,
            second: 0,
        }));
        (
            value,
            ui::time_input("Time", value).presentation(DatePresentation::BottomSheet),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Time");
    input.press_key_with(
        Key::ArrowDown,
        Modifiers {
            alt: true,
            ..Modifiers::default()
        },
    );
    screen.advance(std::time::Duration::from_millis(600));
    let minute = screen.get_by_role(Role::SpinButton, "Minute");
    minute.press_key(Key::ArrowUp);
    assert_eq!(value.get_untracked().unwrap().minute, 21);
    minute.press_key(Key::ArrowDown);
    assert_eq!(value.get_untracked().unwrap().minute, 20);
    minute.a11y(kivact::view::A11yAction::Increment);
    assert_eq!(value.get_untracked().unwrap().minute, 21);
    minute.a11y(kivact::view::A11yAction::Decrement);
    assert_eq!(value.get_untracked().unwrap().minute, 20);
    screen.get_by_role(Role::Button, "Save").click();
    screen.advance(std::time::Duration::from_millis(600));
    assert!(input.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
