//! Role: Node model transactions, field merges and core navigation notifications.
use super::*;
use kivact::interaction::ViewTransform;
use kivact::ui::EditPhase;
use kivact::ui::EditPhase::{Begin, Cancel, Commit};
use kivact::view::{Handled, PointerEvent, PointerKind, PointerPhase, callback};
pub(super) fn nodes() -> Vec<ui::NodeData> {
    vec![
        ui::NodeData::new(
            1,
            "Source",
            vec![ui::NodeSocket::new(
                "In",
                "Scalar",
                kivact::style::rgb(0x9b9ea8),
            )],
            vec![ui::NodeSocket::new(
                "Out",
                "Scalar",
                kivact::style::rgb(0x9b9ea8),
            )],
        )
        .position(Point::new(20.0, 20.0))
        .size(Size::new(180.0, 180.0))
        .controls(vec![
            ui::NodeControl::number("Amount", 75.0, 2.0, 0.0, 10.0, 0.1),
            ui::NodeControl::choice("Mode", 100.0, ["Linear", "Nearest"], 0),
            ui::NodeControl::toggle("Enabled", 125.0, false),
        ]),
        ui::NodeData::new(
            2,
            "Sink",
            vec![ui::NodeSocket::new(
                "In",
                "Scalar",
                kivact::style::rgb(0x9b9ea8),
            )],
            vec![],
        )
        .position(Point::new(320.0, 30.0))
        .size(Size::new(180.0, 160.0)),
    ]
}
fn number(nodes: &[ui::NodeData]) -> f64 {
    match nodes[0].controls[0].value {
        ui::NodeControlValue::Number { value, .. } => value,
        _ => panic!("number"),
    }
}
fn start(screen: &Screen, name: &str, dx: f32, dy: f32) {
    let p = screen
        .get_by_role(Role::Button, name)
        .rect()
        .expect("invariant: fixture node header has geometry");
    let point = Point::new(p.x + 50.0, p.y + 12.0);
    screen.mouse_move(point.x, point.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(point.x + dx, point.y + dy);
}

#[test]
fn node_item_drags_notify_once_and_cancel_on_every_departure() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (data, selected, shown, disabled, hidden, ancestor)) =
        kivact::test::render_with_handle(crate::options(), move || {
            let data = signal(nodes());
            let selected = signal(vec![1, 2]);
            let shown = signal(true);
            let disabled = signal(false);
            let hidden = signal(false);
            let ancestor = signal(false);
            let observer =
                callback(move |phase| seen.borrow_mut().push((phase, data.get_untracked())));
            (
                (data, selected, shown, disabled, hidden, ancestor),
                column((
                    column(show(shown, move || {
                        app_node_editor(
                            data,
                            signal(vec![]),
                            ui::NodeState {
                                selection: selected.into(),
                                ..ui::NodeState::new()
                            },
                        )
                        .on_edit(move |value| observer.try_call(value))
                        .disabled(disabled)
                        .hidden(hidden)
                        .name("Nodes")
                        .style(style().size(620, 240))
                    }))
                    .hidden(ancestor),
                    kivact::ui::button("Other"),
                )),
            )
        });
    for mode in 0..8 {
        log.borrow_mut().clear();
        let original = nodes();
        data.set(original.clone());
        selected.set(vec![1, 2]);
        screen.run_until_idle();
        let contact = if mode == 2 {
            let r = screen.get_by_role(Role::Button, "Source").rect().unwrap();
            let contact = screen.contact_down(PointerKind::Touch, r.x + 50.0, r.y + 12.0);
            screen.contact_move(contact, r.x + 80.0, r.y + 21.0);
            Some(contact)
        } else {
            start(&screen, "Source", 30.0, 9.0);
            None
        };
        assert_ne!(
            data.get_untracked()[0].position,
            original[0].position,
            "mode {mode}"
        );
        let moved = data.get_untracked();
        screen
            .get_by_role(Role::SpinButton, "Source Amount")
            .a11y_with_value(A11yAction::SetValue, "8");
        assert_eq!(data.get_untracked(), moved);
        match mode {
            0 => screen.mouse_up(MouseButton::Primary),
            1 => screen.press_key(Key::Escape),
            2 => screen.contact_cancel(contact.expect("the touch is down")),
            3 => {
                disabled.set(true);
                screen.run_until_idle();
            }
            4 => {
                hidden.set(true);
                screen.run_until_idle();
            }
            5 => {
                shown.set(false);
                screen.run_until_idle();
            }
            6 => {
                for _ in 0..16 {
                    if screen.get_by_role(Role::Button, "Other").is_focused() {
                        break;
                    }
                    screen.press_key(Key::Tab);
                }
                assert!(screen.get_by_role(Role::Button, "Other").is_focused());
            }
            _ => {
                ancestor.set(true);
                screen.run_until_idle();
            }
        }
        let terminal = if mode == 0 { Commit } else { Cancel };

        if mode > 0 {
            assert_eq!(data.get_untracked(), original, "mode {mode}");
        } else {
            assert_eq!(
                data.get_untracked()[1].position,
                Point::new(original[1].position.x + 30.0, original[1].position.y + 9.0)
            );
        }
        assert_eq!(log.borrow().last().unwrap().1, data.get_untracked());
        if mode != 2 {
            screen.mouse_up(MouseButton::Primary);
        }
        disabled.set(false);
        hidden.set(false);
        ancestor.set(false);
        shown.set(true);
        screen.run_until_idle();
        screen.advance(Duration::from_secs(1));
        assert_eq!(
            log.borrow().iter().map(|p| p.0).collect::<Vec<_>>(),
            [Begin, terminal],
            "mode {mode}"
        );
    }
    clean(&screen);
}

