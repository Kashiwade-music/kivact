//! Role: The private node-editor identity experiment through public UI APIs.
//! Held locators observe retired IDs; this does not inject an OS event from
//! a previous scene generation, which the public test API cannot construct.

use super::app_node_editor;
use super::clean;
use kivact::draw::{Point, Size};
use kivact::prelude::*;
use kivact::view::A11yAction;
use kivact::view::MouseButton;
use kivact_ui_creative as ui;

const SOURCE: u64 = u64::MAX;
const SINK: u64 = u64::MAX - 1;

fn node(id: u64, label: &str, x: f32) -> ui::NodeData {
    ui::NodeData::new(
        id,
        label,
        vec![ui::NodeSocket::new(
            "Value",
            "Scalar",
            kivact::style::rgb(0x9b9ea8),
        )],
        vec![ui::NodeSocket::new(
            "Value",
            "Scalar",
            kivact::style::rgb(0x9b9ea8),
        )],
    )
    .controls(vec![ui::NodeControl::number(
        "Amount", 80.0, 2.0, 0.0, 10.0, 1.0,
    )])
    .position(Point::new(x, 20.0))
    .size(Size::new(180.0, 160.0))
    .color(rgb(0x7d88c7))
}

#[test]
fn node_targets_share_draw_pointer_keyboard_and_at_identity_without_id_encoding() {
    let (screen, (items, selected, selected_link)) =
        kivact::test::render_with_handle(crate::options(), || {
            let items = signal(vec![
                node(SOURCE, "Source", 20.0),
                node(SINK, "Sink", 320.0),
            ]);
            let selected = signal(Vec::new());
            let links = signal(vec![ui::NodeLink::new(SOURCE, 0, SINK, 0)]);
            let selected_link = signal(None);
            (
                (items, selected, selected_link),
                app_node_editor(
                    items,
                    links,
                    ui::NodeState {
                        selection: selected.into(),
                        link: selected_link.into(),
                        ..ui::NodeState::new()
                    },
                )
                .style(style().size(620, 240))
                .test_id("nodes"),
            )
        });
    let source = screen.get_by_role(Role::Button, "Source");
    source.a11y(A11yAction::Select);
    assert_eq!(selected.get_untracked(), vec![SOURCE]);
    screen.get_by_test_id("nodes").press_key(Key::ArrowRight);
    assert!(items.get_untracked()[0].position.x > 20.0);
    screen.get_by_role(Role::Button, "Sink").click();
    assert_eq!(selected.get_untracked(), vec![SINK]);
    let amount = screen.get_by_role(Role::SpinButton, "Source Amount");
    amount.a11y_with_value(A11yAction::SetValue, "4");
    assert_eq!(amount.value_range().2, Some(4.0));
    screen
        .get_by_role(Role::Button, "Link Source output 0 to Sink input 0")
        .a11y(A11yAction::Select);
    assert_eq!(
        selected_link.get_untracked(),
        Some(ui::NodeLink::new(SOURCE, 0, SINK, 0))
    );
    clean(&screen);
}

