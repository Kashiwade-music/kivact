//! Role: Tests of the Elements panel.

use std::rc::Rc;

use kivact::prelude::*;
use kivact_devtools::source::Fixture;

use crate::support;

#[test]
fn selecting_a_node_inspects_and_highlights_it() {
    let fixture = support::fixture_with(Vec::new());
    let screen = support::open(&fixture, "Elements");
    let row = screen.get_by_role(Role::TreeItem, "Button \"Save\"");
    row.click();
    assert!(called(&fixture, "get_node", |params| params["node"] == 2));
    assert!(called(&fixture, "highlight", |params| {
        params["node"] == 2 && params["layout"] == false
    }));
    screen.within(row).get_by_text("3");
    screen
        .within(row)
        .get_by_role(Role::Link, "src/main.rs:10:5");
}

#[test]
fn the_layout_tab_highlights_with_the_layout_boxes() {
    let fixture = support::fixture_with(Vec::new());
    let screen = support::open(&fixture, "Elements");
    screen
        .get_by_role(Role::TreeItem, "Button \"Save\"")
        .click();
    screen.get_by_role(Role::Tab, "Layout").click();
    screen
        .get_by_role(Role::CheckBox, "Show layout overlay")
        .click();
    assert!(called(&fixture, "highlight", |params| {
        params["node"] == 2 && params["layout"] == true
    }));
}

#[test]
fn pick_starts_from_the_button_and_the_shortcut() {
    let fixture = support::fixture_with(Vec::new());
    let screen = support::open(&fixture, "Elements");
    screen.get_by_role(Role::Button, "Pick element").click();
    assert!(called(&fixture, "start_pick", |params| params["window"] == 1));
    fixture.calls.borrow_mut().clear();
    screen
        .get_by_role(Role::TreeItem, "Button \"Save\"")
        .click();
    screen.press_key_with(
        Key::Char('C'),
        Modifiers {
            ctrl: true,
            shift: true,
            ..Modifiers::default()
        },
    );
    assert!(called(&fixture, "start_pick", |params| params["window"] == 1));
}

#[test]
fn owners_replace_the_tree_and_select_the_first_node() {
    let fixture = support::fixture_with(vec![(
        "get_owners",
        serde_json::json!({
            "roots": [{
                "id": 10,
                "label": "Root owner",
                "location": {"file": "src/lib.rs", "line": 3, "column": 1},
                "nodes": [2],
                "children": [{
                    "id": 11,
                    "label": "Child owner",
                    "nodes": [3]
                }]
            }]
        }),
    )]);
    let screen = support::open(&fixture, "Elements");
    screen.get_by_role(Role::CheckBox, "Owners").click();
    assert!(called(&fixture, "get_owners", |params| params["window"] == 1));
    screen.get_by_role(Role::TreeItem, "Child owner");
    screen.get_by_role(Role::TreeItem, "Root owner").click();
    assert!(called(&fixture, "get_node", |params| params["node"] == 2));
}

fn called(fixture: &Rc<Fixture>, method: &str, check: impl Fn(&serde_json::Value) -> bool) -> bool {
    fixture
        .calls
        .borrow()
        .iter()
        .any(|(name, params)| name == method && check(params))
}
