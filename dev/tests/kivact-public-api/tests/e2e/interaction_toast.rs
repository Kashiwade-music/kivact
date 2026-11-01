//! Role: Tests for toast notification queue and dismissal.
//!
//! The unstyled `Toaster` parts: the App-wide queue, timed
//! dismissal on the task timer that waits for the pointer and the focus, the
//! visible limit and its waiting line, live regions and roles by tone, Escape,
//! the action and dismissal by id.

use kivact::view::View;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use crate::support::mount;
use kivact::interaction::ToastId;
use kivact::prelude::*;
use kivact::task::sleep;
use kivact::task::spawn;
use kivact::test::Screen;
use kivact::ui::{Toast, Toaster, Tone, toaster};
use kivact::view::{LiveRegion, try_context};

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

fn shown_count(toaster: Toaster) -> usize {
    toaster.toasts().with_untracked(|shown| shown.len())
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
fn each_toast_has_its_own_time() {
    let (screen, toaster) = mount_app();
    toaster.push(toast("Short", Some(ms(1000))));
    screen.advance(ms(400));
    toaster.push(toast("Long", Some(ms(3000))));
    screen.run_until_idle();
    screen.advance(ms(599));
    assert!(shown(&screen, "Short"));
    screen.advance(ms(1));
    assert!(!shown(&screen, "Short"));
    assert!(shown(&screen, "Long"));
    screen.advance(ms(2399));
    assert!(shown(&screen, "Long"));
    screen.advance(ms(1));
    assert!(!shown(&screen, "Long"));
    ok(&screen);
}

#[test]
fn a_toast_without_a_duration_stays_until_it_is_dismissed() {
    let (screen, toaster) = mount_app();
    let id = toaster.push(toast("Stays", None));
    screen.advance(Duration::from_secs(3600));
    assert!(shown(&screen, "Stays"));
    assert_eq!(screen.stats().timers, 0, "no deadline at all");
    toaster.dismiss(id);
    screen.run_until_idle();
    assert!(!shown(&screen, "Stays"));
    toaster.dismiss(id);
    ok(&screen);
}

#[test]
fn the_pointer_over_a_toast_stops_its_time_and_the_rest_resumes_when_it_leaves() {
    let (screen, toaster) = mount_app();
    toaster.push(toast("Hover", Some(ms(5000))).body("Hover body"));
    toaster.push(toast("Beside", Some(ms(5000))).body("Beside body"));
    screen.advance(ms(2000));
    screen.get_by_text("Hover body").hover();
    screen.advance(Duration::from_secs(60));
    assert!(shown(&screen, "Hover"), "held while the pointer is over it");
    assert!(
        !shown(&screen, "Beside"),
        "the other toast went on counting"
    );
    assert_eq!(screen.stats().timers, 0, "no deadline while held");
    screen.mouse_leave();
    screen.advance(ms(2999));
    assert!(shown(&screen, "Hover"), "3 seconds were left");
    screen.advance(ms(1));
    assert!(!shown(&screen, "Hover"));
    ok(&screen);
}

#[test]
fn leaving_and_coming_back_counts_each_visit_once() {
    let (screen, toaster) = mount_app();
    toaster.push(toast("Visit", Some(ms(5000))).body("Visit body"));
    screen.advance(ms(1000));
    screen.get_by_text("Visit body").hover();
    screen.advance(ms(1000));
    screen.mouse_leave();
    screen.advance(ms(1000));
    screen.get_by_text("Visit body").hover();
    screen.advance(ms(1000));
    screen.mouse_leave();
    screen.advance(ms(2999));
    assert!(
        shown(&screen, "Visit"),
        "2 seconds were counted before and 2999 ms after"
    );
    screen.advance(ms(1));
    assert!(!shown(&screen, "Visit"));
    ok(&screen);
}

#[test]
fn the_focus_inside_a_toast_stops_its_time_too() {
    let (screen, toaster) = mount_app();
    toaster.push(toast("Focus", Some(ms(5000))));
    screen.advance(ms(1000));
    screen.get_by_role(Role::Button, "Close Focus").focus();
    screen.advance(Duration::from_secs(60));
    assert!(shown(&screen, "Focus"), "held while the focus is inside");
    assert_eq!(screen.stats().timers, 0);
    screen.get_by_role(Role::Button, "Other").focus();
    screen.advance(ms(3999));
    assert!(shown(&screen, "Focus"), "4 seconds were left");
    screen.advance(ms(1));
    assert!(!shown(&screen, "Focus"));
    ok(&screen);
}

#[test]
fn the_time_is_held_until_both_the_pointer_and_the_focus_have_left() {
    let (screen, toaster) = mount_app();
    toaster.push(toast("Both", Some(ms(2000))).body("Both body"));
    screen.run_until_idle();
    screen.get_by_text("Both body").hover();
    screen.get_by_role(Role::Button, "Close Both").focus();
    screen.mouse_leave();
    screen.advance(Duration::from_secs(30));
    assert!(shown(&screen, "Both"), "the focus still holds it");
    screen.get_by_text("Both body").hover();
    screen.get_by_role(Role::Button, "Other").focus();
    screen.advance(Duration::from_secs(30));
    assert!(shown(&screen, "Both"), "the pointer still holds it");
    screen.mouse_leave();
    screen.advance(ms(2000));
    assert!(!shown(&screen, "Both"));
    ok(&screen);
}

#[test]
fn at_most_max_visible_toasts_are_shown_and_the_rest_wait_without_counting() {
    let (screen, toaster) = mount_app();
    for (title, time) in [
        ("A", 1000),
        ("B", 2000),
        ("C", 3000),
        ("D", 1000),
        ("E", 1000),
    ] {
        toaster.push(toast(title, Some(ms(time))));
    }
    screen.run_until_idle();
    assert_eq!(shown_count(toaster), 3);
    for title in ["A", "B", "C"] {
        assert!(shown(&screen, title), "{title}");
    }
    for title in ["D", "E"] {
        assert!(!shown(&screen, title), "{title} waits");
    }
    screen.advance(ms(1000));
    assert!(!shown(&screen, "A"));
    assert!(
        shown(&screen, "D"),
        "D takes A's place, in the order pushed"
    );
    assert!(!shown(&screen, "E"), "three are shown");
    screen.advance(ms(999));
    assert!(shown(&screen, "D"), "D counts from when it was shown");
    screen.advance(ms(1));
    assert!(!shown(&screen, "B") && !shown(&screen, "D"));
    assert!(shown(&screen, "E") && shown(&screen, "C"));
    screen.advance(ms(1000));
    assert!(!shown(&screen, "E"));
    screen.advance(ms(1000));
    assert!(!shown(&screen, "C"));
    assert_eq!(screen.stats().timers, 0);
    ok(&screen);
}

#[test]
fn max_visible_changes_how_many_are_shown_now() {
    let (screen, toaster) = mount_app();
    let toaster = toaster.max_visible(1);
    for title in ["A", "B", "C"] {
        toaster.push(toast(title, None));
    }
    screen.run_until_idle();
    assert!(shown(&screen, "A") && !shown(&screen, "B"));
    toaster.max_visible(2);
    screen.run_until_idle();
    assert!(shown(&screen, "A") && shown(&screen, "B") && !shown(&screen, "C"));
    toaster.max_visible(0);
    screen.run_until_idle();
    assert!(shown(&screen, "A") && !shown(&screen, "B"), "at least one");
    toaster.max_visible(5);
    screen.run_until_idle();
    assert!(shown(&screen, "C"));
    ok(&screen);
}

#[test]
fn warnings_and_errors_are_alerts_in_an_assertive_region_the_rest_are_polite_statuses() {
    let (screen, toaster) = mount_app();
    let groups = screen.all_by_role(Role::Group, "");
    assert_eq!(groups.len(), 2, "both live regions exist before any toast");
    assert_eq!(groups[0].live_region(), LiveRegion::Polite);
    assert_eq!(groups[1].live_region(), LiveRegion::Assertive);
    toaster.max_visible(4);
    for (title, tone) in [
        ("Info", Tone::Info),
        ("Success", Tone::Success),
        ("Warning", Tone::Warning),
        ("Error", Tone::Danger),
    ] {
        toaster.push(toast(title, None).tone(tone));
    }
    screen.run_until_idle();
    let groups = screen.all_by_role(Role::Group, "");
    assert_eq!(groups.len(), 2, "the regions are the same two");
    for title in ["Info", "Success"] {
        let item = screen.get_by_role(Role::Status, title);
        assert_eq!(item.name(), title);
        assert!(
            screen
                .within(groups[0])
                .query_by_role(Role::Status, title)
                .is_some()
        );
        assert!(screen.query_by_role(Role::Alert, title).is_none());
    }
    for title in ["Warning", "Error"] {
        let item = screen.get_by_role(Role::Alert, title);
        assert_eq!(item.name(), title);
        assert!(
            screen
                .within(groups[1])
                .query_by_role(Role::Alert, title)
                .is_some()
        );
        assert!(screen.query_by_role(Role::Status, title).is_none());
    }
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

#[test]
fn a_toast_pushed_from_a_press_handler_owns_its_action() {
    let (screen, count) = mount(|| {
        let count = signal(0);
        let view = column((
            node()
                .role(Role::Button)
                .name("Delete")
                .on_press(move || {
                    toaster().push(
                        Toast::new("Deleted")
                            .action("Undo", move || count.update(|c| *c += 1))
                            .duration(None),
                    );
                })
                .style(style().size(80, 30)),
            region(card),
        ));
        (count, view)
    });
    screen.get_by_role(Role::Button, "Delete").click();
    screen.get_by_role(Role::Button, "Undo").click();
    assert_eq!(count.get_untracked(), 1);
    assert!(!shown(&screen, "Deleted"));
    ok(&screen);
}

#[test]
fn an_action_outlives_the_component_that_made_it_and_its_error_is_reported() {
    let (screen, (alive, toaster, undo, runs)) = mount(|| {
        let alive = signal(true);
        let runs = signal(0);
        let undo: Rc<RefCell<Option<Toast>>> = Rc::new(RefCell::new(None));
        let slot = undo.clone();
        let view = column((
            show(alive, move || {
                *slot.borrow_mut() = Some(
                    Toast::new("Gone")
                        .action("Undo", move || {
                            runs.update(|n| *n += 1);
                            Err::<(), _>(UiError::msg("undo failed"))
                        })
                        .duration(None),
                );
                text("owner")
            }),
            region(card),
        ));
        ((alive, toaster(), undo, runs), view)
    });
    let undo = undo
        .borrow_mut()
        .take()
        .expect("invariant: the owner made the toast");
    toaster.push(undo);
    alive.set(false);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Undo").click();
    assert_eq!(runs.get_untracked(), 1, "the toast owns its action");
    assert!(!shown(&screen, "Gone"));
    assert_eq!(screen.errors().len(), 1, "{:?}", screen.errors());
}

#[test]
fn dismiss_removes_a_shown_toast_and_a_waiting_one_never_shows() {
    let (screen, toaster) = mount_app();
    let ids: Vec<ToastId> = ["A", "B", "C", "D"]
        .into_iter()
        .map(|title| toaster.push(toast(title, None)))
        .collect();
    screen.run_until_idle();
    assert!(!shown(&screen, "D"));
    toaster.dismiss(ids[3]);
    toaster.dismiss(ids[1]);
    screen.run_until_idle();
    assert!(shown(&screen, "A") && shown(&screen, "C"));
    assert!(
        !shown(&screen, "B") && !shown(&screen, "D"),
        "D was dismissed while it waited"
    );
    toaster.dismiss(ids[1]);
    toaster.dismiss(ids[0]);
    toaster.dismiss(ids[2]);
    screen.run_until_idle();
    assert!(!shown(&screen, "A") && !shown(&screen, "C"));
    assert_eq!(shown_count(toaster), 0);
    ok(&screen);
}

#[test]
fn an_app_without_a_toaster_has_no_toast_state_and_the_first_use_makes_it() {
    let (plain, ()) = mount(|| {
        assert!(try_context::<Toaster>().is_none());
        ((), column((text("plain"), other())))
    });
    let (used, ()) = mount(|| {
        assert!(try_context::<Toaster>().is_none());
        let first = toaster();
        assert!(try_context::<Toaster>().is_some(), "made under the root");
        first.push(toast("Early", None));
        assert_eq!(shown_count(toaster()), 1, "the same toaster again");
        ((), column((text("plain"), other())))
    });
    assert!(
        used.stats().live_reactive > plain.stats().live_reactive,
        "only the App that used it has the queue"
    );
    assert_eq!(used.stats().live_owners, plain.stats().live_owners);
    assert!(!shown(&plain, "Early"));
    ok(&plain);
    ok(&used);
}

#[test]
fn a_task_pushes_and_a_handle_outlives_the_component_that_took_it() {
    let (screen, (alive, taken)) = mount(|| {
        let alive = signal(true);
        let taken: Rc<Cell<Option<Toaster>>> = Rc::new(Cell::new(None));
        let slot = taken.clone();
        spawn(async move {
            sleep(ms(100)).await;
            toaster().push(toast("From a task", None));
        });
        let view = column((
            show(alive, move || {
                slot.set(Some(toaster()));
                spawn(async move {
                    sleep(ms(200)).await;
                    toaster().push(toast("From a gone task", None));
                });
                text("child")
            }),
            region(card),
        ));
        ((alive, taken), view)
    });
    screen.advance(ms(100));
    assert!(shown(&screen, "From a task"));
    alive.set(false);
    screen.advance(ms(300));
    assert!(
        !shown(&screen, "From a gone task"),
        "a task goes with its owner and pushes nothing"
    );
    let toaster = taken.get().expect("invariant: the child took the handle");
    toaster.push(toast("Late", None));
    screen.run_until_idle();
    assert!(shown(&screen, "Late"), "the handle belongs to the App");
    ok(&screen);
}

#[test]
fn outside_a_running_app_get_reports_a_located_error_instead_of_hanging() {
    let caught = std::panic::catch_unwind(toaster);
    assert!(caught.is_err(), "no App is running on this thread");
}

#[test]
fn toasts_release_owners_and_objects_and_bound_the_reactive_cache() {
    let (screen, toaster) = mount_app();
    toaster.push(toast("Warm", Some(ms(10))));
    screen.advance(ms(10));
    let before = screen.stats();
    for round in 0..32 {
        for n in 0..5 {
            let title = format!("T{round}-{n}");
            toaster.push(toast(&title, Some(ms(100 * (n + 1)))));
        }
        screen.advance(ms(2000));
        let current = screen.stats();
        assert_eq!(current.live_owners, before.live_owners);
        assert_eq!(current.live_objects, before.live_objects);
        assert_eq!(current.timers, 0);
        // Shared focus selectors retain a bounded cache of previously read keys.
        assert!(current.live_reactive <= before.live_reactive + 64);
    }
    let after = screen.stats();
    assert_eq!(after.live_owners, before.live_owners);
    assert_eq!(after.live_objects, before.live_objects);
    assert_eq!(after.timers, 0);
    ok(&screen);
}

#[test]
fn a_toast_carries_its_icon_to_the_kit_render() {
    let (screen, toaster) = mount(|| {
        (
            toaster(),
            region(|_, toast| {
                let icon = toast.icon.as_ref().map(|icon| icon());
                column((icon, text(toast.title.clone())))
            }),
        )
    });
    toaster.push(Toast::new("Plain").duration(None));
    toaster.push(
        Toast::new("Iconic")
            .duration(None)
            .icon(|| text("ICON").test_id("toast-icon")),
    );
    screen.run_until_idle();
    assert_eq!(screen.all_by_test_id("toast-icon").len(), 1);
    assert!(
        screen
            .within(screen.get_by_role(Role::Status, "Iconic"))
            .query_by_test_id("toast-icon")
            .is_some()
    );
    assert!(
        screen
            .within(screen.get_by_role(Role::Status, "Plain"))
            .query_by_test_id("toast-icon")
            .is_none()
    );
    ok(&screen);
}

#[test]
fn a_hold_stops_the_time_of_every_toast_and_the_rest_resumes_when_it_ends() {
    let (screen, (toaster, held)) = mount(|| {
        let held = signal(false);
        let toaster = toaster();
        toaster.hold_while(held);
        ((toaster, held), app())
    });
    toaster.push(toast("First", Some(ms(5000))));
    toaster.push(toast("Second", Some(ms(5000))));
    screen.advance(ms(2000));
    held.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_secs(60));
    assert!(shown(&screen, "First") && shown(&screen, "Second"));
    assert_eq!(screen.stats().timers, 0, "no deadline while held");
    held.set(false);
    screen.run_until_idle();
    screen.advance(ms(2999));
    assert!(shown(&screen, "First") && shown(&screen, "Second"));
    screen.advance(ms(1));
    assert!(!shown(&screen, "First") && !shown(&screen, "Second"));
    ok(&screen);
}
