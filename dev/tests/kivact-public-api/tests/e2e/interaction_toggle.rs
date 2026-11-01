//! Role: These tests verify toggle and disclosure interactions.
//!
//! The unstyled `Toggle` and `Disclosure`: roles, keys, click, assistive
//! technology, the mixed state and disabling shared by every check box,
//! switch, toggle button and disclosure of the official kits.

use crate::support::mount;
use kivact::interaction::{use_checkbox, use_switch, use_toggle_button};
use kivact::prelude::*;
use kivact::view::A11yAction;

fn control(label: &str) -> Node {
    node().name(label).style(style().size(120, 30))
}

#[test]
fn each_kind_publishes_its_role_and_the_checked_state_without_a_value_text() {
    let (screen, (check, switch, button)) = mount(|| {
        let check = signal(false);
        let switch = signal(true);
        let button = signal(false);
        (
            (check, switch, button),
            column((
                control("Check").part(use_checkbox(check).control()),
                control("Switch").part(use_switch(switch).control()),
                control("Button").part(use_toggle_button(button).control()),
            )),
        )
    });
    let check_box = screen.get_by_role(Role::CheckBox, "Check");
    let switch_node = screen.get_by_role(Role::Switch, "Switch");
    let button_node = screen.get_by_role(Role::Button, "Button");
    assert_eq!(check_box.is_checked(), Some(false));
    assert_eq!(switch_node.is_checked(), Some(true));
    assert_eq!(button_node.is_checked(), Some(false));
    for node in [&check_box, &switch_node, &button_node] {
        assert_eq!(node.value(), None, "no On/Off text");
        assert!(!node.is_indeterminate());
        assert!(!node.is_disabled());
        let actions = node.a11y_actions();
        assert!(actions.contains(&A11yAction::Activate), "{actions:?}");
        assert!(actions.contains(&A11yAction::Toggle), "{actions:?}");
    }
    check.set(true);
    switch.set(false);
    button.set(true);
    screen.run_until_idle();
    assert_eq!(check_box.is_checked(), Some(true));
    assert_eq!(switch_node.is_checked(), Some(false));
    assert_eq!(button_node.is_checked(), Some(true));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn space_flips_every_kind_and_only_a_toggle_button_flips_on_enter() {
    let (screen, (check, switch, button)) = mount(|| {
        let check = signal(false);
        let switch = signal(false);
        let button = signal(false);
        (
            (check, switch, button),
            column((
                control("Check").part(use_checkbox(check).control()),
                control("Switch").part(use_switch(switch).control()),
                control("Button").part(use_toggle_button(button).control()),
            )),
        )
    });
    let check_box = screen.get_by_role(Role::CheckBox, "Check");
    let switch_node = screen.get_by_role(Role::Switch, "Switch");
    let button_node = screen.get_by_role(Role::Button, "Button");

    check_box.press_key(Key::Space);
    switch_node.press_key(Key::Space);
    button_node.press_key(Key::Space);
    assert_eq!(
        (
            check.get_untracked(),
            switch.get_untracked(),
            button.get_untracked()
        ),
        (true, true, true),
        "Space flips all three"
    );

    check_box.press_key(Key::Enter);
    switch_node.press_key(Key::Enter);
    assert_eq!(
        (check.get_untracked(), switch.get_untracked()),
        (true, true),
        "Enter does not flip a check box or a switch"
    );
    button_node.press_key(Key::Enter);
    assert!(!button.get_untracked(), "Enter flips a toggle button");

    check_box.press_key(Key::Space);
    switch_node.press_key(Key::Space);
    assert!(!check.get_untracked());
    assert!(!switch.get_untracked());
    assert_eq!(button_node.is_checked(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
