//! The Pop catalog's page walker and the public-driver checks of its stateful demos.
//! The walker opens every page from the list in both modes; the three state tests drive
//! the forms, the overlays and the conversation through the same public test driver.

use catalog_shell::check;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::Modifiers;
use kivact_ui_pop_catalog::{THEME, pages};
use std::time::Duration;

#[test]
fn every_page_opens_from_the_list_and_renders_cleanly() {
    check::every_page_opens_cleanly("Pop", pages, THEME, &[]);
}

#[test]
fn every_page_character_is_in_the_catalog_fonts() {
    check::every_page_character_is_in_the_fonts(pages, THEME);
}

fn mount() -> Screen {
    check::mount("Pop", pages, THEME, ThemeMode::Light)
}

fn replace(screen: &Screen, label: &str, value: &str) {
    let input = screen.get_by_role(Role::TextBox, label);
    input.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    input.type_text(value);
    assert_eq!(input.value(), Some(value.to_string()));
}

#[test]
fn pop_forms_edit_drag_choose_date_and_keep_state_in_dark_theme() {
    let screen = mount();
    check::open(&screen, "forms");
    screen.evidence("pop-state-forms-light");
    let email = screen.get_by_role(Role::TextBox, "Email");
    assert!(email.is_invalid());
    assert_eq!(
        email
            .described_by()
            .iter()
            .map(|e| e.text())
            .collect::<Vec<_>>(),
        ["Enter a valid email address."]
    );
    replace(&screen, "Student name *", "Minato Sorasaki, Investigation");
    edit_and_resize_memo(&screen);
    let tags = screen.get_by_role(Role::TextBox, "Add a tag");
    tags.type_text("Liaison");
    tags.press_key(Key::Enter);
    screen.get_by_role(Role::Button, "Remove Liaison");
    screen
        .get_by_role(Role::CheckBox, "Receive notifications")
        .click();
    assert_eq!(
        screen
            .get_by_role(Role::CheckBox, "Receive notifications")
            .checked(),
        Some(CheckedState::Unchecked)
    );
    assert_eq!(
        screen
            .get_by_role(Role::CheckBox, "Partly selected (indeterminate)")
            .checked(),
        Some(CheckedState::Indeterminate)
    );
    let volume = screen.get_by_role(Role::Slider, "BGM volume");
    let track = screen.get_by_test_id("pop-volume");
    let width = track.rect().expect("slider laid out").width;
    track.drag(
        Point::new(width * 0.64, 10.0),
        Point::new(width * 0.82, 10.0),
    );
    let (_, _, now) = volume.value_range();
    assert!(now.is_some_and(|n| n > 75.0 && n < 90.0));
    screen.get_by_test_id("pop-password-reveal").click();
    screen.evidence("pop-state-password-revealed");
    screen.get_by_test_id("pop-password-reveal").click();
    screen.get_by_role(Role::Button, "Choose date").click();
    screen.evidence("pop-state-date-overlay");
    screen
        .get_by_test_id("pop-date-input")
        .get_by_role(Role::GridCell, "2026-10-17")
        .click();
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Date").value(),
        Some("2026-10-17".into())
    );
    screen.get_by_test_id("catalog-theme").click();
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Student name *").value(),
        Some("Minato Sorasaki, Investigation".into())
    );
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Date").value(),
        Some("2026-10-17".into())
    );
    screen.evidence("pop-state-forms-dark");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[cfg(test)]
fn edit_and_resize_memo(screen: &Screen) {
    let memo = screen.get_by_role(Role::TextBox, "Request note");
    memo.commit("Check the safety of the area first.");
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Request note").value(),
        Some("Check the safety of the area first.".into())
    );
    let initial_memo = memo.rect().expect("memo laid out");
    assert_eq!(initial_memo.height, 80.0);
    let resize = |delta: f64| {
        let grip = screen
            .get_by_test_id("pop-memo-resize")
            .get_by_role(Role::Splitter, "Resize text area");
        grip.hover();
        let rect = grip.rect().expect("resize grip laid out");
        let x = rect.x + rect.width / 2.0;
        let y = rect.y + rect.height / 2.0;
        screen.mouse_move(x, y);
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_move(x, y + delta / 2.0);
        screen.mouse_move(x, y + delta);
        screen.mouse_up(MouseButton::Primary);
    };
    resize(48.0);
    assert_eq!(memo.rect().unwrap().height, 128.0);
    assert_eq!(memo.rect().unwrap().width, initial_memo.width);
    assert!(memo.is_focused());
    assert_eq!(
        memo.value(),
        Some("Check the safety of the area first.".into())
    );
    screen.type_text(" Keep going.");
    assert_eq!(
        memo.value(),
        Some("Check the safety of the area first. Keep going.".into())
    );
    screen.evidence("pop-state-textarea-expanded");
    resize(-200.0);
    assert_eq!(memo.rect().unwrap().height, 72.0);
    assert_eq!(memo.rect().unwrap().width, initial_memo.width);
    assert!(memo.is_focused());
    assert_eq!(
        memo.value(),
        Some("Check the safety of the area first. Keep going.".into())
    );
    screen.evidence("pop-state-textarea-minimum");
}

