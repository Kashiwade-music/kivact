//! Role: Tests of the Elements panel.

use std::rc::Rc;

use kivact::prelude::*;
use kivact::test::Screen;
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

/// The button's detail with one base layer: a row that can be overridden (`padding`), a row whose
/// value names a color token (`background` = `accent`), and a read-only row (`border_top_width`).
fn styled_detail() -> serde_json::Value {
    let mut detail =
        serde_json::to_value(support::detail()).expect("invariant: the test detail serializes");
    detail["styles"] = serde_json::json!([{
        "origin": "base",
        "properties": [
            {"name": "padding", "value": "12", "overridden": false},
            {"name": "background", "value": "accent", "overridden": false},
            {"name": "border_top_width", "value": "1", "overridden": false}
        ]
    }]);
    detail
}

/// The fixture with the styled button selected in the Elements panel.
fn open_styled(answers: Vec<(&str, serde_json::Value)>) -> (Rc<Fixture>, Screen) {
    let mut all = vec![("get_node", styled_detail())];
    all.extend(answers);
    let fixture = support::fixture_with(all);
    let screen = support::open(&fixture, "Elements");
    screen
        .get_by_role(Role::TreeItem, "Button \"Save\"")
        .click();
    (fixture, screen)
}

#[test]
fn ticking_a_style_row_sets_an_override_for_this_node() {
    let (fixture, screen) = open_styled(Vec::new());
    screen.get_by_role(Role::CheckBox, "padding").click();
    assert!(called(&fixture, "set_style_override", |params| {
        params
            == &serde_json::json!({
                "scope": {"kind": "node", "node": 2},
                "property": "padding",
                "value": "12"
            })
    }));
}

#[test]
fn committing_an_existing_override_resends_it_and_unticking_clears_its_id() {
    let (fixture, screen) = open_styled(vec![(
        "list_style_overrides",
        serde_json::json!({
            "overrides": [{
                "id": 9,
                "scope": {"kind": "node", "node": 2},
                "property": "padding",
                "value": "12"
            }]
        }),
    )]);
    let input = screen.get_by_role(Role::TextBox, "padding");
    input.press_key(Key::End);
    input.press_key(Key::Backspace);
    input.type_text("6");
    input.press_key(Key::Enter);
    assert!(called(&fixture, "set_style_override", |params| {
        params
            == &serde_json::json!({
                "scope": {"kind": "node", "node": 2},
                "property": "padding",
                "value": "16"
            })
    }));
    screen.get_by_role(Role::CheckBox, "padding").click();
    assert!(called(&fixture, "clear_style_overrides", |params| {
        params == &serde_json::json!({"id": 9})
    }));
}

#[test]
fn the_token_scope_applies_only_to_rows_that_name_a_token() {
    let (fixture, screen) = open_styled(vec![(
        "list_theme_tokens",
        serde_json::json!({
            "tokens": [{"name": "colors.accent", "kind": "color", "value": "#2563eb"}]
        }),
    )]);
    screen.run_until_idle();
    screen.get_by_role(Role::ComboBox, "Override scope").click();
    screen.get_by_role(Role::Option, "Theme token").click();
    assert!(screen.get_by_role(Role::CheckBox, "padding").is_disabled());
    screen.get_by_role(Role::CheckBox, "background").click();
    assert!(called(&fixture, "set_style_override", |params| {
        params
            == &serde_json::json!({
                "scope": {"kind": "token", "token": "colors.accent"},
                "property": "colors.accent",
                "value": "#2563eb"
            })
    }));
}

#[test]
fn recording_a_change_adds_a_review_comment_with_a_screenshot() {
    let (fixture, screen) = open_styled(Vec::new());
    screen
        .get_by_role(Role::Button, "Record change to review")
        .click();
    assert!(called(&fixture, "add_comment", |params| {
        params
            == &serde_json::json!({
                "window": 1,
                "node": 2,
                "text": "Style change",
                "screenshot": true
            })
    }));
}

fn called(fixture: &Rc<Fixture>, method: &str, check: impl Fn(&serde_json::Value) -> bool) -> bool {
    fixture
        .calls
        .borrow()
        .iter()
        .any(|(name, params)| name == method && check(params))
}
