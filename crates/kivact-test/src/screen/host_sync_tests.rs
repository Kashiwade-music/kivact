//! Role: Host synchronization tests.
//!
use std::cell::Cell;
use std::rc::Rc;

use kivact_types::{NodeId, WindowId};

use super::model::Screen;
use super::support::button_box;
use super::support::{Rng, seeds};
use crate::render;
use kivact_core::state::Signal;
use kivact_core::style::style;
use kivact_core::view::{Role, column, for_each, node, text};
use kivact_runtime::signal;

#[test]
fn rejected_pointer_down_cannot_retain_an_ended_region_gesture() {
    use kivact_core::custom::part;
    use kivact_core::view::canvas;
    use kivact_core::view::{ButtonPress, Handled, MouseButton, PointerPhase};
    use kivact_engine::contract::HostEvent;
    use kivact_style::draw::{Rect, RegionId};
    // xtask-allow(state-fields): statistic: a test counter of presses
    let presses = Rc::new(Cell::new(0));
    let calls = presses.clone();
    // xtask-allow(state-fields): statistic: a test counter of observed downs
    let downs = Rc::new(Cell::new(0));
    let observed = downs.clone();
    // xtask-allow(state-fields): independent: a test slot holding the disabled signal; written once
    let disabled = Rc::new(Cell::new(None));
    let disabled_slot = disabled.clone();
    let screen = render(move || {
        // xtask-allow(state-fields): independent: a test signal; the test observes it on its own
        let disabled = signal(false);
        disabled_slot.set(Some(disabled));
        canvas(|draw, _| draw.region(RegionId(1), Rect::new(0., 0., 100., 60.)))
            .role(Role::Canvas)
            .name("gesture canvas")
            .style(style().size(120, 80))
            .part(kivact_core::custom::Part::new(
                kivact_core::state::memo(|| ()),
                move |node| {
                    node.part(part(|state| state))
                        .region_events()
                        .on_pointer(move |event| {
                            if matches!(event.phase(), PointerPhase::Down(_)) {
                                observed.set(observed.get() + 1);
                            }
                            Handled::Yes
                        })
                },
            ))
            .region(
                RegionId(1),
                kivact_core::custom::Part::new(kivact_core::state::memo(|| ()), move |node| {
                    node.part(part(|state| state))
                        .role(Role::Button)
                        .name("action")
                        .disabled(disabled)
                        .on_press(move || calls.set(calls.get() + 1))
                }),
            )
    });
    let node = screen.get_by_role(Role::Canvas, "gesture canvas").node;
    let press = ButtonPress {
        button: MouseButton::Primary,
        clicks: 1,
    };
    let sample = |id, phase| gesture_sample(node, id, phase);
    let host = &screen.app.host;
    let dispatch = |id, phase| {
        host.borrow_mut()
            .inject(HostEvent::Pointer(sample(id, phase)))
    };
    let (down, up) = (PointerPhase::Down(press), PointerPhase::Up(press));
    dispatch(11, down);
    dispatch(22, down);
    assert_eq!(downs.get(), 2, "each pointer has a capture of its own");
    dispatch(11, up);
    dispatch(22, up);
    host.borrow_mut().inject(HostEvent::Activate {
        node,
        region: Some(kivact_types::RegionId(1)),
        pointer: Some(sample(22, up)),
    });
    screen.run_until_idle();
    assert_eq!(presses.get(), 1, "the next pointer activation is not stale");
    dispatch(33, down);
    let disabled = disabled.get().expect("mounted signal");
    disabled.set(true);
    screen.run_until_idle();
    disabled.set(false);
    screen.run_until_idle();
    host.borrow_mut().inject(HostEvent::Activate {
        node,
        region: Some(kivact_types::RegionId(1)),
        pointer: Some(sample(33, up)),
    });
    assert_eq!(presses.get(), 2, "registry cancellation clears its gesture");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_primary_down_of_the_gesture_pointer_releases_a_gesture_whose_up_was_lost() {
    use kivact_core::custom::part;
    use kivact_core::view::canvas;
    use kivact_core::view::{ButtonPress, Handled, MouseButton, PointerKind, PointerPhase};
    use kivact_engine::contract::HostEvent;
    use kivact_style::draw::{Point, Rect, RegionId};
    use kivact_types::{Modifiers, PointerSample};
    // xtask-allow(state-fields): statistic: a test counter of presses
    let presses = Rc::new(Cell::new(0));
    let calls = presses.clone();
    // xtask-allow(state-fields): statistic: a test counter of observed downs
    let downs = Rc::new(Cell::new(0));
    let observed = downs.clone();
    let screen = render(move || {
        canvas(|draw, _| draw.region(RegionId(1), Rect::new(0., 0., 100., 60.)))
            .role(Role::Canvas)
            .name("gesture canvas")
            .style(style().size(120, 80))
            .part(kivact_core::custom::Part::new(
                kivact_core::state::memo(|| ()),
                move |node| {
                    node.part(part(|state| state))
                        .region_events()
                        .on_pointer(move |event| {
                            if !matches!(event.phase(), PointerPhase::Down(_)) {
                                return Handled::No;
                            }
                            observed.set(observed.get() + 1);
                            // Only the first press starts a gesture.
                            if observed.get() == 1 {
                                Handled::Yes
                            } else {
                                Handled::No
                            }
                        })
                },
            ))
            .region(
                RegionId(1),
                kivact_core::custom::Part::new(kivact_core::state::memo(|| ()), move |node| {
                    node.part(part(|state| state))
                        .role(Role::Button)
                        .name("action")
                        .on_press(move || calls.set(calls.get() + 1))
                }),
            )
    });
    let node = screen.get_by_role(Role::Canvas, "gesture canvas").node;
    let press = ButtonPress {
        button: MouseButton::Primary,
        clicks: 1,
    };
    // The mouse (pointer 1) starts a gesture whose Up never arrives.
    screen
        .app
        .host
        .borrow_mut()
        .inject(HostEvent::Pointer(kivact_engine::contract::Routed {
            window: WindowId::MAIN,
            target: Some(node),
            local: Point::new(20., 20.),
            captured: false,
            activation: None,
            region: Some(kivact_types::RegionId(1)),
            input: PointerSample {
                id: kivact_types::PointerId(1),
                phase: PointerPhase::Down(press),
                position: Point::new(20., 20.),
                buttons: kivact_types::PointerButtons::NONE.with(MouseButton::Primary, true),
                modifiers: Modifiers::default(),
                kind: PointerKind::Mouse,
                button: Some(press.button),
                press: Some(press),
                time_ns: 0,
                pressure: None,
                tilt: None,
                is_primary: true,
            },
        }));
    screen.run_until_idle();
    assert_eq!(downs.get(), 1);
    // Its next click is an ordinary press that the gesture does not decline.
    screen.get_by_role(Role::Button, "action").click();
    assert_eq!(downs.get(), 2, "the new press reaches the canvas behavior");
    assert_eq!(presses.get(), 1, "the lost gesture does not decline it");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// Input and host diagnostics in the same turn.

type Draft = (Signal<String>, Signal<Vec<String>>);

fn draft_screen() -> (Screen, Draft, NodeId, NodeId) {
    let slot: Rc<Cell<Option<Draft>>> = Rc::new(Cell::new(None));
    let s2 = slot.clone();
    let screen = render(move || {
        let draft = signal(String::new());
        let list = signal(Vec::<String>::new());
        s2.set(Some((draft, list)));
        column((
            super::support::editor("draft", draft),
            button_box("add").on_press(move || {
                let d = draft.get();
                list.update(|l| l.push(d));
                draft.set(String::new());
            }),
            for_each(
                move || list.get().into_iter().enumerate().collect::<Vec<_>>(),
                |it: &(usize, String)| it.0,
                |v| text(move || v.get().1),
            ),
        ))
    });
    let d = slot.get().expect("setup");
    let field = screen.get_by_role(Role::TextBox, "draft").node;
    let add = screen.get_by_role(Role::Button, "add").node;
    screen.get_by_role(Role::TextBox, "draft").focus();
    (screen, d, field, add)
}

/// `typing_after_click`: allow keystrokes after a click in the same turn
/// (host edit after a programmatic reset); `false` keeps the click last.
fn run_input_with_host_failure(typing_after_click: bool) {
    for seed in seeds() {
        let mut rng = Rng::new(seed);
        let (screen, (draft, list), field, add) = draft_screen();
        let mut expected_list = 0usize;
        for step in 0..40 {
            let ctx = format!("seed {seed} step {step}");
            {
                let n = 1 + rng.below(3);
                let mut clicked = false;
                for _ in 0..n {
                    match rng.below(5) {
                        0 | 1 if typing_after_click || !clicked => {
                            let c = (b'a' + rng.below(26) as u8) as char;
                            // A press on the field focuses it (after a
                            // press on "add" took the focus).
                            screen.queue_click(field);
                            screen.queue_text(&c.to_string());
                        }
                        2 => {
                            screen.queue_click(add);
                            expected_list += 1;
                            clicked = true;
                        }
                        4 => screen.queue_host_failure("probe"),
                        _ => {}
                    }
                }
            }
            screen.run_until_idle();
            let host_value = screen.get_by_role(Role::TextBox, "draft").value();
            assert_eq!(
                host_value,
                Some(draft.get_untracked()),
                "{ctx}: binding vs host"
            );
            assert_eq!(list.get_untracked().len(), expected_list, "{ctx}");
        }
        assert!(
            screen.errors().is_empty(),
            "seed {seed}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn input_with_host_failure_keeps_host_core_and_binding_in_sync() {
    run_input_with_host_failure(false);
}

#[test]
fn input_with_host_failure_and_typing_after_a_click() {
    run_input_with_host_failure(true);
}

#[test]
fn host_edit_after_a_programmatic_reset_in_the_same_turn_is_kept() {
    let (screen, (draft, _list), field, add) = draft_screen();
    screen.queue_text("j");
    screen.queue_click(add); // app clears the draft
    screen.queue_click(field);
    screen.queue_text("i"); // the user keeps typing before the turn runs
    screen.run_until_idle();
    let host_value = screen.get_by_role(Role::TextBox, "draft").value();
    // Either the reset wins on both sides or the keystroke wins on both
    // sides; the host and the binding must not disagree.
    assert_eq!(
        host_value,
        Some(draft.get_untracked()),
        "host and binding diverged"
    );
}

/// A host edit that drops the only prop of a pending `SetProps` (a
/// programmatic reset the keystroke overrides) drops the write too: no
/// empty `SetProps`, and no revision for nothing.
#[test]
fn a_host_edit_that_overrides_the_only_pending_prop_sends_nothing() {
    let slot: Rc<Cell<Option<Signal<String>>>> = Rc::new(Cell::new(None));
    let s2 = slot.clone();
    let screen = render(move || {
        let draft = signal(String::new());
        s2.set(Some(draft));
        column((
            super::support::editor("draft", draft),
            button_box("reset").on_press(move || draft.set(String::new())),
        ))
    });
    let draft = slot.get().expect("setup");
    let field = screen.get_by_role(Role::TextBox, "draft").node;
    let reset = screen.get_by_role(Role::Button, "reset").node;
    screen.get_by_role(Role::TextBox, "draft").type_text("j");
    screen.get_by_role(Role::Button, "reset").focus();
    let before = screen.stats();
    // A reset of the value and a keystroke that overrides it, in one turn.
    screen.queue_click(reset);
    screen.queue_click(field);
    screen.queue_text("i");
    screen.run_until_idle();
    let after = screen.stats();
    assert_eq!(draft.get_untracked(), "ji");
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "draft")
            .value()
            .as_deref(),
        Some("ji")
    );
    assert_eq!(
        (after.batches, after.ops),
        (before.batches, before.ops),
        "nothing to write"
    );
}

// Public TextEditing configures editing only at mount: it cannot remove editing
// from a live node. Exercise that case through the existing core Screen and the
// normal engine Writer boundary, while mounting/highlighting through public APIs.
#[test]
fn replacing_editing_drops_line_state_after_releasing_the_model() {
    use super::editing_drop_probe::{Probe, State};
    use kivact_core::custom::{TextEditing, part};
    use std::cell::RefCell;
    use std::rc::Weak;

    // xtask-allow(state-fields): statistic: a test counter of drops
    let drops = Rc::new(Cell::new(0));
    // xtask-allow(state-fields): statistic: a test counter of re-entered drops
    let reentered = Rc::new(Cell::new(0));
    // xtask-allow(state-fields): independent: a test slot holding the reader; written once
    let reader = Rc::new(RefCell::new(None));
    let probes: Rc<RefCell<Vec<Weak<Probe>>>> = Rc::default();
    let (d, e, read, seen) = (
        drops.clone(),
        reentered.clone(),
        reader.clone(),
        probes.clone(),
    );
    let screen = render(move || {
        let editing =
            TextEditing::new("one line").line_styles((), move |_: &(), _: &str, _: &State| {
                let probe = Rc::new(Probe {
                    armed: Cell::new(false),
                    drops: d.clone(),
                    reentered: e.clone(),
                    reader: read.clone(),
                });
                seen.borrow_mut().push(Rc::downgrade(&probe));
                (Vec::new(), State(Some(probe)))
            });
        column((
            node()
                .style(style().size(300, 80))
                .part(kivact_core::custom::Part::new(
                    kivact_core::state::memo(|| ()),
                    move |node| {
                        node.part(part(|state| state))
                            .role(Role::TextBox)
                            .name("editor")
                            .text_editing(editing)
                    },
                )),
            node().test_id("independent").style(style().size(10, 10)),
        ))
    });
    screen.screenshot();
    let editor = screen.get_by_role(Role::TextBox, "editor").node;
    let independent = screen.get_by_test_id("independent").node;
    let writer = screen.with_host(|host| host.writer());
    *reader.borrow_mut() = Some((writer.reader(), independent));
    let mut live = 0;
    for weak in probes.borrow().iter() {
        if let Some(probe) = weak.upgrade() {
            probe.armed.set(true);
            live += 1;
        }
    }
    assert_eq!(live, 1, "one live line; disposable oracle states are gone");
    assert!(writer.set_editing(editor, None));
    // No commit/idle pump: destruction belongs to the setter boundary.
    assert_eq!(drops.get(), 1);
    assert_eq!(reentered.get(), 1);
    assert!(!writer.set_editing(editor, None));
    assert_eq!(drops.get(), 1, "an equal write does not drop again");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn gesture_sample(
    node: NodeId,
    id: u64,
    phase: kivact_types::PointerPhase,
) -> kivact_engine::contract::Routed<kivact_types::PointerSample> {
    use kivact_style::draw::Point;
    use kivact_types::{Modifiers, MouseButton, PointerKind, PointerPhase, PointerSample};
    let is_primary = id != 22;
    kivact_engine::contract::Routed {
        window: WindowId::MAIN,
        target: Some(node),
        local: Point::new(20., 20.),
        captured: false,
        activation: None,
        region: Some(kivact_types::RegionId(1)),
        input: PointerSample {
            id: kivact_types::PointerId(id),
            phase,
            position: Point::new(20., 20.),
            buttons: kivact_types::PointerButtons::NONE
                .with(MouseButton::Primary, matches!(phase, PointerPhase::Down(_))),
            modifiers: Modifiers::default(),
            kind: PointerKind::Touch,
            button: match phase {
                PointerPhase::Down(press) | PointerPhase::Up(press) => Some(press.button),
                _ => None,
            },
            press: match phase {
                PointerPhase::Down(press) | PointerPhase::Up(press) => Some(press),
                _ => None,
            },
            time_ns: 0,
            pressure: None,
            tilt: None,
            is_primary,
        },
    }
}