#[test]
fn application_cancel_restores_its_node_snapshot() {
    let (screen, (data, selected)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(nodes());
        let selected = signal(vec![1, 2]);
        (
            (data, selected),
            app_node_editor(
                data,
                signal(vec![]),
                ui::NodeState {
                    selection: selected.into(),
                    ..ui::NodeState::new()
                },
            )
            .name("Nodes")
            .style(style().size(620, 240)),
        )
    });
    start(&screen, "Source", 30.0, 10.0);
    data.update(|nodes| {
        nodes[0].label = "External label".into();
        nodes[0].color = rgb(0xabcdef);
        nodes.retain(|n| n.id != 2);
        nodes.push(ui::NodeData::new(3, "External", vec![], vec![]));
    });
    screen.run_until_idle();
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    let current = data.get_untracked();
    assert_eq!(current[0].position, nodes()[0].position);
    assert_eq!(current[0].label, "Source");
    assert_eq!(current[0].color, nodes()[0].color);
    assert!(current.iter().any(|n| n.id == 2));
    assert!(!current.iter().any(|n| n.id == 3));
    data.set(nodes());
    selected.set(vec![]);
    screen.run_until_idle();
    screen.advance(Duration::from_secs(1));
    let p = screen
        .get_by_role(Role::SpinButton, "Source Amount")
        .rect()
        .unwrap()
        .center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(p.x + 20.0, p.y);
    assert!((number(&data.get_untracked()) - 4.0).abs() < 0.001);
    data.update(|nodes| nodes[0].label = "Renamed".into());
    screen.run_until_idle();
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(number(&data.get_untracked()), 2.0);
    assert_eq!(data.get_untracked()[0].label, "Source");
    clean(&screen);
}

