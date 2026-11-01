//! Role: Tests that the engine alone decides what input a shown modal admits.
//!
//! The engine confines input to the topmost shown modal; kivact asks it
//! once per event, right before dispatching the event, so a modal that an
//! earlier event of the same turn opened already applies to the later
//! ones. A drag that began before the modal keeps its moves and release,
//! the overlay an unhandled Escape dismisses lies in the modal, and a
//! focus restore never lands outside a modal that stays open.

use kivact::primitive::NodeExt;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::{mount, mount_with};
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, WindowMode};
use kivact::view::{Handled, MouseButton, Placement, PointerEvent, PointerPhase, overlay};

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        ..RenderOptions::from_env()
    }
}

fn button(label: &'static str) -> Node {
    node()
        .role(Role::Button)
        .focusable(true)
        .name(label)
        .child(text(label))
}

fn focused_name(screen: &Screen) -> Option<String> {
    screen.focused().map(|e| e.name())
}

/// The notes of every turn so far.
fn notes(screen: &Screen) -> Vec<String> {
    screen.trace().into_iter().flat_map(|t| t.notes).collect()
}

fn dropped(screen: &Screen, what: &str) -> usize {
    notes(screen)
        .iter()
        .filter(|n| n.starts_with(what) && n.ends_with("outside the open modal overlay dropped"))
        .count()
}

type Phases = Rc<RefCell<Vec<PointerPhase>>>;

/// A listener the modal opens beside, logging every pointer phase it gets.
/// `opens_on` is the phase whose handler shows the modal.
fn listener_with_modal(
    log: Phases,
    opens_on: Option<PointerPhase>,
    presses: Rc<Cell<u32>>,
) -> (Signal<bool>, impl IntoView) {
    let open = signal(false);
    let anchor = node_ref();
    let pad = button("Pad")
        .node_ref(anchor)
        .test_id("pad")
        .style(style().size(100, 80))
        .on_pointer(move |event: PointerEvent| {
            log.borrow_mut().push(event.phase);
            if Some(event.phase) == opens_on {
                open.set(true);
            }
            Handled::No
        })
        .on_press(move || presses.set(presses.get() + 1));
    (
        open,
        (
            pad,
            show(open, move || {
                overlay(
                    anchor,
                    Placement::Below,
                    node().style(style().size(120, 80)),
                )
                .modal(true)
                .role(Role::Dialog)
                .name("Modal")
            }),
        ),
    )
}

fn terminal_phases(log: &Phases) -> Vec<PointerPhase> {
    let mut phases: Vec<PointerPhase> = log
        .borrow()
        .iter()
        .copied()
        .filter(|p| {
            matches!(
                p,
                PointerPhase::Down | PointerPhase::Move | PointerPhase::Up | PointerPhase::Cancel
            )
        })
        .collect();
    phases.dedup();
    phases
}

