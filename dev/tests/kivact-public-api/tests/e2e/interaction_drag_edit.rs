//! Role: Tests for drags whose edit begins at the first change.
//!
//! `Drag::edit_on_change`: a press that changes nothing reports no edit, the
//! first `Drag::change` begins it, and the release, Escape and an engine
//! Cancel end it. Escape stops any active drag and otherwise reaches the
//! component; `Drag::buttons` starts a drag with another button.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::custom::part;
use kivact::geometry::Point;
use kivact::interaction::{DragCoordinates, DragPhase, DragUpdate, EditSession, use_drag};
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions, Screen};
use kivact::view::EditPhase;
use kivact::view::{Handled, KeyEvent, MouseButton};

/// What the drag reported: its edit phases with the value at that time, the
/// ends `on_end` saw, and the Escapes that reached the component.
#[derive(Default)]
struct Log {
    edits: Vec<(EditPhase, f64)>,
    ends: Vec<(DragPhase, MouseButton)>,
    escapes: usize,
}

type Shared = Rc<RefCell<Log>>;

fn real_window() -> RenderOptions {
    RenderOptions::from_env().mirror(MirrorMode::Required)
}

/// A handle whose drag moves `x` in whole steps of 10 and edits it from the
/// first step; its own Escape (a part after the drag's source) counts in the log.
fn stepper(x: Signal<f64>, log: &Shared) -> Node {
    let (edits, ends, escapes) = (log.clone(), log.clone(), log.clone());
    let origin = signal(0.0_f64);
    let edit = EditSession::new(move |start| x.set(start))
        .on_edit(move |phase| edits.borrow_mut().edits.push((phase, x.get_untracked())));
    let drag = use_drag::<()>(DragCoordinates::Window)
        .buttons([MouseButton::Middle])
        .edit_on_change(edit, move || x.get_untracked())
        .on_start(move |_| origin.set(x.get_untracked()))
        .on_end(move |update: DragUpdate<()>| {
            ends.borrow_mut().ends.push((update.phase, update.button))
        });
    let drag = drag.on_update(move |update: DragUpdate<()>| {
        let next = origin.get_untracked() + (update.delta.x / 10.0).round() * 10.0;
        if next != x.get_untracked() {
            drag.change(|| x.set(next));
        }
    });
    let own = {
        let attachment = part(|input| input);
        kivact::custom::Part::new(attachment.state(), move |node| {
            node.part(attachment).on_key(move |event: KeyEvent| {
                if event.key() == Key::Escape {
                    escapes.borrow_mut().escapes += 1;
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
        })
    };
    node()
        .test_id("stepper")
        .name("Stepper")
        .focusable(true)
        .style(style().size(100, 40))
        .part(drag.source(()))
        .part(own)
}

fn center(screen: &Screen) -> Point {
    screen
        .get_by_test_id("stepper")
        .rect()
        .expect("laid out")
        .center()
}

fn mount_stepper() -> (Screen, Signal<f64>, Shared) {
    let log = Shared::default();
    let seen = log.clone();
    let (screen, x) = mount_with(real_window(), move || {
        let x = signal(0.0_f64);
        (x, stepper(x, &seen))
    });
    (screen, x, log)
}

use EditPhase::{Begin, Cancel, Commit};

#[test]
fn the_edit_begins_at_the_first_change_and_a_still_press_reports_none() {
    let (screen, x, log) = mount_stepper();
    let at = center(&screen);
    screen.mouse_move(at.x, at.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(at.x + 3.0, at.y);
    assert!(log.borrow().edits.is_empty(), "a move that changes nothing");
    screen.mouse_up(MouseButton::Primary);
    assert!(log.borrow().edits.is_empty(), "a press is not an edit");
    assert_eq!(log.borrow().ends, [(DragPhase::End, MouseButton::Primary)]);

    screen.mouse_move(at.x, at.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(at.x + 12.0, at.y);
    assert_eq!(log.borrow().edits, [(Begin, 0.0)], "before the first write");
    screen.mouse_move(at.x + 25.0, at.y);
    assert_eq!(x.get_untracked(), 30.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(log.borrow().edits, [(Begin, 0.0), (Commit, 30.0)]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn escape_stops_the_drag_and_cancels_only_a_begun_edit() {
    let (screen, x, log) = mount_stepper();
    let at = center(&screen);
    screen.mouse_move(at.x, at.y);
    screen.mouse_down(MouseButton::Primary);
    screen.press_key(Key::Escape);
    assert!(
        log.borrow().edits.is_empty(),
        "no change, nothing to cancel"
    );
    assert_eq!(
        log.borrow().ends,
        [(DragPhase::Cancel, MouseButton::Primary)]
    );
    assert_eq!(log.borrow().escapes, 0, "the drag consumed it");
    screen.mouse_move(at.x + 30.0, at.y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        x.get_untracked(),
        0.0,
        "the rest of the gesture is not a drag"
    );

    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(at.x + 50.0, at.y);
    assert_eq!(x.get_untracked(), 20.0);
    screen.press_key(Key::Escape);
    assert_eq!(
        log.borrow().edits,
        [(Begin, 0.0), (Cancel, 0.0)],
        "the start is back before Cancel"
    );
    assert_eq!(log.borrow().escapes, 0);
    screen.mouse_up(MouseButton::Primary);

    screen.press_key(Key::Escape);
    assert_eq!(
        log.borrow().escapes,
        1,
        "without a drag Escape reaches the component"
    );
    assert_eq!(log.borrow().edits.len(), 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_drag_during_an_edit_it_did_not_begin_leaves_that_edit_open() {
    let log = Shared::default();
    let seen = log.clone();
    let (screen, x) = mount_with(real_window(), move || {
        let x = signal(0.0_f64);
        let edits = seen.clone();
        let edit = EditSession::new(move |start| x.set(start))
            .on_edit(move |phase| edits.borrow_mut().edits.push((phase, x.get_untracked())));
        let source = use_drag::<()>(DragCoordinates::Window)
            .edit(edit, move || x.get_untracked())
            .on_update(move |update: DragUpdate<()>| x.set(update.delta.x))
            .source(());
        let button = |name: &'static str| {
            node()
                .role(Role::Button)
                .name(name)
                .test_id(name)
                .focusable(true)
                .style(style().size(100, 30).shrink(0))
        };
        (
            x,
            column((
                button("Begin").on_press(move || edit.begin(x.get_untracked())),
                button("Commit").on_press(move || edit.commit()),
                node()
                    .test_id("stepper")
                    .name("Stepper")
                    .focusable(true)
                    .style(style().size(100, 40))
                    .part(source),
            )),
        )
    });
    screen.get_by_test_id("Begin").click();
    assert_eq!(log.borrow().edits, [(Begin, 0.0)]);
    let at = center(&screen);
    screen.mouse_move(at.x, at.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(at.x + 20.0, at.y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(x.get_untracked(), 20.0);
    assert_eq!(
        log.borrow().edits,
        [(Begin, 0.0)],
        "the drag joined the edit and left it open"
    );
    screen.get_by_test_id("Commit").click();
    assert_eq!(log.borrow().edits, [(Begin, 0.0), (Commit, 20.0)]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn configured_drag(
    slop: f64,
    lock: bool,
    velocity: bool,
) -> (Screen, Rc<RefCell<Vec<DragUpdate<()>>>>) {
    let updates = Rc::new(RefCell::new(Vec::new()));
    let seen = updates.clone();
    let (screen, ()) = mount_with(real_window(), move || {
        let started = seen.clone();
        let moved = seen.clone();
        let ended = seen.clone();
        let mut drag = use_drag::<()>(DragCoordinates::Window).slop(slop);
        if lock {
            drag = drag.axis_lock();
        }
        if velocity {
            drag = drag.track_velocity();
        }
        let source = drag
            .on_start(move |update| started.borrow_mut().push(update))
            .on_update(move |update| moved.borrow_mut().push(update))
            .on_end(move |update| ended.borrow_mut().push(update))
            .source(());
        (
            (),
            node()
                .test_id("stepper")
                .name("Drag handle")
                .focusable(true)
                .style(style().size(100, 100))
                .part(source),
        )
    });
    (screen, updates)
}

#[test]
fn drag_slop_waits_before_starting() {
    let (screen, updates) = configured_drag(10.0, false, false);
    let at = center(&screen);
    screen.mouse_move(at.x, at.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(at.x + 6.0, at.y + 8.0);
    assert!(
        updates.borrow().is_empty(),
        "the threshold must be exceeded"
    );
    screen.mouse_up(MouseButton::Primary);
    assert!(
        updates.borrow().is_empty(),
        "a pending press ends without callbacks"
    );
    screen.mouse_move(at.x, at.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(at.x + 11.0, at.y);
    assert_eq!(
        updates
            .borrow()
            .iter()
            .map(|update| update.phase)
            .collect::<Vec<_>>(),
        [DragPhase::Start, DragPhase::Move]
    );
    assert_eq!(updates.borrow()[1].delta, Point::new(11.0, 0.0));
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        updates.borrow().last().expect("end update").phase,
        DragPhase::End
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn drag_axis_lock_keeps_the_first_axis() {
    let (screen, updates) = configured_drag(10.0, true, false);
    let at = center(&screen);
    screen.mouse_move(at.x, at.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(at.x + 12.0, at.y + 3.0);
    screen.mouse_move(at.x + 14.0, at.y + 35.0);
    screen.mouse_up(MouseButton::Primary);
    let updates = updates.borrow();
    assert_eq!(
        updates.last().expect("end update").delta,
        Point::new(14.0, 0.0)
    );
    assert!(updates.iter().all(|update| update.delta.y == 0.0));
    assert!(
        updates
            .iter()
            .all(|update| update.velocity == Point::new(0.0, 0.0)),
        "velocity tracking is opt-in"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn drag_track_velocity_reports_pixels_per_ms() {
    use std::time::Duration;
    let (screen, updates) = configured_drag(0.0, false, true);
    let at = center(&screen);
    screen.mouse_move(at.x, at.y);
    screen.mouse_down(MouseButton::Primary);
    screen.advance(Duration::from_millis(20));
    screen.mouse_move(at.x + 20.0, at.y);
    screen.advance(Duration::from_millis(20));
    screen.mouse_move(at.x + 40.0, at.y);
    let velocity = updates.borrow().last().expect("move update").velocity;
    assert!(
        (0.8..1.2).contains(&velocity.x),
        "40 pixels over 40 ms: {velocity:?}"
    );
    assert_eq!(velocity.y, 0.0);
    screen.advance(Duration::from_millis(150));
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        updates.borrow().last().expect("end update").velocity,
        Point::new(0.0, 0.0),
        "samples beyond the 100 ms window expire"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn drag_source_with_samples_once_at_the_accepted_press() {
    use std::cell::Cell;
    let calls = Rc::new(Cell::new(0));
    let updates = Rc::new(RefCell::new(Vec::new()));
    let sampled = calls.clone();
    let seen = updates.clone();
    let (screen, (payload, disabled)) = mount_with(real_window(), move || {
        let payload = signal(3);
        let disabled = signal(false);
        let started = seen.clone();
        let moved = seen.clone();
        let ended = seen.clone();
        let source = use_drag(DragCoordinates::Window)
            .slop(10)
            .disabled(disabled)
            .on_start(move |update: DragUpdate<i32>| started.borrow_mut().push(update.payload))
            .on_update(move |update: DragUpdate<i32>| moved.borrow_mut().push(update.payload))
            .on_end(move |update: DragUpdate<i32>| ended.borrow_mut().push(update.payload))
            .source_with(move || {
                sampled.set(sampled.get() + 1);
                payload.get_untracked()
            });
        (
            (payload, disabled),
            node()
                .test_id("stepper")
                .name("Drag snapshot")
                .focusable(true)
                .style(style().size(100, 100))
                .part(source),
        )
    });
    assert_eq!(calls.get(), 0, "setup must not sample the payload");
    let at = center(&screen);
    screen.mouse_move(at.x, at.y);
    disabled.set(true);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    disabled.set(false);
    screen.mouse_down(MouseButton::Secondary);
    screen.mouse_up(MouseButton::Secondary);
    assert_eq!(calls.get(), 0, "rejected presses must not call the factory");

    screen.mouse_down(MouseButton::Primary);
    assert_eq!(calls.get(), 1, "a pending press samples immediately");
    payload.set(7);
    screen.mouse_move(at.x + 5.0, at.y);
    assert!(updates.borrow().is_empty());
    screen.mouse_move(at.x + 15.0, at.y);
    screen.mouse_move(at.x + 25.0, at.y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        *updates.borrow(),
        [3, 3, 3, 3],
        "start, moves and end share the press-time snapshot"
    );
    assert_eq!(calls.get(), 1, "moves and release never resample");

    updates.borrow_mut().clear();
    screen.mouse_move(at.x, at.y);
    screen.mouse_down(MouseButton::Primary);
    payload.set(9);
    screen.mouse_move(at.x + 15.0, at.y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        *updates.borrow(),
        [7, 7, 7],
        "the next gesture takes a new snapshot"
    );
    assert_eq!(calls.get(), 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