#[test]
fn deleted_and_regenerated_targets_retire_all_previous_region_locators() {
    let (screen, (items, selected)) = kivact::test::render_with_handle(crate::options(), || {
        let items = signal(vec![
            node(SOURCE, "Source", 20.0),
            node(SINK, "Sink", 320.0),
        ]);
        let selected = signal(Vec::new());
        let links = signal(vec![ui::NodeLink::new(SOURCE, 0, SINK, 0)]);
        (
            (items, selected),
            app_node_editor(
                items,
                links,
                ui::NodeState {
                    selection: selected.into(),
                    ..ui::NodeState::new()
                },
            )
            .style(style().size(620, 240)),
        )
    });
    let old_item = screen.get_by_role(Role::Button, "Source");
    let old_socket = screen.get_by_role(Role::Button, "Source output Value");
    let old_control = screen.get_by_role(Role::SpinButton, "Source Amount");
    let old_link = screen.get_by_role(Role::Button, "Link Source output 0 to Sink input 0");
    let sink = screen.get_by_role(Role::Button, "Sink");
    items.update(|nodes| nodes.retain(|node| node.id != SOURCE));
    screen.run_until_idle();
    for old in [&old_item, &old_socket, &old_control, &old_link] {
        assert_eq!(
            old.name(),
            "",
            "a deleted ID must not resolve to another target"
        );
    }
    assert_eq!(sink.name(), "Sink", "unchanged targets keep their identity");
    items.update(|nodes| nodes.push(node(SOURCE, "Regenerated", 20.0)));
    screen.run_until_idle();
    for old in [&old_item, &old_socket, &old_control, &old_link] {
        assert_eq!(old.name(), "", "re-added keys receive fresh IDs");
    }
    screen
        .get_by_role(Role::Button, "Regenerated")
        .a11y(A11yAction::Select);
    assert_eq!(selected.get_untracked(), vec![SOURCE]);
    screen.get_by_role(Role::Button, "Link Regenerated output 0 to Sink input 0");
    clean(&screen);
}

#[test]
fn filtered_targets_leave_the_scene_and_cannot_be_hit_by_geometry_fallbacks() {
    let (screen, (visible, selected_link, selected, links)) =
        kivact::test::render_with_handle(crate::options(), || {
            let sink = node(SINK, "Sink", 320.0);
            let items = signal(vec![node(SOURCE, "Source", 20.0), sink]);
            let links = signal(vec![ui::NodeLink::new(SOURCE, 0, SINK, 0)]);
            let selected_link = signal(None);
            let selected = signal(Vec::new());
            let visible = signal(vec![0_usize, 1]);
            // Visibility belongs to the application's projection. Writes merge visible
            // nodes into the canonical collection so hidden nodes and links survive.
            let shown = binding(
                (move || {
                    items
                        .get()
                        .into_iter()
                        .filter(|node| {
                            visible
                                .get()
                                .contains(&if node.id == SOURCE { 0 } else { 1 })
                        })
                        .collect::<Vec<_>>()
                })
                .into_value(),
                move |next: Vec<ui::NodeData>| {
                    items.update(|all| {
                        for node in next {
                            if let Some(existing) = all.iter_mut().find(|n| n.id == node.id) {
                                *existing = node;
                            }
                        }
                    });
                },
            );
            (
                (visible, selected_link, selected, links),
                ui::node_editor(
                    shown,
                    links,
                    ui::NodeState {
                        link: selected_link.into(),
                        selection: selected.into(),
                        ..ui::NodeState::new()
                    },
                )
                .on_edit(super::node_support::edits(items, links, None))
                .style(style().size(620, 240)),
            )
        });
    let old_link = screen.get_by_role(Role::Button, "Link Source output 0 to Sink input 0");
    let point = old_link.rect().expect("visible link").center();
    let sink = screen
        .get_by_role(Role::Button, "Sink")
        .rect()
        .expect("visible sink");
    let input = screen
        .get_by_role(Role::Button, "Sink input Value")
        .rect()
        .expect("visible input")
        .center();
    let output = screen
        .get_by_role(Role::Button, "Source output Value")
        .rect()
        .expect("visible output")
        .center();
    visible.set(vec![0]);
    screen.run_until_idle();
    assert_eq!(old_link.name(), "");
    assert!(screen.query_by_role(Role::Button, "Sink").is_none());
    screen.click_at(point.x, point.y);
    assert_eq!(selected_link.get_untracked(), None);
    screen.mouse_move(sink.x - 10.0, sink.y - 10.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(sink.x + sink.width + 10.0, sink.y + 190.0);
    screen.mouse_up(MouseButton::Primary);
    assert!(
        selected.get_untracked().is_empty(),
        "marquee excludes hidden items"
    );
    links.set(Vec::new());
    screen.run_until_idle();
    screen.mouse_move(output.x, output.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(input.x, input.y);
    screen.mouse_up(MouseButton::Primary);
    assert!(
        links.get_untracked().is_empty(),
        "link drop excludes hidden sockets"
    );
    links.set(vec![ui::NodeLink::new(SOURCE, 0, SINK, 0)]);
    visible.set(vec![0, 1]);
    screen.run_until_idle();
    assert_eq!(old_link.name(), "", "hidden targets retired their IDs");
    screen
        .get_by_role(Role::Button, "Link Source output 0 to Sink input 0")
        .a11y(A11yAction::Select);
    assert_eq!(
        selected_link.get_untracked(),
        Some(ui::NodeLink::new(SOURCE, 0, SINK, 0))
    );
    clean(&screen);
}

#[test]
fn invalid_region_scene_reports_a_location_but_keeps_visual_and_keyboard_fallback() {
    let (screen, (items, links)) = kivact::test::render_with_handle(crate::options(), || {
        let items = signal(vec![
            node(SOURCE, "Source", 20.0),
            node(SINK, "Sink", 320.0),
        ]);
        let selected = signal(vec![SOURCE]);
        let links = signal(vec![ui::NodeLink::new(SOURCE, 0, SINK, 0)]);
        (
            (items, links),
            app_node_editor(
                items,
                links,
                ui::NodeState {
                    selection: selected.into(),
                    ..ui::NodeState::new()
                },
            )
            .style(style().size(620, 240))
            .test_id("nodes"),
        )
    });
    let source = screen
        .get_by_role(Role::Button, "Source")
        .rect()
        .expect("drawn item");
    let x = (source.x + 120.0) as u32;
    let y = (source.y + 12.0) as u32;
    let before = screen.screenshot().pixel(x, y);
    links.update(|edges| edges.push(ui::NodeLink::new(SOURCE, 0, SINK, 0)));
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Button, "Source").is_none());
    assert_eq!(
        screen.screenshot().pixel(x, y),
        before,
        "failed allocation does not omit painting"
    );
    assert!(
        screen
            .errors()
            .iter()
            .any(|error| error.message() == "duplicate node link" && error.location().is_some())
    );
    screen.get_by_test_id("nodes").press_key(Key::ArrowRight);
    assert!(
        items.get_untracked()[0].position.x > 20.0,
        "keyboard editing does not depend on region allocation"
    );
}