#[test]
fn node_control_drags_use_total_delta_and_cancel_after_slot_retirement() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (data, shown, disabled, hidden)) =
        kivact::test::render_with_handle(crate::options(), move || {
            let data = signal(nodes());
            let shown = signal(true);
            let disabled = signal(false);
            let hidden = signal(false);
            let observer =
                callback(move |phase| seen.borrow_mut().push((phase, data.get_untracked())));
            (
                (data, shown, disabled, hidden),
                show(shown, move || {
                    app_node_editor(data, signal(vec![]), ui::NodeState::new())
                        .on_edit(move |value| observer.try_call(value))
                        .disabled(disabled)
                        .hidden(hidden)
                        .name("Nodes")
                        .style(style().size(620, 240))
                }),
            )
        });
    for mode in 0..6 {
        data.set(nodes());
        screen.run_until_idle();
        let p = screen
            .get_by_role(Role::SpinButton, "Source Amount")
            .rect()
            .unwrap()
            .center();
        let contact = if mode == 2 {
            let contact = screen.contact_down(PointerKind::Touch, p.x, p.y);
            screen.contact_move(contact, p.x + 10.0, p.y);
            screen.contact_move(contact, p.x + 20.0, p.y);
            Some(contact)
        } else {
            screen.mouse_move(p.x, p.y);
            screen.mouse_down(MouseButton::Primary);
            screen.mouse_move(p.x + 10.0, p.y);
            screen.mouse_move(p.x + 20.0, p.y);
            None
        };
        assert!((number(&data.get_untracked()) - 4.0).abs() < 0.001);
        assert_eq!(log.borrow().last().unwrap().0, Begin);
        match mode {
            0 => screen.mouse_up(MouseButton::Primary),
            1 => screen.press_key(Key::Escape),
            2 => screen.contact_cancel(contact.expect("the touch is down")),
            3 => {
                disabled.set(true);
                screen.run_until_idle();
            }
            4 => {
                hidden.set(true);
                screen.run_until_idle();
            }
            _ => {
                shown.set(false);
                screen.run_until_idle();
            }
        }
        assert_eq!(
            log.borrow().last().unwrap().0,
            if mode == 0 { Commit } else { Cancel }
        );
        assert_eq!(
            number(&data.get_untracked()),
            if mode == 0 { 4.0 } else { 2.0 }
        );
        assert_eq!(log.borrow().last().unwrap().1, data.get_untracked());
        if mode != 2 {
            screen.mouse_up(MouseButton::Primary);
        }
        disabled.set(false);
        hidden.set(false);
        shown.set(true);
        screen.run_until_idle();
        screen.advance(Duration::from_secs(1));
    }
    data.set(nodes());
    screen.run_until_idle();
    let p = screen
        .get_by_role(Role::SpinButton, "Source Amount")
        .rect()
        .unwrap()
        .center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(p.x + 20.0, p.y);
    data.update(|nodes| nodes[0].controls.swap(0, 1));
    screen.run_until_idle();
    assert_eq!(log.borrow().last().unwrap().0, Cancel);
    assert!(matches!(
        data.get_untracked()[0].controls[0].value,
        ui::NodeControlValue::Number { .. }
    ));
    let current = data.get_untracked();
    screen.mouse_move(p.x + 40.0, p.y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(data.get_untracked(), current);
    clean(&screen);
}

#[test]
fn node_release_applies_last_position_when_move_was_consumed_and_snap_is_relative() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, data) = kivact::test::render_with_handle(crate::options(), move || {
        let data = signal(nodes());
        (
            data,
            app_node_editor(
                data,
                signal(vec![]),
                ui::NodeState {
                    selection: (signal(vec![1, 2])).into(),
                    ..ui::NodeState::new()
                },
            )
            .snap(true)
            .on_edit(move |p| seen.borrow_mut().push(p))
            .style(style().size(620, 240))
            .on_pointer(|event: PointerEvent| {
                if event.phase == PointerPhase::Move {
                    Handled::Yes
                } else {
                    Handled::No
                }
            }),
        )
    });
    start(&screen, "Source", 17.0, 9.0);
    assert_eq!(data.get_untracked(), nodes());
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(data.get_untracked()[0].position, Point::new(40.0, 29.0));
    assert_eq!(data.get_untracked()[1].position, Point::new(340.0, 39.0));
    assert_eq!(*log.borrow(), [Begin, Commit]);
    clean(&screen);
}