#[test]
fn a_drag_started_before_a_modal_opens_keeps_its_moves_and_its_release() {
    let log = Phases::default();
    let (screen, open) = mount_with(real_window(), {
        let log = log.clone();
        move || listener_with_modal(log, None, Rc::default())
    });
    let pad = screen.get_by_test_id("pad").rect().expect("laid out");
    screen.mouse_move(pad.x + 20.0, pad.y + 20.0);
    log.borrow_mut().clear();
    screen.mouse_down(MouseButton::Primary);
    open.set(true);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Dialog, "Modal").is_some());
    screen.mouse_move(400.0, 300.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        terminal_phases(&log),
        [PointerPhase::Down, PointerPhase::Move, PointerPhase::Up],
        "the drag the modal found in progress finishes on its listener"
    );
    // The drag is over: a new press where the listener is stays behind the
    // modal (the engine puts it on the backdrop, nothing reaches the pad).
    log.borrow_mut().clear();
    screen.mouse_move(pad.x + 20.0, pad.y + 20.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert!(
        terminal_phases(&log).is_empty(),
        "a press outside the modal reaches nothing: {:?}",
        log.borrow()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_press_that_follows_the_event_opening_a_modal_in_the_same_turn_is_dropped() {
    let (log, presses) = (Phases::default(), Rc::new(Cell::new(0)));
    let (screen, _open) = mount_with(real_window(), {
        let (log, presses) = (log.clone(), presses.clone());
        move || listener_with_modal(log, Some(PointerPhase::Enter), presses)
    });
    let before = screen.stats().stale_events;
    // The click is one turn: the pointer enters the pad, presses, releases
    // and activates it. The Enter handler shows the modal, so every event
    // after it is aimed at a node outside the open modal and no gesture was
    // admitted before it: the press, the release and the activation are
    // dropped, each with its diagnostic.
    screen.get_by_test_id("pad").click();
    assert!(screen.query_by_role(Role::Dialog, "Modal").is_some());
    let seen = log.borrow().clone();
    assert!(
        seen.contains(&PointerPhase::Enter)
            && !seen.contains(&PointerPhase::Down)
            && !seen.contains(&PointerPhase::Up),
        "the Enter that opened the modal arrived, the press after it did not: {seen:?}"
    );
    assert_eq!(presses.get(), 0);
    assert!(
        dropped(&screen, "pointer event") >= 2,
        "the press and the release: {:?}",
        notes(&screen)
    );
    assert_eq!(
        dropped(&screen, "activate event"),
        1,
        "{:?}",
        notes(&screen)
    );
    assert!(
        screen.stats().stale_events >= before + 3,
        "counted as dropped"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_click_whose_press_opens_a_modal_finishes_but_its_activation_is_dropped() {
    let (log, presses) = (Phases::default(), Rc::new(Cell::new(0)));
    let (screen, _open) = mount_with(real_window(), {
        let (log, presses) = (log.clone(), presses.clone());
        move || listener_with_modal(log, Some(PointerPhase::Down), presses)
    });
    // Down, Up and the click's Activate are one turn: the Down handler
    // shows the modal, the Up of the same gesture is still admitted and the
    // Activate aimed at the pad outside the modal is not.
    screen.get_by_test_id("pad").click();
    assert!(screen.query_by_role(Role::Dialog, "Modal").is_some());
    assert_eq!(
        terminal_phases(&log),
        [PointerPhase::Down, PointerPhase::Up],
        "the gesture admitted at its Down ends on its listener"
    );
    assert_eq!(
        presses.get(),
        0,
        "the activation behind the modal is dropped"
    );
    assert_eq!(
        dropped(&screen, "activate event"),
        1,
        "{:?}",
        notes(&screen)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The screen, the open flags of the dialog, its popup and the later popup,
/// and how often each was dismissed.
type EscapeScreen = (Screen, [Signal<bool>; 3], [Rc<Cell<u32>>; 3]);

/// A modal dialog holding a popup, and a popup that comes after the dialog
/// in tree order but lies outside it.
fn escape_screen() -> EscapeScreen {
    let counts: [Rc<Cell<u32>>; 3] = Default::default();
    let c = counts.clone();
    let (screen, signals) = mount(move || {
        let (dialog_on, popup_on, late_on) = (signal(true), signal(true), signal(true));
        let [dialog_count, popup_count, late_count] = c;
        let anchor = node_ref();
        (
            [dialog_on, popup_on, late_on],
            column((
                button("Page"),
                show(dialog_on, move || {
                    let (popup_count, dialog_count) = (popup_count.clone(), dialog_count.clone());
                    overlay(
                        node_ref(),
                        Placement::Center,
                        column((
                            button("In dialog").node_ref(anchor),
                            show(popup_on, move || {
                                let popup_count = popup_count.clone();
                                overlay(anchor, Placement::Below, text("popup"))
                                    .role(Role::Menu)
                                    .name("Popup")
                                    .on_dismiss(move |_| {
                                        popup_count.set(popup_count.get() + 1);
                                        popup_on.set(false);
                                    })
                            }),
                        )),
                    )
                    .modal(true)
                    .role(Role::Dialog)
                    .name("Dialog")
                    .on_dismiss(move |_| {
                        dialog_count.set(dialog_count.get() + 1);
                        dialog_on.set(false);
                    })
                }),
                show(late_on, move || {
                    let late_count = late_count.clone();
                    overlay(node_ref(), Placement::Center, text("late"))
                        .role(Role::Menu)
                        .name("Late")
                        .on_dismiss(move |_| {
                            late_count.set(late_count.get() + 1);
                            late_on.set(false);
                        })
                }),
            )),
        )
    });
    (screen, signals, counts)
}

#[test]
fn escape_in_a_modal_dismisses_the_innermost_overlay_in_it_not_one_outside() {
    let (screen, _signals, [dialog, popup, late]) = escape_screen();
    screen.get_by_role(Role::Button, "In dialog").focus();
    assert_eq!(focused_name(&screen).as_deref(), Some("In dialog"));
    // The page's popup comes last in tree order but lies outside the modal.
    screen.press_key(Key::Escape);
    assert_eq!(
        (dialog.get(), popup.get(), late.get()),
        (0, 1, 0),
        "the popup inside the dialog goes first"
    );
    screen.press_key(Key::Escape);
    assert_eq!(
        (dialog.get(), popup.get(), late.get()),
        (1, 1, 0),
        "then the dialog itself, never the popup outside it"
    );
    screen.press_key(Key::Escape);
    assert_eq!(
        (dialog.get(), popup.get(), late.get()),
        (1, 1, 1),
        "with the modal gone the remaining overlay is the topmost"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_modal_that_unmounts_while_another_stays_open_returns_the_focus_nowhere_outside_it() {
    let (screen, (first_on, second_on)) = mount(|| {
        let (first_on, second_on) = (signal(false), signal(false));
        (
            (first_on, second_on),
            column((
                button("Page"),
                show(first_on, move || {
                    overlay(
                        node_ref(),
                        Placement::Center,
                        button("In first").on_press(move || second_on.set(true)),
                    )
                    .modal(true)
                    .role(Role::Dialog)
                    .name("First")
                }),
                // No tab stop: the focus has nowhere to go while it is open.
                show(second_on, move || {
                    overlay(node_ref(), Placement::Center, text("second"))
                        .modal(true)
                        .role(Role::Dialog)
                        .name("Second")
                }),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Page").focus();
    first_on.set(true);
    screen.run_until_idle();
    assert_eq!(focused_name(&screen).as_deref(), Some("In first"));
    screen.get_by_role(Role::Button, "In first").click();
    assert!(screen.query_by_role(Role::Dialog, "Second").is_some());
    assert!(screen.focused().is_none(), "the second modal has no stop");
    // The first modal's opener is the page button, outside the second modal
    // that stays open: it does not get the focus back.
    first_on.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Dialog, "First").is_none());
    assert!(
        screen.focused().is_none(),
        "the focus stays out of the page behind the modal: {:?}",
        focused_name(&screen)
    );
    second_on.set(false);
    screen.run_until_idle();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_modal_that_is_leaving_no_longer_blocks_the_page_and_gives_the_focus_back() {
    use kivact::motion::{Easing, Transition};
    use kivact::style::TransitionProp;
    let presses = Rc::new(Cell::new(0));
    let (screen, open) = mount({
        let presses = presses.clone();
        move || {
            let open = signal(false);
            (
                open,
                column((
                    button("Opener"),
                    button("Behind").on_press(move || presses.set(presses.get() + 1)),
                    show(open, || {
                        overlay(node_ref(), Placement::Center, button("In modal"))
                            .modal(true)
                            .role(Role::Dialog)
                            .name("Modal")
                            .style(style().exit(style().opacity(0.0)).transition(
                                TransitionProp::Paint,
                                Transition::tween(100, Easing::Linear),
                            ))
                    }),
                )),
            )
        }
    });
    screen.get_by_role(Role::Button, "Opener").focus();
    open.set(true);
    screen.run_until_idle();
    assert_eq!(focused_name(&screen).as_deref(), Some("In modal"));
    screen.get_by_role(Role::Button, "Behind").force_click();
    assert_eq!(presses.get(), 0, "the open modal blocks the page");
    assert!(screen.query_by_role(Role::Dialog, "Modal").is_some());
    open.set(false);
    screen.run_until_idle();
    // Drawn until its exit ends, but taking no input and confining nothing.
    assert!(screen.query_by_role(Role::Dialog, "Modal").is_none());
    assert_eq!(focused_name(&screen).as_deref(), Some("Opener"));
    screen.get_by_role(Role::Button, "Behind").click();
    assert_eq!(presses.get(), 1, "a leaving modal does not block the page");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