#[test]
fn dynamic_node_semantics_follow_updates_insertions_reorders_and_removals() {
    let (screen, items) = kivact::test::render_with_handle(crate::options(), || {
        let items = signal(vec![node(SOURCE, "Source", 20.0)]);
        (
            items,
            app_node_editor(items, signal(Vec::new()), ui::NodeState::new())
                .style(style().size(620, 240)),
        )
    });
    let source = screen.get_by_role(Role::Button, "Source");
    let amount = screen.get_by_role(Role::SpinButton, "Source Amount");
    items.update(|nodes| {
        nodes[0].label = "Renamed".into();
        let data = &mut nodes[0];
        data.outputs[0].label = "Result".into();
        data.controls[0] = ui::NodeControl::number("Level", 80.0, 4.0, -2.0, 12.0, 1.0);
        nodes.insert(0, node(SINK, "Inserted", 320.0));
    });
    screen.run_until_idle();
    assert_eq!(source.name(), "Renamed");
    assert_eq!(amount.name(), "Renamed Level");
    assert_eq!(amount.value_range(), (Some(-2.0), Some(12.0), Some(4.0)));
    screen.get_by_role(Role::Button, "Renamed output Result");
    screen.get_by_role(Role::Button, "Inserted");
    items.update(|nodes| nodes.reverse());
    screen.run_until_idle();
    assert_eq!(source.name(), "Renamed");
    amount.a11y_with_value(A11yAction::SetValue, "6");
    assert_eq!(amount.value_range().2, Some(6.0));
    items.update(|nodes| nodes.retain(|node| node.id != SOURCE));
    screen.run_until_idle();
    assert_eq!(source.name(), "");
    assert_eq!(amount.name(), "");
    assert!(
        screen
            .query_by_role(Role::Button, "Renamed output Result")
            .is_none()
    );
    screen.get_by_role(Role::Button, "Inserted");
    clean(&screen);
}