#[test]
fn node_atomic_controls_keys_and_link_commands_have_one_pair_selection_has_none() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (data, links)) = kivact::test::render_with_handle(crate::options(), move || {
        let data = signal(nodes());
        let links = signal(vec![]);
        (
            (data, links),
            app_node_editor(data, links, ui::NodeState::new())
                .on_edit(move |p| seen.borrow_mut().push(p))
                .name("Nodes")
                .style(style().size(620, 240)),
        )
    });
    screen
        .get_by_role(Role::Canvas, "Nodes")
        .drag(Point::new(5.0, 5.0), Point::new(205.0, 210.0));
    assert!(log.borrow().is_empty());
    screen
        .get_by_role(Role::Button, "Source")
        .a11y(A11yAction::Select);
    screen.get_by_role(Role::Button, "Source").click();
    assert!(log.borrow().is_empty());
    screen
        .get_by_role(Role::Canvas, "Nodes")
        .press_key(Key::ArrowRight);
    assert_eq!(data.get_untracked()[0].position.x, 21.0);
    let amount = screen.get_by_role(Role::SpinButton, "Source Amount");
    amount.a11y_with_value(A11yAction::SetValue, "5");
    amount.a11y(A11yAction::Increment);
    amount.a11y(A11yAction::Decrement);
    assert_eq!(number(&data.get_untracked()), 5.0);
    screen.get_by_role(Role::Button, "Source Enabled").click();
    assert_eq!(
        data.get_untracked()[0].controls[2].value,
        ui::NodeControlValue::Toggle(true)
    );
    screen
        .get_by_role(Role::Button, "Source Enabled")
        .a11y(A11yAction::Activate);
    assert_eq!(
        data.get_untracked()[0].controls[2].value,
        ui::NodeControlValue::Toggle(false)
    );
    screen
        .get_by_role(Role::Button, "Source Mode")
        .a11y(A11yAction::Activate);
    screen.get_by_role(Role::Option, "Nearest").click();
    assert!(matches!(
        data.get_untracked()[0].controls[1].value,
        ui::NodeControlValue::Choice { selected: 1, .. }
    ));
    screen
        .get_by_role(Role::Button, "Source output Out")
        .a11y(A11yAction::Activate);
    let before = log.borrow().len();
    screen
        .get_by_role(Role::Button, "Sink input In")
        .a11y(A11yAction::Activate);
    assert_eq!(log.borrow().len(), before + 2);
    assert_eq!(links.get_untracked(), [ui::NodeLink::new(1, 0, 2, 0)]);
    screen
        .get_by_role(Role::Button, "Link Source output 0 to Sink input 0")
        .a11y(A11yAction::Select);
    let before = log.borrow().len();
    screen
        .get_by_role(Role::Canvas, "Nodes")
        .press_key(Key::Delete);
    assert!(links.get_untracked().is_empty());
    assert_eq!(log.borrow().len(), before + 2);
    screen
        .get_by_role(Role::Button, "Source output Out")
        .press_key(Key::Enter);
    screen
        .get_by_role(Role::Button, "Sink input In")
        .press_key(Key::Enter);
    assert_eq!(links.get_untracked(), [ui::NodeLink::new(1, 0, 2, 0)]);
    assert_eq!(log.borrow().len(), before + 4);
    screen
        .get_by_role(Role::Button, "Source")
        .a11y(A11yAction::Select);
    screen
        .get_by_role(Role::Canvas, "Nodes")
        .press_key(Key::Char('m'));
    assert!(data.get_untracked()[0].muted);
    screen
        .get_by_role(Role::Canvas, "Nodes")
        .press_key(Key::Delete);
    assert!(data.get_untracked().iter().all(|node| node.id != 1));
    assert!(links.get_untracked().is_empty());
    assert_eq!(log.borrow().len(), before + 8);
    assert!(
        log.borrow()
            .as_chunks::<2>()
            .0
            .iter()
            .all(|p| *p == [Begin, Commit])
    );
    clean(&screen);
}

