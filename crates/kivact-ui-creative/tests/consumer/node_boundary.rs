//! Role: Read-only node models, absolute proposals and application connection guards.
use super::*;

#[test]
fn ignored_node_requests_leave_items_unchanged_and_report_absolute_drag_values() {
    let reports = Rc::new(RefCell::new(Vec::new()));
    let seen = reports.clone();
    let (screen, data) = kivact::test::render_with_handle(crate::options(), move || {
        let data = signal(super::node_edit::nodes());
        (
            data,
            ui::node_editor(
                data,
                vec![],
                ui::NodeState {
                    selection: (signal(vec![1])).into(),
                    ..ui::NodeState::new()
                },
            )
            .on_edit(move |event| seen.borrow_mut().push(event))
            .name("Requests")
            .style(style().size(620, 240)),
        )
    });
    let header = screen.get_by_role(Role::Button, "Source").rect().unwrap();
    screen.mouse_move(header.x + 40.0, header.y + 10.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(header.x + 50.0, header.y + 15.0);
    screen.mouse_move(header.x + 70.0, header.y + 19.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(data.get_untracked(), super::node_edit::nodes());
    assert_eq!(reports.borrow().first(), Some(&ui::EditorEvent::Begin));
    assert_eq!(reports.borrow().last(), Some(&ui::EditorEvent::Commit));
    let positions = reports
        .borrow()
        .iter()
        .filter_map(|event| match event {
            ui::EditorEvent::Request(ui::NodeRequest::Move(values)) => Some(values.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(positions[0], [(1, Point::new(30.0, 25.0))]);
    assert_eq!(positions.last().unwrap(), &[(1, Point::new(50.0, 29.0))]);
    reports.borrow_mut().clear();
    let control = screen
        .get_by_role(Role::SpinButton, "Source Amount")
        .rect()
        .unwrap()
        .center();
    screen.mouse_move(control.x, control.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(control.x + 10.0, control.y);
    screen.mouse_move(control.x + 20.0, control.y);
    screen.mouse_up(MouseButton::Primary);
    let values = reports
        .borrow()
        .iter()
        .filter_map(|event| match event {
            ui::EditorEvent::Request(ui::NodeRequest::Control {
                value: ui::NodeControlValue::Number { value, .. },
                ..
            }) => Some(*value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(values[0], 3.0);
    assert_eq!(values.last(), Some(&4.0));
    assert_eq!(data.get_untracked(), super::node_edit::nodes());
    clean(&screen);
}

#[test]
fn default_connection_guard_accepts_application_types_and_callback_can_reject() {
    let reports = Rc::new(RefCell::new(Vec::new()));
    let seen = reports.clone();
    let guarded = reports.clone();
    let (screen, allow) = kivact::test::render_with_handle(crate::options(), move || {
        let mut nodes = super::node_edit::nodes();
        nodes[1].inputs[0].type_name = "Custom".into();
        let allow = signal(true);
        let nodes = signal(nodes);
        (
            allow,
            column((
                ui::node_editor(nodes, vec![], ui::NodeState::new())
                    .on_edit(move |event| seen.borrow_mut().push(event))
                    .name("Default")
                    .style(style().size(620, 240)),
                ui::node_editor(nodes, vec![], ui::NodeState::new())
                    .can_connect(move |_| allow.get_untracked())
                    .on_edit(move |event| guarded.borrow_mut().push(event))
                    .name("Guarded")
                    .style(style().size(620, 240)),
            )),
        )
    });
    let root = screen.get_by_role(Role::Canvas, "Default");
    root.get_by_role(Role::Button, "Source output Out")
        .a11y(A11yAction::Activate);
    root.get_by_role(Role::Button, "Sink input In")
        .a11y(A11yAction::Activate);
    assert!(
        reports
            .borrow()
            .contains(&ui::EditorEvent::Request(ui::NodeRequest::Connect(
                ui::NodeLink::new(1, 0, 2, 0)
            )))
    );
    assert_eq!(
        root.get_by_role(Role::Button, "Sink input In").value(),
        Some("Custom".into())
    );
    reports.borrow_mut().clear();
    allow.set(false);
    let guarded = screen.get_by_role(Role::Canvas, "Guarded");
    guarded
        .get_by_role(Role::Button, "Source output Out")
        .a11y(A11yAction::Activate);
    guarded
        .get_by_role(Role::Button, "Sink input In")
        .a11y(A11yAction::Activate);
    assert!(reports.borrow().is_empty());
    allow.set(true);
    guarded
        .get_by_role(Role::Button, "Source output Out")
        .a11y(A11yAction::Activate);
    guarded
        .get_by_role(Role::Button, "Sink input In")
        .a11y(A11yAction::Activate);
    assert!(
        reports
            .borrow()
            .contains(&ui::EditorEvent::Request(ui::NodeRequest::Connect(
                ui::NodeLink::new(1, 0, 2, 0)
            )))
    );
    reports.borrow_mut().clear();
    guarded
        .get_by_role(Role::Button, "Source output Out")
        .a11y(A11yAction::Activate);
    guarded
        .get_by_role(Role::Button, "Source input In")
        .a11y(A11yAction::Activate);
    assert!(reports.borrow().is_empty());
    clean(&screen);
}

#[test]
fn rewire_reports_disconnect_and_connect_in_one_gesture() {
    let reports = Rc::new(RefCell::new(Vec::new()));
    let seen = reports.clone();
    let old = ui::NodeLink::new(1, 0, 2, 0);
    let new = ui::NodeLink::new(1, 0, 3, 0);
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), move || {
        let mut nodes = super::node_edit::nodes();
        let mut third = nodes[1].clone();
        third.id = 3;
        third.label = "Third".into();
        third.position.y = 210.0;
        nodes.push(third);
        (
            (),
            ui::node_editor(nodes, vec![old], ui::NodeState::new())
                .on_edit(move |event| seen.borrow_mut().push(event))
                .style(style().size(620, 430)),
        )
    });
    let from = screen
        .get_by_role(Role::Button, "Sink input In")
        .rect()
        .unwrap()
        .center();
    let to = screen
        .get_by_role(Role::Button, "Third input In")
        .rect()
        .unwrap()
        .center();
    screen.mouse_move(from.x, from.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(to.x, to.y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        *reports.borrow(),
        [
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::NodeRequest::Disconnect(old)),
            ui::EditorEvent::Request(ui::NodeRequest::Connect(new)),
            ui::EditorEvent::Commit
        ]
    );
    reports.borrow_mut().clear();
    // Existing identical connections still reach the application reducer.
    for _ in 0..2 {
        screen
            .get_by_role(Role::Button, "Source output Out")
            .a11y(A11yAction::Activate);
        screen
            .get_by_role(Role::Button, "Sink input In")
            .a11y(A11yAction::Activate);
    }
    assert_eq!(
        reports
            .borrow()
            .iter()
            .filter(|event| **event == ui::EditorEvent::Request(ui::NodeRequest::Connect(old)))
            .count(),
        2
    );
    clean(&screen);
}

#[test]
fn single_node_request_captures_value_before_begin_callback() {
    let (screen, (data, events)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(super::node_edit::nodes());
        let events = signal(Vec::new());
        (
            (data, events),
            ui::node_editor(data, vec![], ui::NodeState::new())
                .on_edit(move |event| {
                    if event == ui::EditorEvent::Begin {
                        data.update(|nodes| {
                            nodes[0].controls[0] =
                                ui::NodeControl::number("Amount", 75.0, 7.0, 0.0, 20.0, 2.0);
                        });
                    }
                    events.update(|events| events.push(event));
                })
                .style(style().size(620, 240)),
        )
    });
    screen
        .get_by_role(Role::SpinButton, "Source Amount")
        .a11y(A11yAction::Increment);
    assert_eq!(
        events.get_untracked(),
        vec![
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::NodeRequest::Control {
                node: 1,
                index: 0,
                value: ui::NodeControlValue::Number {
                    value: 2.1,
                    min: 0.0,
                    max: 10.0,
                    step: 0.1,
                },
            }),
            ui::EditorEvent::Commit,
        ]
    );
    let mut expected = super::node_edit::nodes();
    expected[0].controls[0] = ui::NodeControl::number("Amount", 75.0, 7.0, 0.0, 20.0, 2.0);
    assert_eq!(data.get_untracked(), expected);
    clean(&screen);
}