#[test]
fn pop_modals_drawer_calendar_and_conversation_use_live_models() {
    let screen = mount();
    check::open(&screen, "overlays");
    let opener = screen.get_by_role(Role::Button, "Confirm dialog");
    opener.click();
    screen.get_by_role(Role::Dialog, "Start the mission?");
    screen
        .get_by_role(Role::CheckBox, "Do not show this confirmation again")
        .click();
    assert_eq!(
        screen
            .get_by_role(Role::CheckBox, "Do not show this confirmation again")
            .checked(),
        Some(CheckedState::Checked)
    );
    screen.evidence("pop-state-confirm-modal");
    screen.press_key(Key::Escape);
    assert!(opener.is_focused());
    assert!(
        screen
            .query_by_role(Role::Dialog, "Start the mission?")
            .is_none()
    );
    screen.get_by_role(Role::Button, "Form dialog").click();
    assert!(screen.get_by_test_id("pop-create-request").is_disabled());
    screen
        .get_by_test_id("pop-request-title")
        .type_text("Central ward patrol");
    assert!(!screen.get_by_test_id("pop-create-request").is_disabled());
    screen.evidence("pop-state-form-modal");
    screen.get_by_test_id("pop-create-request").click();
    assert!(
        screen
            .query_by_role(Role::Dialog, "Create a new request")
            .is_none()
    );
    screen.get_by_role(Role::Button, "Open drawer").click();
    screen.get_by_role(Role::Dialog, "Minato Sorasaki");
    screen.evidence("pop-state-profile-drawer");
    screen.get_by_test_id("pop-drawer-save").click();
    assert!(
        screen
            .query_by_role(Role::Dialog, "Minato Sorasaki")
            .is_none()
    );
    check::open(&screen, "schedule");
    screen.get_by_role(Role::GridCell, "2026-10-10").click();
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-10")
            .is_selected()
    );
    screen.get_by_text("Agenda for 10/10 (Sat)");
    screen.evidence("pop-state-calendar-selection");
    check::open(&screen, "messaging");
    let message = screen.get_by_role(Role::TextBox, "Type a message");
    message.commit("Got it. Safety first, please.");
    message.press_key(Key::Enter);
    assert_eq!(message.value(), Some(String::new()));
    screen.get_by_text("Got it. Safety first, please.");
    assert!(screen.query_by_test_id("pop-reply-card").is_none());
    assert!(screen.query_by_role(Role::Status, "Typing").is_none());
    screen.evidence("pop-state-message-sent");
    screen.advance(Duration::from_millis(410));
    screen.get_by_role(Role::Status, "Typing");
    screen.evidence("pop-state-message-typing");
    screen.advance(Duration::from_millis(1310));
    assert!(screen.query_by_role(Role::Status, "Typing").is_none());
    screen.get_by_text("Got it! I will check right away.");
    screen.evidence("pop-state-message-replied");
    let thread = screen.get_by_test_id("pop-thread-1");
    thread.get_by_text("1");
    thread.click();
    assert!(thread.is_selected());
    assert!(screen.within(thread).query_by_text("1").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pop_two_quick_messages_both_get_a_reply() {
    let screen = mount();
    check::open(&screen, "messaging");
    let message = screen.get_by_role(Role::TextBox, "Type a message");
    message.commit("First message");
    message.press_key(Key::Enter);
    message.commit("Second message");
    message.press_key(Key::Enter);
    screen.advance(Duration::from_millis(4000));
    screen.get_by_text("Got it! I will check right away.");
    screen.get_by_text("Thank you, teacher ♪");
    assert!(screen.query_by_test_id("pop-reply-card").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
