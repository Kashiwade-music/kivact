//! Role: Tests for window-level key handling and event propagation.
//!
//! Window-level keys: while nothing has the focus and no modal is open, a
//! key starts at the App's view, so its `on_key` handlers act as window
//! shortcuts, and an Enter that got there that way does not activate it; a
//! focused control still sees the key first and the key bubbles to the
//! App's view from there; while a modal is open the key stops at the modal.

use crate::support::mount;
use kivact::draw::Point;
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::view::overlay_at;
use kivact::view::{Handled, KeyEvent, Placement};

#[test]
fn unfocused_keys_reach_the_app_view_without_activating_it() {
    let (screen, (log, open)) = mount(|| {
        let log = signal(Vec::<String>::new());
        let open = signal(false);
        let note = move |entry: String| log.update(|l| l.push(entry));
        let view = node()
            .role(Role::Group)
            .name("app")
            .on_key(move |e: KeyEvent| {
                note(format!("app {:?} {:?}", e.key, e.target_role));
                Handled::No
            })
            .on_press(move || note("app activated".into()))
            .child(
                node()
                    .role(Role::Button)
                    .name("control")
                    .test_id("control")
                    .focusable(true)
                    .style(style().size(40, 24))
                    .on_key(move |e: KeyEvent| {
                        note(format!("control {:?}", e.key));
                        if e.key == Key::Char('x') {
                            Handled::Yes
                        } else {
                            Handled::No
                        }
                    }),
            )
            .child(show(open, move || {
                // No tab stop inside: nothing is focused while it is open.
                overlay_at(Point::new(200.0, 200.0), Placement::Center, text("notice"))
                    .modal(true)
                    .role(Role::Dialog)
                    .on_key(move |e: KeyEvent| {
                        note(format!("modal {:?}", e.key));
                        Handled::No
                    })
            }));
        ((log, open), view)
    });
    let take = || {
        let seen = log.get_untracked();
        log.set(Vec::new());
        seen
    };

    assert!(screen.focused().is_none(), "nothing has the focus yet");
    screen.press_key(Key::Char('x'));
    assert_eq!(
        take(),
        ["app Char('x') Some(Group)"],
        "an unfocused key starts at the App's view"
    );
    screen.press_key(Key::Enter);
    screen.press_key(Key::Space);
    assert_eq!(
        take(),
        ["app Enter Some(Group)", "app Space Some(Group)"],
        "an unfocused Enter or Space does not activate the App's view"
    );

    screen.get_by_test_id("control").focus();
    screen.press_key(Key::Char('x'));
    assert_eq!(
        take(),
        ["control Char('x')"],
        "the focused control handles the key first and stops it"
    );
    screen.press_key(Key::Char('y'));
    assert_eq!(
        take(),
        ["control Char('y')", "app Char('y') Some(Button)"],
        "an unhandled key bubbles from the focused control to the App's view"
    );

    open.set(true);
    screen.run_until_idle();
    assert!(screen.focused().is_none(), "the modal has no tab stop");
    screen.press_key(Key::Char('z'));
    assert_eq!(
        take(),
        ["modal Char('z')"],
        "with a modal open the key starts at the modal and stops there"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_context_menu_key_reaches_on_key_as_context_menu() {
    let (screen, log) = mount(|| {
        let log = signal(Vec::<Key>::new());
        let view = node()
            .role(Role::Group)
            .name("app")
            .on_key(move |e: KeyEvent| {
                log.update(|l| l.push(e.key));
                Handled::No
            });
        (log, view)
    });

    screen.press_key(Key::ContextMenu);
    assert_eq!(log.get_untracked(), [Key::ContextMenu]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
