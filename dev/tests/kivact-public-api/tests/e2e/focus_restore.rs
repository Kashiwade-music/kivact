//! Role: Tests for focus restoration after UI changes.
//!
//! `NodeRef::restore_focus`: the focus goes back to the bound node after a
//! part closes, unless by the time the turn's writes are sent the node was
//! removed, is hidden, disabled or not focusable, or lies outside a modal
//! that stays open; an explicit focus request of the same turn wins, and a
//! disposed ref does nothing. None of these is an error.

use std::cell::Cell;
use std::rc::Rc;

use crate::support::mount;
use kivact::draw::Point;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::Placement;

/// What the close button does to the anchor in the turn it closes the
/// panel and restores the focus.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Close {
    Plain,
    Remove,
    Hide,
    Disable,
    Unfocusable,
    OpenModal,
    FocusOtherFirst,
    FocusOtherAfter,
}

struct Handles {
    open: Signal<bool>,
    case: Signal<Close>,
}

fn focused_name(screen: &Screen) -> Option<String> {
    screen.focused().map(|e| e.name())
}

fn button(label: &'static str) -> Node {
    node()
        .role(Role::Button)
        .focusable(true)
        .name(label)
        .child(text(label))
}

fn panel() -> (Screen, Handles) {
    mount(|| {
        let open = signal(false);
        let case = signal(Close::Plain);
        let (mounted, hidden, disabled, focusable, modal) = (
            signal(true),
            signal(false),
            signal(false),
            signal(true),
            signal(false),
        );
        let (anchor, other) = (node_ref(), node_ref());
        let close = move || {
            let c = case.get_untracked();
            if c == Close::FocusOtherFirst {
                other.focus();
            }
            open.set(false);
            match c {
                Close::Remove => mounted.set(false),
                Close::Hide => hidden.set(true),
                Close::Disable => disabled.set(true),
                Close::Unfocusable => focusable.set(false),
                Close::OpenModal => modal.set(true),
                _ => {}
            }
            anchor.restore_focus();
            if c == Close::FocusOtherAfter {
                other.focus();
            }
        };
        let view = column((
            show(mounted, move || {
                node()
                    .role(Role::Button)
                    .name("anchor")
                    .child(text("anchor"))
                    .focusable(focusable)
                    .disabled(disabled)
                    .hidden(hidden)
                    .node_ref(anchor)
                    .on_press(move || open.set(true))
            }),
            // The close button sits before "other": once the panel is
            // gone the host moves the focus on to "other" by itself.
            show(open, move || column((button("close").on_press(close),))),
            kivact::control::node()
                .name("other")
                .part(kivact::interaction::use_text_edit(kivact::signal(String::new())).control())
                .node_ref(other),
            show(modal, || {
                kivact::view::overlay_at(
                    Point::new(300.0, 300.0),
                    Placement::Center,
                    button("in modal"),
                )
                .modal(true)
            }),
        ));
        (Handles { open, case }, view)
    })
}

#[test]
fn restore_focus_returns_to_an_eligible_anchor_and_is_cancelled_otherwise() {
    let expected = [
        (Close::Plain, "anchor"),
        (Close::Remove, "other"),
        (Close::Hide, "other"),
        (Close::Disable, "other"),
        (Close::Unfocusable, "other"),
        (Close::OpenModal, "in modal"),
        (Close::FocusOtherFirst, "other"),
        (Close::FocusOtherAfter, "other"),
    ];
    for (case, want) in expected {
        let (screen, h) = panel();
        h.case.set(case);
        screen.get_by_role(Role::Button, "anchor").click();
        assert!(h.open.get_untracked());
        screen.get_by_role(Role::Button, "close").click();
        assert!(!h.open.get_untracked(), "{case:?}: closed");
        assert_eq!(focused_name(&screen).as_deref(), Some(want), "{case:?}");
        assert!(
            screen.errors().is_empty(),
            "{case:?}: a cancelled restore is no error: {:?}",
            screen.errors()
        );
    }
}

/// A ref whose owner is gone restores nothing and reports nothing (where
/// `NodeRef::focus` would report the disposed ref).
#[test]
fn restore_focus_through_a_disposed_ref_does_nothing() {
    let (screen, shown) = mount(|| {
        let shown = signal(true);
        let stale: Rc<Cell<Option<NodeRef>>> = Rc::default();
        let keep = stale.clone();
        let view = column((
            show(shown, move || {
                let inner = node_ref();
                keep.set(Some(inner));
                button("inner").node_ref(inner)
            }),
            button("restore").on_press(move || {
                if let Some(inner) = stale.get() {
                    inner.restore_focus();
                }
            }),
        ));
        (shown, view)
    });
    shown.set(false);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "restore").click();
    assert_eq!(focused_name(&screen).as_deref(), Some("restore"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
