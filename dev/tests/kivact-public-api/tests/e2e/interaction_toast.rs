//! Role: Tests for toast notification queue and dismissal.
//!
//! The unstyled `Toaster` parts: the App-wide queue, timed
//! dismissal on the task timer that waits for the pointer and the focus, the
//! visible limit and its waiting line, live regions and roles by tone, Escape,
//! the action and dismissal by id.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::support::mount;
use kivact::interaction::ToastId;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::ui::{Toast, ToastOptions, Toaster, toaster};
use kivact::view::{LiveRegion, View};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// The card a kit would draw: the title, the body, a close button and the
/// toast's action button, if it has one.
fn card(id: ToastId, toast: Toast) -> Node {
    let mut card = column((
        text(toast.title.clone()),
        text(toast.body.clone().unwrap_or_default()),
        node()
            .role(Role::Button)
            .name(format!("Close {}", toast.title))
            .focusable(true)
            .style(style().size(60, 24))
            .on_press(move || toaster().dismiss(id))
            .child(text("x")),
    ))
    .style(style().width(300).padding(8).gap(4));
    if let Some((label, _)) = toast.action.clone() {
        card = card.child(
            node()
                .role(Role::Button)
                .name(label.clone())
                .focusable(true)
                .style(style().size(60, 24))
                .on_press(move || toast.activate(id))
                .child(text(label)),
        );
    }
    card
}

/// A focusable "Other" button outside the region.
fn other() -> Node {
    node()
        .role(Role::Button)
        .name("Other")
        .focusable(true)
        .style(style().size(100, 30))
        .child(text("Other"))
}

/// A region and the "Other" button.
fn region(render: impl Fn(ToastId, Toast) -> Node + Copy + 'static) -> Node {
    let toaster = toaster();
    let group = move |assertive, live| {
        column(for_each(
            toaster.visible(assertive),
            |id| *id,
            move |id, _| {
                let toast = toaster.toast(id).expect("invariant: visible toast exists");
                render(id, toast).part(toaster.item(id))
            },
        ))
        .part(toaster.region(live))
    };
    column((
        group(false, LiveRegion::Polite),
        group(true, LiveRegion::Assertive),
    ))
}

fn app() -> View {
    column((other(), region(card))).into_view()
}

fn mount_app() -> (Screen, Toaster) {
    mount(|| (toaster(), app()))
}

fn toast(title: &str, duration: Option<Duration>) -> Toast {
    Toast::new(title).duration(duration)
}

fn shown(screen: &Screen, title: &str) -> bool {
    screen
        .query_by_role(Role::Status, title)
        .or_else(|| screen.query_by_role(Role::Alert, title))
        .is_some()
}

fn ok(screen: &Screen) {
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dismissing_a_focused_toast_restores_the_previous_external_focus() {
    let (screen, toaster) = mount_app();
    let other = screen.get_by_role(Role::Button, "Other");
    other.focus();
    toaster.push(toast("Saved", None));
    screen.run_until_idle();
    let close = screen.get_by_role(Role::Button, "Close Saved");
    close.focus();
    assert!(close.is_focused());
    close.press_key(Key::Escape);
    assert!(!shown(&screen, "Saved"));
    assert!(other.is_focused());
    ok(&screen);
}

#[test]
fn keyed_toasts_deduplicate_and_only_a_shown_toast_runs_its_hide_callback() {
    let (screen, toaster) = mount_app();
    toaster.max_visible(1);
    let hidden = Rc::new(Cell::new(0));
    let options = {
        let hidden = hidden.clone();
        ToastOptions::default()
            .key("saved")
            .on_hide(move || hidden.set(hidden.get() + 1))
    };
    let first = toaster.push_with(toast("Saved", Some(ms(80))), options.clone());
    let second = toaster.push_with(toast("Duplicate", None), options.clone());
    assert_eq!(first, second);
    let waiting = toaster.push_with(toast("Queued", None), options.clone().key("queued"));
    screen.run_until_idle();
    assert!(shown(&screen, "Saved"));
    assert!(!shown(&screen, "Duplicate"));
    toaster.dismiss(waiting);
    assert_eq!(hidden.get(), 0);
    screen.advance(ms(80));
    assert_eq!(hidden.get(), 1);
    toaster.dismiss(first);
    assert_eq!(hidden.get(), 1);
    assert_ne!(toaster.push_with(toast("Saved", None), options), first);
    ok(&screen);
}

#[test]
fn a_pushed_toast_is_shown_and_goes_when_its_time_is_up_without_frame_polling() {
    let (screen, toaster) = mount_app();
    assert!(!shown(&screen, "Saved"));
    toaster.push(Toast::new("Saved").body("Written to disk."));
    screen.run_until_idle();
    assert!(shown(&screen, "Saved"));
    screen.get_by_text("Written to disk.");
    assert_eq!(screen.stats().timers, 1, "one deadline, no polling");
    assert_eq!(screen.stats().frame_loops, 0);
    let turns = screen.stats().turns;
    screen.advance(ms(4999));
    assert!(shown(&screen, "Saved"), "5 seconds by default");
    screen.advance(ms(1));
    assert!(!shown(&screen, "Saved"));
    assert_eq!(screen.stats().timers, 0);
    assert!(
        screen.stats().turns - turns <= 6,
        "the wait cost a few turns, not one per frame: {}",
        screen.stats().turns - turns
    );
    ok(&screen);
}

#[test]
fn escape_dismisses_the_toast_the_focus_is_in_and_only_that_one() {
    let (screen, toaster) = mount_app();
    toaster.push(toast("First", None));
    toaster.push(toast("Second", None));
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Other").focus();
    screen.press_key(Key::Escape);
    assert!(shown(&screen, "First") && shown(&screen, "Second"));
    screen.get_by_role(Role::Button, "Close First").focus();
    screen.press_key(Key::Escape);
    assert!(!shown(&screen, "First"));
    assert!(shown(&screen, "Second"));
    ok(&screen);
}

#[test]
fn the_action_runs_its_callback_and_dismisses_the_toast() {
    let (screen, (count, toaster, undo)) = mount(|| {
        let count = signal(0);
        let undo = Toast::new("Deleted")
            .action("Undo", move || count.update(|c| *c += 1))
            .duration(None);
        ((count, toaster(), undo), app())
    });
    toaster.push(undo);
    toaster.push(toast("Other toast", None));
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Undo").click();
    assert_eq!(count.get_untracked(), 1);
    assert!(!shown(&screen, "Deleted"));
    assert!(shown(&screen, "Other toast"));
    ok(&screen);
}
