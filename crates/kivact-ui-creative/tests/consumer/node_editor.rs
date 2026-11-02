//! Role: Node editor sockets, links, deletion and parameters through the public API.

use crate::node_support::app_node_editor;
use crate::support::clean;
use kivact::draw::{Point, Size};
use kivact::prelude::*;
use kivact::view::A11yAction;
use kivact_ui_creative as ui;

fn test_node(
    id: u64,
    label: &str,
    x: f32,
    inputs: Vec<ui::NodeSocket>,
    outputs: Vec<ui::NodeSocket>,
) -> ui::NodeData {
    ui::NodeData::new(id, label, inputs, outputs)
        .position(Point::new(x, 30.0))
        .size(Size::new(160.0, 180.0))
        .color(rgb(0x8b7cff))
}

#[test]
fn typed_multiple_sockets_validate_reconnect_and_delete_nodes() {
    let (screen, (data, links)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let socket = ui::NodeSocket::new;
            let data = signal(vec![
                test_node(
                    1,
                    "Image",
                    30.0,
                    vec![],
                    vec![
                        socket("Image", "Image", kivact::style::rgb(0xffd25a)),
                        socket("Alpha", "Scalar", kivact::style::rgb(0x9b9ea8)),
                    ],
                ),
                test_node(
                    2,
                    "Mix",
                    300.0,
                    vec![
                        socket("Image", "Image", kivact::style::rgb(0xffd25a)),
                        socket("Factor", "Scalar", kivact::style::rgb(0x9b9ea8)),
                    ],
                    vec![socket("Image", "Image", kivact::style::rgb(0xffd25a))],
                ),
            ]);
            let links = signal(Vec::<ui::NodeLink>::new());
            (
                (data, links),
                app_node_editor(data, links, ui::NodeState::new())
                    .style(style().height(300))
                    .test_id("nodes"),
            )
        });
    screen
        .get_by_role(Role::Button, "Image output Alpha")
        .a11y(A11yAction::Activate);
    screen
        .get_by_role(Role::Button, "Mix input Image")
        .a11y(A11yAction::Activate);
    assert!(links.get_untracked().is_empty());
    screen
        .get_by_role(Role::Button, "Image output Image")
        .a11y(A11yAction::Activate);
    screen
        .get_by_role(Role::Button, "Mix input Image")
        .a11y(A11yAction::Activate);
    assert_eq!(links.get_untracked(), vec![ui::NodeLink::new(1, 0, 2, 0)]);
    screen
        .get_by_role(Role::Button, "Image output Alpha")
        .a11y(A11yAction::Activate);
    screen
        .get_by_role(Role::Button, "Mix input Factor")
        .a11y(A11yAction::Activate);
    assert_eq!(links.get_untracked().len(), 2);
    screen
        .get_by_role(Role::Button, "Image")
        .a11y(A11yAction::Select);
    screen.get_by_test_id("nodes").press_key(Key::Delete);
    assert_eq!(data.get_untracked().len(), 1);
    assert!(links.get_untracked().is_empty());
    clean(&screen);
}

#[test]
fn a_keyboard_connection_ends_when_the_editor_loses_focus() {
    let (screen, links) = kivact::test::render_with_handle(crate::support::options(), || {
        let socket = ui::NodeSocket::new;
        let data = signal(vec![
            test_node(
                1,
                "Image",
                30.0,
                vec![],
                vec![socket("Image", "Image", kivact::style::rgb(0xffd25a))],
            ),
            test_node(
                2,
                "Mix",
                300.0,
                vec![socket("Image", "Image", kivact::style::rgb(0xffd25a))],
                vec![],
            ),
        ]);
        let links = signal(Vec::<ui::NodeLink>::new());
        (
            links,
            column((
                app_node_editor(data, links, ui::NodeState::new())
                    .style(style().height(300))
                    .test_id("nodes"),
                kivact::ui::button("Outside"),
            )),
        )
    });
    screen.get_by_test_id("nodes").focus();
    screen
        .get_by_role(Role::Button, "Image output Image")
        .a11y(A11yAction::Activate);
    screen.get_by_role(Role::Button, "Outside").focus();
    screen
        .get_by_role(Role::Button, "Mix input Image")
        .a11y(A11yAction::Activate);
    assert!(
        links.get_untracked().is_empty(),
        "the pending connection ended with the focus"
    );
    clean(&screen);
}

#[test]
fn node_parameters_accept_finite_set_value_drag_and_toggle() {
    let (screen, data) = kivact::test::render_with_handle(crate::support::options(), || {
        let data = signal(vec![
            ui::NodeData::new(1, "Mix", vec![], vec![])
                .position(Point::new(20.0, 20.0))
                .size(Size::new(180.0, 180.0))
                .color(rgb(0xffd25a))
                .controls(vec![
                    ui::NodeControl::number("Factor", 60.0, 0.5, 0.0, 1.0, 0.01),
                    ui::NodeControl::toggle("Use Alpha", 90.0, true),
                ]),
        ]);
        (
            data,
            app_node_editor(data, signal(Vec::new()), ui::NodeState::new())
                .test_id("node-parameters"),
        )
    });
    let value = screen.get_by_role(Role::SpinButton, "Mix Factor");
    value.a11y_set_value("0.75");
    assert_eq!(value.value_range().2, Some(0.75));
    value.a11y_set_value("NaN");
    assert_eq!(value.value_range().2, Some(0.75));
    value.a11y_set_value("8");
    assert_eq!(value.value_range().2, Some(1.0));
    let alpha = screen.get_by_role(Role::Button, "Mix Use Alpha");
    assert_eq!(alpha.checked(), Some(CheckedState::Checked));
    alpha.a11y(A11yAction::Toggle);
    assert_eq!(alpha.checked(), Some(CheckedState::Unchecked));
    assert!(matches!(
        data.get_untracked()[0].controls[1].value,
        ui::NodeControlValue::Toggle(false)
    ));
    alpha.a11y(A11yAction::Activate);
    assert_eq!(alpha.checked(), Some(CheckedState::Checked));
    value.drag(Point::new(60.0, 90.0), Point::new(40.0, 90.0));
    assert_eq!(value.value_range().2, Some(0.8));
    clean(&screen);
}