#[test]
fn changing_a_node_control_kind_replaces_its_role_and_advertised_actions() {
    let (screen, items) = kivact::test::render_with_handle(crate::options(), || {
        let items = signal(vec![node(SOURCE, "Source", 20.0)]);
        (
            items,
            app_node_editor(items, signal(Vec::new()), ui::NodeState::new())
                .style(style().size(620, 240)),
        )
    });
    screen.get_by_role(Role::SpinButton, "Source Amount");
    items.update(|nodes| {
        nodes[0].controls[0] = ui::NodeControl::toggle("Enabled", 80.0, false);
    });
    screen.run_until_idle();
    assert!(
        screen
            .query_by_role(Role::SpinButton, "Source Amount")
            .is_none()
    );
    let enabled = screen.get_by_role(Role::Button, "Source Enabled");
    assert!(!enabled.a11y_actions().contains(&A11yAction::SetValue));
    assert!(enabled.a11y_actions().contains(&A11yAction::Toggle));
    enabled.a11y(A11yAction::Toggle);
    assert_eq!(enabled.is_checked(), Some(true));
    items.update(|nodes| {
        nodes[0].controls[0] = ui::NodeControl::number("Amount", 80.0, 3.0, 0.0, 10.0, 1.0);
    });
    screen.run_until_idle();
    let amount = screen.get_by_role(Role::SpinButton, "Source Amount");
    assert!(!amount.a11y_actions().contains(&A11yAction::Toggle));
    amount.a11y(A11yAction::Increment);
    assert_eq!(amount.value_range().2, Some(4.0));
    clean(&screen);
}

#[test]
fn hidden_node_editor_hides_its_open_parameter_popup() {
    let (screen, hidden) = kivact::test::render_with_handle(crate::options(), || {
        let hidden = signal(false);
        let mut model = node(1, "Source", 20.0);
        model.controls = vec![ui::NodeControl::choice("Mode", 80.0, ["A", "B"], 0)];
        (
            hidden,
            app_node_editor(
                signal(vec![model]),
                signal(Vec::new()),
                ui::NodeState::new(),
            )
            .hidden(hidden)
            .style(style().size(620, 240)),
        )
    });
    screen
        .get_by_role(Role::Button, "Source Mode")
        .a11y(A11yAction::Activate);
    assert!(screen.get_by_role(Role::Option, "B").is_visible());
    hidden.set(true);
    screen.run_until_idle();
    assert!(
        !screen
            .query_by_role(Role::Option, "B")
            .is_some_and(|element| element.is_visible())
    );
    clean(&screen);
}
#[test]
fn node_fit_contains_a_node_with_a_negative_scene_origin() {
    let (screen, transform) = kivact::test::render_with_handle(crate::options(), || {
        let transform = signal(kivact::interaction::ViewTransform::default());
        (
            transform,
            app_node_editor(
                signal(vec![
                    node(1, "Negative", -240.0).position(Point::new(-240.0, -160.0)),
                ]),
                signal(Vec::new()),
                ui::NodeState {
                    transform: transform.into(),
                    ..ui::NodeState::new()
                },
            )
            .style(style().size(620, 240))
            .test_id("negative nodes"),
        )
    });
    let canvas = screen.get_by_test_id("negative nodes");
    canvas.press_key(Key::Char('0'));
    let surface = canvas.rect().expect("canvas");
    let header = screen
        .get_by_role(Role::Button, "Negative")
        .rect()
        .expect("fitted header");
    assert!(
        header.x >= surface.x
            && header.y >= surface.y
            && header.x + header.width <= surface.x + surface.width + 0.01
    );
    assert!(transform.get_untracked().scale > 1.0);
    clean(&screen);
}