#[test]
fn node_link_drag_is_preview_until_release_and_escape_never_mutates_model() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, links) = kivact::test::render_with_handle(crate::options(), move || {
        let links = signal(vec![]);
        (
            links,
            app_node_editor(signal(nodes()), links, ui::NodeState::new())
                .on_edit(move |p| seen.borrow_mut().push(p))
                .name("Nodes")
                .style(style().size(620, 240)),
        )
    });
    let output = screen
        .get_by_role(Role::Button, "Source output Out")
        .rect()
        .unwrap()
        .center();
    let input = screen
        .get_by_role(Role::Button, "Sink input In")
        .rect()
        .unwrap()
        .center();
    screen.mouse_move(output.x, output.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(input.x, input.y);
    assert!(links.get_untracked().is_empty());
    assert!(log.borrow().is_empty());
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(links.get_untracked(), [ui::NodeLink::new(1, 0, 2, 0)]);
    assert_eq!(*log.borrow(), [Begin, Commit]);
    screen.mouse_move(input.x, input.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(input.x + 60.0, input.y + 60.0);
    assert_eq!(links.get_untracked().len(), 1);
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(links.get_untracked().len(), 1);
    assert_eq!(log.borrow().len(), 2);
    clean(&screen);
}

#[test]
fn node_popup_numeric_phases_and_slot_removal_close_without_writing_another_control() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, data) = kivact::test::render_with_handle(crate::options(), move || {
        let data = signal(nodes());
        (
            data,
            app_node_editor(data, signal(vec![]), ui::NodeState::new())
                .on_edit(move |p| seen.borrow_mut().push(p))
                .name("Nodes")
                .style(style().size(620, 240)),
        )
    });
    screen
        .get_by_role(Role::SpinButton, "Source Amount")
        .double_click();
    let number = screen.get_by_role(Role::SpinButton, "Amount");
    number.a11y_with_value(A11yAction::SetValue, "6");
    assert_eq!(*log.borrow(), [Begin, Commit]);
    data.update(|nodes| {
        nodes[0].controls.remove(0);
    });
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::SpinButton, "Amount").is_none());
    assert!(matches!(
        data.get_untracked()[0].controls[0].value,
        ui::NodeControlValue::Choice { .. }
    ));
    clean(&screen);
}

