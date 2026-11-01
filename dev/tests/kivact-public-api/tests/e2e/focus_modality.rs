//! Role: Tests for the focus indicator of programmatic focus requests.
//!
//! `NodeRef::focus` and `NodeRef::restore_focus` show the focus indicator
//! unless the last key or pointer press was a pointer press, so a row a
//! click selected shows no keyboard ring while arrow keys bring it back.

use crate::support::mount;
use kivact::prelude::*;
use kivact::view::{Handled, PointerPhase};

fn button(label: &'static str) -> Node {
    node()
        .role(Role::Button)
        .focusable(true)
        .name(label)
        .child(text(label))
}

#[test]
fn a_focus_request_follows_the_modality_of_the_last_input() {
    let (screen, (target, visible)) = mount(|| {
        let target = node_ref();
        let visible = target.focus_visible();
        let view = column((
            button("Target").node_ref(target),
            node()
                .test_id("by-pointer")
                .on_pointer(move |event| {
                    if event.phase == PointerPhase::Down {
                        target.focus();
                    }
                    Handled::Yes
                })
                .child(text("by pointer")),
            node()
                .test_id("restore-by-pointer")
                .on_pointer(move |event| {
                    if event.phase == PointerPhase::Down {
                        target.restore_focus();
                    }
                    Handled::Yes
                })
                .child(text("restore by pointer")),
            button("Driver").on_key(move |event| {
                if event.key == Key::ArrowDown {
                    target.focus();
                    return Handled::Yes;
                }
                if event.key == Key::ArrowUp {
                    target.restore_focus();
                    return Handled::Yes;
                }
                Handled::No
            }),
        ));
        ((target, visible), view)
    });
    let _ = target;
    let driver = screen.get_by_role(Role::Button, "Driver");

    // Before any input a request shows the indicator.
    driver.focus();
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Button, "Target").is_focused());
    assert!(visible.get_untracked(), "no input yet: visible");

    // A pointer press makes the next request invisible.
    screen.get_by_test_id("by-pointer").click();
    assert!(screen.get_by_role(Role::Button, "Target").is_focused());
    assert!(
        !visible.get_untracked(),
        "NodeRef::focus after a pointer press shows no indicator"
    );

    // A key press brings it back, even though the focus came by a click.
    driver.focus();
    screen.press_key(Key::ArrowDown);
    assert!(
        visible.get_untracked(),
        "NodeRef::focus after a key press shows the indicator"
    );

    // The same rules hold for restore_focus.
    driver.focus();
    screen.get_by_test_id("restore-by-pointer").click();
    assert!(
        screen.get_by_role(Role::Button, "Target").is_focused(),
        "restore moved the focus"
    );
    assert!(
        !visible.get_untracked(),
        "NodeRef::restore_focus after a pointer press shows no indicator"
    );
    driver.focus();
    screen.press_key(Key::ArrowUp);
    assert!(
        visible.get_untracked(),
        "NodeRef::restore_focus after a key press shows the indicator"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
