//! Role: Verify composer editing keys and tool disclosure semantics.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::view::Key;
use kivact::view::Modifiers;
use kivact::view::Role;
use kivact_ui_soft as soft;

#[test]
fn composer_inserts_shift_enter_and_recalls_the_last_submission() {
    let submitted = Rc::new(RefCell::new(Vec::new()));
    let received = submitted.clone();
    let screen = render_with(
        RenderOptions::with_theme(soft::theme(ThemeMode::Light)),
        move || {
            kivact::ui::chat_composer(signal(String::new())).on_submit(move |message| {
                received.borrow_mut().push(message);
            })
        },
    );
    let input = screen.get_by_role(Role::TextBox, "Message input");
    input.type_text("First line");
    input.press_key_with(
        Key::Enter,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    input.type_text("Second line");
    assert!(submitted.borrow().is_empty());
    input.press_key(Key::Enter);
    assert_eq!(submitted.borrow().as_slice(), ["First line\nSecond line"]);
    assert!(input.value().unwrap_or_default().is_empty());

    input.press_key(Key::ArrowUp);
    assert_eq!(input.value().as_deref(), Some("First line\nSecond line"));
    assert_eq!(input.text_selection(), Some((0, 22)));
    input.press_key(Key::ArrowDown);
    assert!(input.value().unwrap_or_default().is_empty());

    input.type_text("Another message");
    input.press_key(Key::Enter);
    input.type_text("Draft");
    input.press_key(Key::ArrowUp);
    assert_eq!(input.value().as_deref(), Some("Draft"));
    input.press_key(Key::Home);
    input.press_key(Key::ArrowUp);
    assert_eq!(input.value().as_deref(), Some("Another message"));
    input.press_key(Key::ArrowUp);
    assert_eq!(input.value().as_deref(), Some("First line\nSecond line"));
    input.press_key(Key::ArrowDown);
    assert_eq!(input.value().as_deref(), Some("Another message"));
    input.press_key(Key::ArrowDown);
    assert_eq!(input.value().as_deref(), Some("Draft"));
    assert_eq!(submitted.borrow().len(), 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tool_calls_publish_keyboard_disclosures_and_hide_collapsed_details() {
    let screen = render_with(
        RenderOptions::with_theme(soft::theme(ThemeMode::Light)),
        || {
            soft::chat_tool_calls(vec![
                soft::ToolCall::new("read", "Read file").status(soft::ToolStatus::Error {
                    message: Some("Access denied".into()),
                }),
                soft::ToolCall::new("check", "Check path"),
            ])
            .detail(|_| kivact::ui::button("Inspect output"))
        },
    );
    let group = screen.get_by_role(Role::Button, "2 tool calls");
    screen.get_by_text("Check path");
    assert!(screen.query_by_text("Read file").is_none());
    assert_eq!(group.is_expanded(), Some(false));
    assert!(
        screen
            .query_by_role(Role::Button, "Details for Read file")
            .is_none()
    );
    group.press_key(Key::Enter);
    assert_eq!(group.is_expanded(), Some(true));
    assert_eq!(group.controls().len(), 1);
    screen.advance(Duration::from_millis(320));
    screen.get_by_text("! Failed");
    screen.get_by_text("Error: Access denied");

    let details = screen.get_by_role(Role::Button, "Details for Read file");
    assert_eq!(details.is_expanded(), Some(false));
    assert!(
        screen
            .query_by_role(Role::Button, "Inspect output")
            .is_none()
    );
    details.press_key(Key::Space);
    assert_eq!(details.is_expanded(), Some(true));
    assert_eq!(details.controls().len(), 1);
    let inspect = screen.get_by_role(Role::Button, "Inspect output");
    inspect.press_key(Key::Escape);
    assert!(details.is_focused());
    assert_eq!(details.is_expanded(), Some(false));
    assert!(
        screen
            .query_by_role(Role::Button, "Inspect output")
            .is_none()
    );
    group.press_key(Key::Space);
    assert_eq!(group.is_expanded(), Some(false));
    screen.advance(Duration::from_millis(320));
    screen.get_by_text("Check path");
    assert!(
        screen
            .query_by_role(Role::Button, "Details for Read file")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn composer_does_not_submit_during_an_ime_composition() {
    let submitted = Rc::new(RefCell::new(Vec::new()));
    let received = submitted.clone();
    let screen = render_with(
        RenderOptions::with_theme(soft::theme(ThemeMode::Light)),
        move || {
            kivact::ui::chat_composer(signal(String::new())).on_submit(move |message| {
                received.borrow_mut().push(message);
            })
        },
    );
    let input = screen.get_by_role(Role::TextBox, "Message input");
    input.compose("仮");
    input.press_key(Key::Enter);
    assert!(submitted.borrow().is_empty());
    input.commit("仮");
    input.press_key(Key::Enter);
    assert_eq!(submitted.borrow().as_slice(), ["仮"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn one_tool_call_has_no_group_toggle_and_can_open_its_details() {
    let screen = render_with(
        RenderOptions::with_theme(soft::theme(ThemeMode::Light)),
        || {
            soft::chat_tool_calls(vec![
                soft::ToolCall::new("read", "Read file").status(soft::ToolStatus::Running),
            ])
            .detail(|_| kivact::ui::button("Inspect output"))
        },
    );
    assert!(screen.query_by_role(Role::Button, "1 tool calls").is_none());
    screen.get_by_role(Role::Status, "Running");
    let details = screen.get_by_role(Role::Button, "Details for Read file");
    details.press_key(Key::Enter);
    assert_eq!(details.is_expanded(), Some(true));
    screen.get_by_role(Role::Button, "Inspect output");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
