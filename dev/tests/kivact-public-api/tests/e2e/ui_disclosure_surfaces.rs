//! Role: Verify the public accordion, hover preview, drawer, and clipboard contracts.
use crate::support::mount;
use kivact::geometry::Edge;
use kivact::interaction::{DialogMode, use_copy_feedback};
use kivact::prelude::*;
use kivact::view::MouseButton;
use std::time::Duration;

#[test]
fn disclosure_group_single_keeps_one_item_open() {
    let (screen, selected) = mount(|| {
        let selected = signal(Some("a"));
        (
            selected,
            kivact::ui::disclosure_group(selected)
                .panel("a", "Alpha", || text("Alpha content"))
                .panel("b", "Beta", || text("Beta content")),
        )
    });
    screen.get_by_role(Role::Button, "Beta").click();
    assert_eq!(selected.get_untracked(), Some("b"));
    assert_eq!(
        screen.get_by_role(Role::Button, "Alpha").is_expanded(),
        Some(false)
    );
    assert_eq!(
        screen.get_by_role(Role::Button, "Beta").is_expanded(),
        Some(true)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn hover_card_opens_after_the_delay_and_on_focus() {
    let (screen, ()) = mount(|| {
        (
            (),
            kivact::ui::hover_card(|| text("Preview body"))
                .label("Preview")
                .open_delay(Duration::from_millis(200))
                .close_delay(Duration::from_millis(50))
                .attach(button("Preview trigger")),
        )
    });
    screen.get_by_role(Role::Button, "Preview trigger").hover();
    screen.advance(Duration::from_millis(100));
    assert!(screen.query_by_role(Role::Dialog, "Preview").is_none());
    screen.advance(Duration::from_millis(150));
    assert!(screen.query_by_role(Role::Dialog, "Preview").is_some());
    screen.mouse_leave();
    screen.advance(Duration::from_millis(100));
    screen.get_by_role(Role::Button, "Preview trigger").focus();
    assert!(screen.query_by_role(Role::Dialog, "Preview").is_some());
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Dialog, "Preview").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn drawer_closes_on_swipe() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        (
            open,
            dialog("Drawer", open)
                .mode(DialogMode::Drawer(Edge::Bottom))
                .content(|| text("Drawer body")),
        )
    });
    let handle = screen
        .get_by_role(Role::Group, "Drag to close")
        .rect()
        .expect("laid out")
        .center();
    screen.mouse_move(handle.x, handle.y);
    screen.mouse_down(MouseButton::Primary);
    screen.advance(Duration::from_millis(50));
    screen.mouse_move(handle.x, handle.y + 250.0);
    screen.mouse_up(MouseButton::Primary);
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn copy_feedback_resets_after_its_duration() {
    let (screen, copied) = mount(|| {
        let copy = use_copy_feedback(Duration::from_millis(200));
        (
            copy.is_copied(),
            button("Copy").on_press(move || copy.copy("Hello")),
        )
    });
    screen.get_by_role(Role::Button, "Copy").click();
    assert!(copied.get_untracked());
    screen.advance(Duration::from_millis(150));
    screen.get_by_role(Role::Button, "Copy").click();
    screen.advance(Duration::from_millis(100));
    assert!(
        copied.get_untracked(),
        "a repeated copy restarts the confirmation duration"
    );
    screen.advance(Duration::from_millis(150));
    assert!(!copied.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn hover_card_controlled_binding_rejection_and_external_close_are_authoritative() {
    let (screen, open) = mount(|| {
        let open = signal(false);
        (
            open,
            column((
                kivact::ui::hover_card(|| text("Readonly body"))
                    .label("Readonly")
                    .open(binding(|| false, |_| ()))
                    .open_delay(Duration::ZERO)
                    .attach(button("Readonly trigger")),
                kivact::ui::hover_card(|| text("Controlled body"))
                    .label("Controlled")
                    .open(open)
                    .open_delay(Duration::ZERO)
                    .attach(button("Controlled trigger")),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Readonly trigger").hover();
    assert!(screen.query_by_role(Role::Dialog, "Readonly").is_none());
    screen
        .get_by_role(Role::Button, "Controlled trigger")
        .hover();
    assert!(open.get_untracked());
    open.set(false);
    screen.advance(Duration::ZERO);
    assert!(screen.query_by_role(Role::Dialog, "Controlled").is_none());
    screen.advance(Duration::from_millis(1000));
    assert!(
        !open.get_untracked(),
        "the same pointer engagement cannot reopen an external close"
    );
    screen.mouse_leave();
    screen
        .get_by_role(Role::Button, "Controlled trigger")
        .hover();
    assert!(open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