#[test]
fn node_core_pan_zoom_fit_forward_phases_after_negative_origin_correction() {
    let log = Rc::new(RefCell::new(Vec::<(EditPhase, ViewTransform)>::new()));
    let seen = log.clone();
    let (screen, (transform, data)) =
        kivact::test::render_with_handle(crate::options(), move || {
            let mut n = nodes();
            n[0].position = Point::new(-100.0, -50.0);
            let data = signal(n);
            let transform = signal(ViewTransform::default());
            let observer =
                callback(move |phase| seen.borrow_mut().push((phase, transform.get_untracked())));
            (
                (transform, data),
                column((
                    app_node_editor(
                        data,
                        signal(vec![]),
                        ui::NodeState {
                            transform: transform.into(),
                            ..ui::NodeState::new()
                        },
                    )
                    .on_edit(move |value| observer.try_call(value))
                    .name("Nodes")
                    .style(style().size(620, 240)),
                    kivact::ui::button("Other"),
                )),
            )
        });
    let canvas = screen.get_by_role(Role::Canvas, "Nodes");
    canvas.press_key(Key::Char('0'));
    let fit = transform.get_untracked();
    assert_eq!(log.borrow().len(), 2);
    assert_eq!(log.borrow()[1], (Commit, fit));
    let origin = fit.to_view(Point::new(-100.0, -50.0));
    assert!(origin.x >= 0.0 && origin.y >= 0.0);
    canvas.press_key(Key::Char('+'));
    assert_eq!(log.borrow().len(), 4);
    let rect = canvas.rect().unwrap();
    screen.mouse_move(rect.center().x, rect.y + 5.0);
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_move(rect.center().x + 20.0, rect.y + 15.0);
    let before = data.get_untracked();
    screen
        .get_by_role(Role::Button, "Source")
        .a11y(A11yAction::Select);
    assert_eq!(data.get_untracked(), before);
    screen.mouse_up(MouseButton::Middle);
    assert_eq!(log.borrow().len(), 6);
    assert_eq!(log.borrow().last().unwrap().0, Commit);
    let initial = transform.get_untracked();
    screen.mouse_move(rect.center().x, rect.y + 5.0);
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_move(rect.center().x + 15.0, rect.y + 15.0);
    screen.get_by_role(Role::Button, "Other").focus();
    assert_eq!(transform.get_untracked(), initial);
    assert_eq!(log.borrow().last().unwrap().0, Cancel);
    screen.mouse_up(MouseButton::Middle);
    canvas.focus();
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    screen.mouse_move(rect.center().x, rect.y + 5.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(rect.center().x + 10.0, rect.y + 15.0);
    screen.mouse_up(MouseButton::Primary);
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    assert_eq!(log.borrow().last().unwrap().0, Commit);
    clean(&screen);
}

#[test]
fn node_header_regions_forward_middle_space_pan_and_wheel_to_core() {
    let mut results = Vec::new();
    for mode in 0..4 {
        let log = Rc::new(RefCell::new(Vec::new()));
        let seen = log.clone();
        let (screen, (transform, selected)) =
            kivact::test::render_with_handle(crate::options(), move || {
                let transform = signal(ViewTransform::default());
                let selected = signal(Vec::<u64>::new());
                (
                    (transform, selected),
                    app_node_editor(
                        signal(nodes()),
                        signal(vec![]),
                        ui::NodeState {
                            transform: transform.into(),
                            selection: selected.into(),
                            ..ui::NodeState::new()
                        },
                    )
                    .on_edit(move |phase| seen.borrow_mut().push(phase))
                    .name("Nodes")
                    .style(style().size(620, 240)),
                )
            });
        let r = screen
            .get_by_role(
                Role::Button,
                if mode == 3 {
                    "Source output Out"
                } else {
                    "Source"
                },
            )
            .rect()
            .unwrap();
        let p = if mode == 3 {
            r.center()
        } else {
            Point::new(r.x + 50.0, r.y + 12.0)
        };
        if matches!(mode, 1 | 3) {
            screen.get_by_role(Role::Canvas, "Nodes").focus();
            screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
        }
        screen.mouse_move(p.x, p.y);
        match mode {
            0 => {
                screen.mouse_down(MouseButton::Middle);
                screen.mouse_move(p.x + 20.0, p.y + 10.0);
                screen.mouse_up(MouseButton::Middle);
            }
            1 | 3 => {
                screen.mouse_down(MouseButton::Primary);
                screen.mouse_move(p.x + 20.0, p.y + 10.0);
                screen.mouse_up(MouseButton::Primary);
                screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
            }
            _ => screen.scroll(0.0, -100.0),
        }
        if matches!(mode, 1 | 3) {
            assert!(
                selected.get_untracked().is_empty(),
                "mode={mode} transform={:?} phases={:?} selection={:?}",
                transform.get_untracked(),
                log.borrow(),
                selected.get_untracked()
            );
            let before = transform.get_untracked();
            screen.get_by_role(Role::Button, "Source").click();
            assert_eq!(selected.get_untracked(), vec![1]);
            assert_eq!(transform.get_untracked(), before);
        }
        results.push((mode, transform.get_untracked(), log.borrow().clone()));
        clean(&screen);
    }
    assert!(
        results.iter().all(
            |(_, transform, phases)| *transform != ViewTransform::default()
                && *phases == [Begin, Commit]
        ),
        "{results:?}"
    );
}
