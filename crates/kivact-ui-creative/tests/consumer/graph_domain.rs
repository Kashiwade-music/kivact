//! Role: Public graph domain editing, view geometry and selective rollback contracts.
use super::*;
use kivact::ui::EditPhase;
#[derive(Clone, Copy)]
struct Model {
    curves: Signal<Vec<ui::Curve>>,
    selection: Signal<Vec<u64>>,
    view: Signal<ui::GraphView>,
    frame: Signal<f64>,
    lanes: Signal<Vec<ui::Lane>>,
    snap: Signal<bool>,
    phases: Signal<Vec<EditPhase>>,
    disabled: Signal<bool>,
    visible: Signal<bool>,
    present: Signal<bool>,
}
fn fixture() -> (Screen, Model) {
    kivact::test::render_with_handle(crate::options(), || {
        let model = Model {
            curves: signal(vec![
                ui::Curve::new(
                    10,
                    "Position",
                    vec![
                        ui::CurveKey::new(2, 8.0, 2.0),
                        ui::CurveKey::new(1, 2.0, -1.0),
                    ],
                ),
                ui::Curve::new(20, "Opacity", vec![ui::CurveKey::new(3, 5.0, 0.5)]),
            ]),
            selection: signal(Vec::new()),
            view: signal(ui::GraphView {
                time: (0.0, 10.0),
                value: (-2.0, 3.0),
            }),
            frame: signal(0.0),
            lanes: signal(vec![
                ui::Lane::new(10, "Position"),
                ui::Lane::new(20, "Opacity"),
            ]),
            snap: signal(false),
            phases: signal(Vec::new()),
            disabled: signal(false),
            visible: signal(true),
            present: signal(true),
        };
        (
            model,
            column((
                show(model.present, move || {
                    app_graph(
                        model.curves,
                        ui::GraphState {
                            view: model.view.into(),
                            selection: model.selection.into(),
                            playhead: model.frame.into(),
                        },
                    )
                    .lanes(model.lanes)
                    .snap(model.snap)
                    .step(0.5, 0.25)
                    .on_edit(graph_edits(model.curves, move |phase| {
                        model.phases.update(|p| p.push(phase))
                    }))
                    .name("Graph")
                    .disabled(model.disabled)
                    .hidden(move || !model.visible.get())
                    .style(style().size(700, 322))
                }),
                kivact::ui::button("Outside"),
            )),
        )
    })
}
fn key(model: Model, id: u64) -> ui::CurveKey {
    model
        .curves
        .get_untracked()
        .iter()
        .flat_map(|c| &c.keys)
        .find(|k| k.id == id)
        .expect("key")
        .clone()
}
fn drag(screen: &Screen, name: &str, dx: f32, dy: f32) {
    let item = screen.get_by_role(Role::Button, name);
    let p = item.rect().expect("key rect").center();
    item.drag(p, Point::new(p.x + dx, p.y + dy));
}
fn start(screen: &Screen, name: &str) {
    let p = screen
        .get_by_role(Role::Button, name)
        .rect()
        .expect("key rect")
        .center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(p.x + 30.0, p.y - 15.0);
}
#[test]
fn real_values_axes_selection_keyboard_and_accessibility() {
    let (screen, m) = fixture();
    let first = screen.get_by_role(Role::Button, "Position key 2");
    let second = screen.get_by_role(Role::Button, "Position key 8");
    let a = first.rect().unwrap().center();
    let b = second.rect().unwrap().center();
    assert!(near(f64::from(b.x - a.x), 300.0, 1e-5));
    assert!(near(f64::from(a.y - b.y), 180.0, 1e-5));
    first.a11y(A11yAction::Select);
    assert_eq!(m.selection.get_untracked(), [1]);
    first.press_key(Key::ArrowRight);
    assert!(near(key(m, 1).time, 2.5, 1e-5));
    assert!(near(key(m, 1).value, -1.0, 1e-5));
    screen
        .get_by_role(Role::Button, "Position key 2.5")
        .a11y_with_value(A11yAction::SetValue, "3, -0.5");
    assert!(near(key(m, 1).time, 3.0, 1e-5));
    assert!(near(key(m, 1).value, -0.5, 1e-5));
    screen
        .get_by_role(Role::Slider, "Playhead")
        .a11y_with_value(A11yAction::SetValue, "4.25");
    assert!(near(m.frame.get_untracked(), 4.25, 1e-5));
    screen
        .get_by_role(Role::Slider, "Playhead")
        .press_key(Key::ArrowDown);
    assert!(near(m.frame.get_untracked(), 3.75, 1e-5));
    assert_eq!(
        m.curves.get_untracked()[0]
            .keys
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        [2, 1]
    );
    m.lanes.set(vec![ui::Lane::new(20, "Opacity")]);
    screen.run_until_idle();
    assert!(
        screen
            .query_by_role(Role::Button, "Position key 3")
            .is_none()
    );
    let canvas = screen.get_by_role(Role::Canvas, "Graph");
    canvas.press_key(Key::Char('a'));
    clean(&screen);
}
#[test]
fn group_drag_snap_and_cancel_preserve_external_fields() {
    let (screen, m) = fixture();
    m.selection.set(vec![1, 2]);
    screen.run_until_idle();
    drag(&screen, "Position key 2", 17.0, -9.0);
    assert!(near(key(m, 1).time, 2.34, 1e-5));
    assert!(near(key(m, 2).time, 8.34, 1e-5));
    assert!(near(key(m, 1).value, -0.85, 1e-5));
    assert_eq!(m.selection.get_untracked(), [1, 2]);
    m.snap.set(true);
    screen.run_until_idle();
    drag(&screen, "Position key 2.34", 17.0, -9.0);
    assert!(near(key(m, 1).time, 2.5, 1e-5));
    assert!(near(key(m, 1).value, -0.75, 1e-5));
    let before = key(m, 1);
    start(&screen, "Position key 2.5");
    m.curves.update(|curves| {
        curves[0].label = "External".into();
        curves[0].keys[1].interpolation = ui::Interpolation::Linear;
        curves[1].keys[0].value = 1.5;
        curves[1].keys.push(ui::CurveKey::new(4, 6.0, 1.0));
    });
    screen.run_until_idle();
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert!(near(key(m, 1).time, before.time, 1e-5));
    assert!(near(key(m, 1).value, before.value, 1e-5));
    assert_eq!(key(m, 1).interpolation, ui::Interpolation::Linear);
    assert_eq!(m.curves.get_untracked()[0].label, "External");
    assert!(near(key(m, 3).value, 1.5, 1e-5));
    assert_eq!(key(m, 4).id, 4);
    assert_eq!(m.phases.get_untracked().last(), Some(&EditPhase::Cancel));
    clean(&screen);
}
#[test]
fn tangent_edits_are_relative_and_do_not_move_keys() {
    let (screen, m) = fixture();
    screen
        .get_by_role(Role::Button, "Position key 2")
        .a11y(A11yAction::Select);
    let tangent = screen.get_by_role(Role::Button, "Position key 2 outgoing tangent");
    tangent.a11y_with_value(A11yAction::SetValue, "1.5 0.75");
    assert!(near(key(m, 1).out_tangent.time, 1.5, 1e-5));
    assert!(near(key(m, 1).out_tangent.value, 0.75, 1e-5));
    assert!(near(key(m, 1).time, 2.0, 1e-5));
    tangent.press_key(Key::ArrowUp);
    assert!(near(key(m, 1).out_tangent.value, 1.0, 1e-5));
    assert!(near(key(m, 1).value, -1.0, 1e-5));
    m.curves
        .update(|curves| curves[0].keys[0].interpolation = ui::Interpolation::Linear);
    m.selection.set(vec![2]);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Position key 8 incoming tangent");
    assert!(
        screen
            .query_by_role(Role::Button, "Position key 8 outgoing tangent")
            .is_none()
    );
    clean(&screen);
}
#[test]
fn pan_zoom_and_fit_use_independent_domain_axes() {
    let (screen, m) = fixture();
    let canvas = screen.get_by_role(Role::Canvas, "Graph");
    screen.mouse_move(450.0, 170.0);
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_move(475.0, 182.0);
    screen.mouse_up(MouseButton::Middle);
    assert!(near(m.view.get_untracked().time.0, -0.5, 1e-5));
    assert!(near(m.view.get_untracked().value.0, -1.8, 1e-5));
    let before = m.view.get_untracked();
    screen.key_down(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    canvas.scroll(0.0, -100.0);
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    let after = m.view.get_untracked();
    assert!(after.time.1 - after.time.0 < before.time.1 - before.time.0);
    assert_eq!(after.value, before.value);
    canvas.press_key(Key::Char('f'));
    assert_eq!(
        m.view.get_untracked(),
        ui::GraphView {
            time: (1.4, 8.6),
            value: (-1.3, 2.3)
        }
    );
    m.selection.set(vec![3]);
    canvas.press_key(Key::Home);
    assert_eq!(
        m.view.get_untracked(),
        ui::GraphView {
            time: (4.5, 5.5),
            value: (0.25, 0.75)
        }
    );
    clean(&screen);
}
#[test]
fn disable_focus_loss_hide_and_lane_filter_cancel_active_edits() {
    for mode in 0..6 {
        let (screen, m) = fixture();
        let initial = key(m, 1);
        start(&screen, "Position key 2");
        assert_ne!(key(m, 1).time, initial.time);
        match mode {
            0 => m.disabled.set(true),
            1 => screen.get_by_role(Role::Button, "Outside").focus(),
            2 => m.visible.set(false),
            3 => m.lanes.set(vec![ui::Lane::new(20, "Opacity")]),
            4 => m.present.set(false),
            _ => screen.set_window_focused(false),
        };
        screen.run_until_idle();
        assert!(near(key(m, 1).time, initial.time, 1e-5));
        assert!(near(key(m, 1).value, initial.value, 1e-5));
        assert_eq!(m.phases.get_untracked().last(), Some(&EditPhase::Cancel));
        screen.mouse_up(MouseButton::Primary);
        clean(&screen);
    }
}
#[test]
fn invalid_axes_and_extreme_keys_draw_without_rewriting_bindings() {
    let (screen, m) = fixture();
    let view = ui::GraphView {
        time: (f64::NAN, 0.0),
        value: (-2.0, 3.0),
    };
    m.view.set(view);
    m.curves.update(|curves| {
        curves[0].keys[0].time = f64::MAX;
        curves[0].keys[0].value = f64::INFINITY;
    });
    screen.run_until_idle();
    assert!(m.view.get_untracked().time.0.is_nan());
    assert_eq!(m.view.get_untracked().value, view.value);
    screen.get_by_role(Role::Button, "Position key 2");
    clean(&screen);
}

#[test]
fn list_selection_marquee_visible_delete_and_interpolation_use_domain_keys() {
    let (screen, m) = fixture();
    screen.get_by_role(Role::Button, "Position key 2").click();
    screen.key_down(
        Key::Unidentified,
        KeyCode::ShiftLeft,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    screen.get_by_role(Role::Button, "Position key 8").click();
    screen.key_up(Key::Unidentified, KeyCode::ShiftLeft, Modifiers::default());
    assert_eq!(m.selection.get_untracked(), [1, 2]);
    screen.get_by_role(Role::Button, "Position key 2").click();
    assert_eq!(m.selection.get_untracked(), [1]);
    let canvas = screen.get_by_role(Role::Canvas, "Graph");
    canvas.drag(Point::new(285.0, 70.0), Point::new(615.0, 270.0));
    assert_eq!(m.selection.get_untracked(), [1, 2, 3]);
    canvas.press_key(Key::Char('1'));
    assert!(
        m.curves
            .get_untracked()
            .iter()
            .flat_map(|c| &c.keys)
            .all(|k| k.interpolation == ui::Interpolation::Constant)
    );
    canvas.press_key(Key::Escape);
    assert!(m.selection.get_untracked().is_empty());
    m.lanes.set(vec![ui::Lane::new(20, "Opacity")]);
    screen.run_until_idle();
    screen.key_down(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    canvas.press_key(Key::Char('a'));
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    assert_eq!(m.selection.get_untracked(), [3]);
    m.selection.set(vec![1, 2, 3]);
    canvas.press_key(Key::Delete);
    assert_eq!(m.curves.get_untracked()[0].keys.len(), 2);
    assert!(m.curves.get_untracked()[1].keys.is_empty());
    clean(&screen);
}

#[test]
fn axis_zoom_keeps_pointer_anchor_and_snap_configuration_order() {
    let (screen, m) = fixture();
    let before = m.view.get_untracked();
    screen.mouse_move(400.0, 142.0);
    screen.key_down(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        },
    );
    screen.scroll(0.0, -80.0);
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    let after = m.view.get_untracked();
    assert_eq!(before.time, after.time);
    assert!(after.value.1 - after.value.0 < 5.0);
    assert!(near(
        after.value.1 - 0.4 * (after.value.1 - after.value.0),
        1.0,
        1e-5
    ));
    clean(&screen);
    for step_first in [false, true] {
        let (screen, curves) = kivact::test::render_with_handle(crate::options(), move || {
            let curves = signal(vec![ui::Curve::new(
                1,
                "Order",
                vec![ui::CurveKey::new(1, 20.2, 0.2)],
            )]);
            let graph = app_graph(curves, ui::GraphState::new());
            let graph = if step_first {
                graph.step(2.0, 0.5).snap(true)
            } else {
                graph.snap(true).step(2.0, 0.5)
            };
            (curves, graph.name("Graph").style(style().size(700, 322)))
        });
        drag(&screen, "Order key 20.2", 17.0, -9.0);
        assert!(near(curves.get_untracked()[0].keys[0].time, 24.0, 1e-5));
        assert!(near(curves.get_untracked()[0].keys[0].value, 0.5, 1e-5));
        clean(&screen);
    }
}

#[test]
fn tangent_cancel_restores_application_key_snapshot_and_never_revives_keys() {
    let (screen, m) = fixture();
    screen
        .get_by_role(Role::Button, "Position key 2")
        .a11y(A11yAction::Select);
    drag(&screen, "Position key 2 outgoing tangent", 25.0, -30.0);
    assert!(near(key(m, 1).out_tangent.time, 1.5, 1e-5));
    assert!(near(key(m, 1).out_tangent.value, 0.5, 1e-5));
    assert!(near(key(m, 1).time, 2.0, 1e-5));
    start(&screen, "Position key 2 outgoing tangent");
    m.curves.update(|curves| {
        curves[0].keys[1].in_tangent = ui::CurveTangent::new(-4.0, -2.0);
        curves[0].keys[1].value = -0.5;
        curves[0].keys.reverse();
    });
    screen.run_until_idle();
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert!(near(key(m, 1).out_tangent.time, 1.5, 1e-5));
    assert!(near(key(m, 1).out_tangent.value, 0.5, 1e-5));
    assert!(near(key(m, 1).in_tangent.time, -1.0, 1e-5));
    assert!(near(key(m, 1).value, -1.0, 1e-5));
    assert_eq!(m.curves.get_untracked()[0].keys[0].id, 1);
    start(&screen, "Position key 2");
    m.curves
        .update(|curves| curves[0].keys.retain(|key| key.id != 1));
    screen.run_until_idle();
    assert!(
        m.curves.get_untracked()[0]
            .keys
            .iter()
            .all(|key| key.id != 1)
    );
    assert_eq!(m.phases.get_untracked().last(), Some(&EditPhase::Cancel));
    screen.mouse_up(MouseButton::Primary);
    clean(&screen);
}

#[test]
fn live_snap_uses_total_motion_and_key_regions_win_over_overlapping_handles() {
    let (screen, m) = fixture();
    m.snap.set(true);
    m.selection.set(vec![1]);
    screen.run_until_idle();
    let p = screen
        .get_by_role(Role::Button, "Position key 2")
        .rect()
        .unwrap()
        .center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    for dx in 1..=20 {
        screen.mouse_move(p.x + dx as f32, p.y);
    }
    assert!(near(key(m, 1).time, 2.5, 1e-5));
    screen.mouse_up(MouseButton::Primary);
    assert!(near(key(m, 1).time, 2.5, 1e-5));
    m.view.set(ui::GraphView::default());
    m.selection.set(vec![1, 2]);
    screen.run_until_idle();
    let p = screen
        .get_by_role(Role::Button, "Position key 2.5")
        .rect()
        .unwrap()
        .center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(p.x + 10.0, p.y);
    screen.mouse_up(MouseButton::Primary);
    assert!(near(key(m, 1).time, 4.5, 1e-5));
    assert!(near(key(m, 2).time, 10.0, 1e-5));
    assert!(near(key(m, 1).out_tangent.time, 1.0, 1e-5));
    assert_eq!(m.selection.get_untracked(), [1, 2]);
    clean(&screen);
}

#[test]
fn graph_step_is_reactive_and_invalid_steps_fall_back_to_the_defaults() {
    let (screen, (curves, time, value)) =
        kivact::test::render_with_handle(crate::options(), || {
            let curves = signal(vec![ui::Curve::new(
                1,
                "Step",
                vec![ui::CurveKey::new(1, 2.0, 0.0)],
            )]);
            let (time, value) = (signal(0.5), signal(0.25));
            (
                (curves, time, value),
                app_graph(
                    curves,
                    ui::GraphState {
                        selection: (signal(vec![1])).into(),
                        ..ui::GraphState::new()
                    },
                )
                .snap(true)
                .step(time, value)
                .name("Graph")
                .style(style().size(700, 322)),
            )
        });
    let at = || curves.get_untracked()[0].keys[0].clone();
    let press = |name: &str, key| screen.get_by_role(Role::Button, name).press_key(key);
    press("Step key 2", Key::ArrowRight);
    press("Step key 2.5", Key::ArrowUp);
    assert!(near(at().time, 2.5, 1e-5));
    assert!(near(at().value, 0.25, 1e-5));
    time.set(2.0);
    value.set(0.5);
    screen.run_until_idle();
    press("Step key 2.5", Key::ArrowRight);
    press("Step key 4", Key::ArrowUp);
    assert!(near(at().time, 4.0, 1e-5));
    assert!(near(at().value, 1.0, 1e-5));
    time.set(0.0);
    value.set(f64::NAN);
    screen.run_until_idle();
    press("Step key 4", Key::ArrowRight);
    press("Step key 5", Key::ArrowUp);
    assert!(near(at().time, 5.0, 1e-5));
    assert!(near(at().value, 1.01, 1e-5));
    clean(&screen);
}

#[test]
fn ignored_graph_requests_keep_data_and_geometry_and_drag_proposals_are_absolute() {
    let (screen, (curves, events, snap)) =
        kivact::test::render_with_handle(crate::options(), || {
            let curves = signal(vec![ui::Curve::new(
                1,
                "Readonly",
                vec![ui::CurveKey::new(7, 2.0, 0.0)],
            )]);
            let events = signal(Vec::new());
            let snap = signal(false);
            (
                (curves, events, snap),
                ui::graph_editor(
                    curves,
                    ui::GraphState {
                        view: (signal(ui::GraphView {
                            time: (0.0, 10.0),
                            value: (-2.0, 3.0),
                        }))
                        .into(),
                        selection: (signal(vec![7])).into(),
                        ..ui::GraphState::new()
                    },
                )
                .snap(snap)
                .step(0.5, 0.25)
                .on_edit(move |event| events.update(|events| events.push(event)))
                .name("Readonly graph")
                .style(style().size(700, 322)),
            )
        });
    let original = curves.get_untracked();
    let item = screen.get_by_role(Role::Button, "Readonly key 2");
    let rect = item.rect().unwrap();
    let p = rect.center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(p.x + 17.0, p.y - 9.0);
    screen.mouse_move(p.x + 34.0, p.y - 18.0);
    let moves = events
        .get_untracked()
        .into_iter()
        .filter_map(|event| match event {
            ui::EditorEvent::Request(ui::GraphRequest::Move(values)) => Some(values[0]),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(moves.len(), 2);
    assert!(near(moves[0].time, 2.34, 1e-5));
    assert!(near(moves[1].time, 2.68, 1e-5));
    assert!(near(moves[1].value, 0.3, 1e-5));
    snap.set(true);
    screen.run_until_idle();
    screen.mouse_up(MouseButton::Primary);
    let log = events.get_untracked();
    assert_eq!(log.first(), Some(&ui::EditorEvent::Begin));
    assert_eq!(log.last(), Some(&ui::EditorEvent::Commit));
    let ui::EditorEvent::Request(ui::GraphRequest::Move(values)) = &log[log.len() - 2] else {
        panic!("release proposal");
    };
    assert!(near(values[0].time, 2.5, 1e-5));
    assert!(near(values[0].value, 0.25, 1e-5));
    assert_eq!(curves.get_untracked(), original);
    assert_eq!(item.rect().unwrap(), rect);
    events.set(Vec::new());
    item.press_key(Key::ArrowRight);
    assert!(matches!(
        events.get_untracked().as_slice(),
        [
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::GraphRequest::Move(_)),
            ui::EditorEvent::Commit
        ]
    ));
    events.set(Vec::new());
    item.press_key(Key::Char('1'));
    assert_eq!(
        events.get_untracked(),
        [
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::GraphRequest::Interpolate(
                vec![7],
                ui::Interpolation::Constant
            )),
            ui::EditorEvent::Commit
        ]
    );
    events.set(Vec::new());
    item.a11y_with_value(A11yAction::SetValue, "4 1");
    assert!(matches!(
        events.get_untracked().as_slice(),
        [
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::GraphRequest::Move(_)),
            ui::EditorEvent::Commit
        ]
    ));
    events.set(Vec::new());
    let tangent = screen.get_by_role(Role::Button, "Readonly key 2 outgoing tangent");
    let tangent_rect = tangent.rect().unwrap();
    tangent.a11y_with_value(A11yAction::SetValue, "1.5 0.75");
    let log = events.get_untracked();
    let ui::EditorEvent::Request(ui::GraphRequest::Move(values)) = &log[1] else {
        panic!("tangent proposal");
    };
    assert_eq!(values[0].out_tangent, ui::CurveTangent::new(1.5, 0.75));
    assert_eq!(values[0].time, 2.0);
    assert_eq!(values[0].value, 0.0);
    assert_eq!(tangent.rect().unwrap(), tangent_rect);
    assert_eq!(curves.get_untracked(), original);
    events.set(Vec::new());
    item.press_key(Key::Delete);
    assert_eq!(
        events.get_untracked(),
        [
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::GraphRequest::Delete(vec![7])),
            ui::EditorEvent::Commit
        ]
    );
    assert_eq!(curves.get_untracked(), original);
    assert_eq!(item.rect().unwrap(), rect);
    events.set(Vec::new());
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(p.x + 20.0, p.y);
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert!(matches!(
        events.get_untracked().as_slice(),
        [
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::GraphRequest::Move(_)),
            ui::EditorEvent::Cancel
        ]
    ));
    assert_eq!(curves.get_untracked(), original);
    assert_eq!(item.rect().unwrap(), rect);
    clean(&screen);
}
