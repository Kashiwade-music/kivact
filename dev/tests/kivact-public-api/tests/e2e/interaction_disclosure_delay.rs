//! Role: Tests for the single host-clock deadline of a delayed disclosure.

use kivact::primitive::NodeExt;
use std::time::Duration;

use crate::support::mount;
use kivact::binding;
use kivact::diagnostics::ErrorKind;
use kivact::interaction::use_disclosure;
use kivact::prelude::*;
use kivact::view::{Handled, Placement, overlay};
use kivact::view::{MouseButton, PointerKind, PointerPhase};

#[test]
fn delayed_disclosure_cancels_old_deadlines_and_supports_focus_only() {
    let (screen, alive) = mount(|| {
        let alive = signal(true);
        (
            alive,
            column((
                show(alive, move || {
                    let anchor = node_ref();
                    let hover = signal(false);
                    let focused = anchor.focus_within();
                    let active = memo(move || hover.get() || focused.get());
                    let disclosure = use_disclosure(binding(active, |_: bool| {}))
                        .delay(Duration::from_millis(250));
                    node()
                        .role(Role::Button)
                        .name("Delayed hint target")
                        .focusable(true)
                        .style(style().size(160, 40))
                        .child(text("Delayed hint target"))
                        .node_ref(anchor)
                        .on_pointer_default(move |event| match event.phase {
                            PointerPhase::Enter => hover.set(true),
                            PointerPhase::Leave => hover.set(false),
                            PointerPhase::Cancel | PointerPhase::Down => disclosure.close(),
                            _ => {}
                        })
                        .on_key_default(move |_| disclosure.close())
                        .child(show(disclosure.shown(), move || {
                            overlay(anchor, Placement::Above, text("Deadline hint"))
                                .role(Role::Tooltip)
                                .name("Deadline hint")
                        }))
                }),
                {
                    let label = ("Hint outside").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .style(style().margin_top(80)),
            )),
        )
    });
    screen
        .get_by_role(Role::Button, "Delayed hint target")
        .hover();
    screen.advance(Duration::from_millis(200));
    screen.mouse_leave();
    screen
        .get_by_role(Role::Button, "Delayed hint target")
        .hover();
    screen.advance(Duration::from_millis(50));
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Deadline hint")
            .is_none()
    );
    screen.advance(Duration::from_millis(199));
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Deadline hint")
            .is_none()
    );
    screen.advance(Duration::from_millis(1));
    screen.get_by_role(Role::Tooltip, "Deadline hint");
    screen.mouse_leave();
    screen
        .get_by_role(Role::Button, "Delayed hint target")
        .focus();
    screen.advance(Duration::from_millis(250));
    screen.get_by_role(Role::Tooltip, "Deadline hint");
    screen.press_key(Key::Char('x'));
    screen.advance(Duration::from_secs(1));
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Deadline hint")
            .is_none()
    );
    screen.get_by_role(Role::Button, "Hint outside").focus();
    screen
        .get_by_role(Role::Button, "Delayed hint target")
        .focus();
    screen.advance(Duration::from_millis(100));
    alive.set(false);
    screen.run_until_idle();
    screen.advance(Duration::from_secs(1));
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Deadline hint")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_captured_pointer_cancel_removes_a_disclosure_deadline_and_suppresses_reopening() {
    let (screen, (active, disclosure)) = mount(|| {
        let active = signal(false);
        let disclosure = use_disclosure(active).delay(Duration::from_millis(250));
        (
            (active, disclosure),
            node()
                .role(Role::Button)
                .name("Cancel hint target")
                .style(style().size(160, 40))
                .on_pointer_default(move |event| match event.phase {
                    PointerPhase::Down if event.button == Some(MouseButton::Primary) => {
                        Handled::Yes
                    }
                    PointerPhase::Cancel => {
                        disclosure.close();
                        Handled::Yes
                    }
                    _ => Handled::No,
                })
                .child(show(disclosure.shown(), || text("Canceled deadline hint"))),
        )
    });
    let timers = screen.stats().timers;
    active.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(200));
    let rect = screen
        .get_by_role(Role::Button, "Cancel hint target")
        .rect()
        .unwrap();
    let contact = screen.contact_down(PointerKind::Touch, rect.x + 10.0, rect.y + 10.0);
    screen.contact_cancel(contact);
    assert_eq!(
        screen.stats().timers,
        timers,
        "Cancel removes the host deadline"
    );
    screen.advance(Duration::from_secs(1));
    assert!(!disclosure.shown().get_untracked());
    assert!(screen.query_by_text("Canceled deadline hint").is_none());
    active.set(false);
    screen.run_until_idle();
    active.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(250));
    screen.get_by_text("Canceled deadline hint");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_rejected_disclosure_dismiss_keeps_the_pending_deadline() {
    let (screen, (reject, disclosure)) = mount(|| {
        let reject = signal(false);
        let disclosure = use_disclosure(binding(true.into_value(), |_: bool| {}))
            .delay(Duration::from_millis(250));
        (
            (reject, disclosure),
            column((
                text(move || {
                    if reject.get() {
                        disclosure.close();
                    }
                    "Pure disclosure observer".to_owned()
                }),
                show(disclosure.shown(), || text("Kept deadline hint")),
            )),
        )
    });
    screen.advance(Duration::from_millis(200));
    reject.set(true);
    screen.run_until_idle();
    assert_eq!(screen.errors().len(), 1);
    assert_eq!(screen.errors()[0].kind(), &ErrorKind::WriteInPureContext);
    reject.set(false);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(50));
    assert!(disclosure.shown().get_untracked());
    screen.get_by_text("Kept deadline hint");
    assert_eq!(screen.errors().len(), 1);
}
