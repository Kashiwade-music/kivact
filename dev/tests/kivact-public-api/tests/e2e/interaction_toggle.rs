//! Role: These tests verify toggle and disclosure interactions.
//!
//! The unstyled `Toggle` and `Disclosure`: roles, keys, click, assistive
//! technology, the mixed state and disabling shared by every check box,
//! switch, toggle button and disclosure of the official kits.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount;
use kivact::binding;
use kivact::control::canvas;
use kivact::draw::{Rect, RegionId};
use kivact::interaction::{
    use_button, use_checkbox, use_disclosure, use_switch, use_toggle_button,
};
use kivact::prelude::*;
use kivact::view::{A11yAction, Handled, KeyEvent};

fn control(label: &str) -> Node {
    node().name(label).style(style().size(120, 30))
}

/// A parent that records the keys no toggle handler consumed.
fn recorded(child: impl IntoView, keys: Rc<RefCell<Vec<Key>>>) -> Node {
    node()
        .on_key(move |event: KeyEvent| {
            keys.borrow_mut().push(event.key);
            Handled::No
        })
        .child(child)
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

#[test]
fn a_check_box_and_a_switch_consume_enter_and_a_toggle_button_lets_it_bubble() {
    let keys = Rc::new(RefCell::new(Vec::new()));
    let seen = keys.clone();
    let (screen, ()) = mount(move || {
        (
            (),
            recorded(
                column((
                    control("Check").part(use_checkbox(signal(false)).control()),
                    control("Button").part(use_toggle_button(signal(false)).control()),
                )),
                seen,
            ),
        )
    });
    screen
        .get_by_role(Role::CheckBox, "Check")
        .press_key(Key::Enter);
    assert!(!keys.borrow().contains(&Key::Enter), "{:?}", keys.borrow());
    screen
        .get_by_role(Role::Button, "Button")
        .press_key(Key::Enter);
    assert!(keys.borrow().contains(&Key::Enter), "{:?}", keys.borrow());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_click_and_both_accessibility_actions_flip_every_kind() {
    let (screen, values) = mount(|| {
        let values = [signal(false), signal(false), signal(false)];
        (
            values,
            column((
                control("Check").part(use_checkbox(values[0]).control()),
                control("Switch").part(use_switch(values[1]).control()),
                control("Button").part(use_toggle_button(values[2]).control()),
            )),
        )
    });
    let nodes = [
        screen.get_by_role(Role::CheckBox, "Check"),
        screen.get_by_role(Role::Switch, "Switch"),
        screen.get_by_role(Role::Button, "Button"),
    ];
    let read = || values.map(|v| v.get_untracked());
    for node in &nodes {
        node.click();
    }
    assert_eq!(read(), [true; 3], "click");
    for node in &nodes {
        node.a11y(A11yAction::Activate);
    }
    assert_eq!(read(), [false; 3], "Activate");
    for node in &nodes {
        node.a11y(A11yAction::Toggle);
    }
    assert_eq!(read(), [true; 3], "Toggle");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_mixed_check_box_publishes_the_state_and_a_press_checks_it_and_lets_the_caller_clear_it() {
    let (screen, (checked, mixed, writes)) = mount(|| {
        let checked = signal(false);
        let mixed = signal(true);
        let writes = Rc::new(RefCell::new(Vec::new()));
        let log = writes.clone();
        let binding = binding(checked.into_value(), move |on: bool| {
            log.borrow_mut().push(on);
            mixed.set(false);
            checked.set(on);
        });
        (
            (checked, mixed, writes),
            control("All").part(use_checkbox(binding).indeterminate(mixed).control()),
        )
    });
    let all = screen.get_by_role(Role::CheckBox, "All");
    assert!(all.is_indeterminate());
    assert_eq!(all.is_checked(), Some(false), "the binary value is kept");
    assert!(screen.accessibility_tree().contains("indeterminate"));

    all.click();
    assert_eq!(*writes.borrow(), vec![true]);
    assert!(checked.get_untracked());
    assert!(!all.is_indeterminate(), "the caller cleared it");
    assert!(!screen.accessibility_tree().contains("indeterminate"));

    mixed.set(true);
    screen.run_until_idle();
    assert!(all.is_indeterminate());
    assert_eq!(all.is_checked(), Some(true));
    all.press_key(Key::Space);
    assert_eq!(
        *writes.borrow(),
        vec![true, true],
        "a mixed press writes true even when checked is already true"
    );
    assert!(!all.is_indeterminate());

    all.click();
    assert_eq!(*writes.borrow(), vec![true, true, false]);
    assert!(!checked.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn only_a_check_box_has_a_mixed_state() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                control("Switch").part(use_switch(signal(false)).indeterminate(true).control()),
                control("Button").part(
                    use_toggle_button(signal(false))
                        .indeterminate(true)
                        .control(),
                ),
            )),
        )
    });
    assert!(
        !screen
            .get_by_role(Role::Switch, "Switch")
            .is_indeterminate()
    );
    assert!(
        !screen
            .get_by_role(Role::Button, "Button")
            .is_indeterminate()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_disabled_toggle_ignores_input_and_assistive_technology() {
    let (screen, (value, disabled)) = mount(|| {
        let value = signal(false);
        let disabled = signal(true);
        (
            (value, disabled),
            control("Gated").part(use_toggle_button(value).disabled(disabled).control()),
        )
    });
    let node = screen.get_by_role(Role::Button, "Gated");
    assert!(node.is_disabled());
    node.force_click();
    node.a11y(A11yAction::Activate);
    node.a11y(A11yAction::Toggle);
    screen.press_key(Key::Space);
    screen.press_key(Key::Enter);
    assert!(!value.get_untracked(), "disabled: no click, AT or keys");

    disabled.set(false);
    screen.run_until_idle();
    assert!(!node.is_disabled());
    node.click();
    assert!(value.get_untracked());
    node.press_key(Key::Enter);
    assert!(!value.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_disclosure_trigger_is_a_button_that_publishes_expanded() {
    let (screen, open) = mount(|| {
        let open = signal(false);
        let details = use_disclosure(open);
        (
            open,
            column((
                control("Details").part(details.trigger()),
                show(details.shown(), move || {
                    node()
                        .part(details.content())
                        .role(Role::Group)
                        .name("Details panel")
                        .child(text("The details"))
                }),
            )),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Details");
    assert_eq!(trigger.is_expanded(), Some(false));
    assert_eq!(trigger.is_checked(), None, "expanded, not checked");
    assert!(!screen.accessibility_tree().contains("The details"));
    assert!(trigger.controls().is_empty(), "no content mounted");

    trigger.click();
    assert!(open.get_untracked());
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(screen.get_by_text("The details").exists());
    let controlled: Vec<String> = trigger.controls().iter().map(|e| e.name()).collect();
    assert_eq!(
        controlled,
        ["Details panel"],
        "the trigger controls its content"
    );
    trigger.press_key(Key::Enter);
    assert!(!open.get_untracked(), "Enter");
    trigger.press_key(Key::Space);
    assert!(open.get_untracked(), "Space");
    trigger.a11y(A11yAction::Activate);
    assert!(!open.get_untracked(), "Activate");
    trigger.a11y(A11yAction::Toggle);
    assert!(open.get_untracked(), "Toggle");
    trigger.a11y(A11yAction::Collapse);
    assert!(!open.get_untracked(), "Collapse");
    trigger.a11y(A11yAction::Collapse);
    assert!(!open.get_untracked(), "Collapse is not a flip");
    trigger.a11y(A11yAction::Expand);
    trigger.a11y(A11yAction::Expand);
    assert!(open.get_untracked(), "Expand is not a flip");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_disabled_disclosure_ignores_input_and_assistive_technology() {
    let (screen, (open, disabled)) = mount(|| {
        let open = signal(false);
        let disabled = signal(true);
        (
            (open, disabled),
            control("Locked").part(use_disclosure(open).disabled(disabled).trigger()),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Locked");
    assert!(trigger.is_disabled());
    trigger.force_click();
    trigger.a11y(A11yAction::Activate);
    trigger.a11y(A11yAction::Toggle);
    trigger.a11y(A11yAction::Expand);
    screen.press_key(Key::Enter);
    screen.press_key(Key::Space);
    assert!(!open.get_untracked());

    disabled.set(false);
    screen.run_until_idle();
    trigger.click();
    assert!(open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn parts_share_button_behavior_and_isolate_canvas_toggle_targets() {
    use kivact::test::{RenderOptions, WindowMode};
    use kivact::view::{KeyCode, Modifiers};

    let (screen, (first, second, first_state, second_state, root_state, presses)) =
        crate::support::mount_with(
            RenderOptions {
                window: WindowMode::Real,
                ..RenderOptions::from_env()
            },
            || {
                let first = signal(false);
                let second = signal(false);
                let presses = signal(0);
                let one = use_checkbox(first).control().name("First region");
                let two = use_checkbox(second).control().name("Second region");
                let root = use_button().control();
                let states = (one.state(), two.state(), root.state());
                let regions = canvas(|draw, _| {
                    draw.region(RegionId(101), Rect::new(0.0, 0.0, 90.0, 50.0));
                    draw.region(RegionId(102), Rect::new(100.0, 0.0, 90.0, 50.0));
                })
                .part(root)
                .name("Canvas button")
                .on_press(move || presses.update(|value| *value += 1))
                .region(RegionId(101), one)
                .region(RegionId(102), two)
                .style(style().size(200, 50));
                (
                    (first, second, states.0, states.1, states.2, presses),
                    regions,
                )
            },
        );
    let one = screen.get_by_role(Role::CheckBox, "First region");
    let two = screen.get_by_role(Role::CheckBox, "Second region");
    one.a11y(A11yAction::Activate);
    assert!(first.get());
    assert!(!second.get());
    assert_eq!(presses.get(), 0);
    two.a11y(A11yAction::Toggle);
    assert!(second.get());
    one.click();
    assert!(!first.get());
    assert!(first_state.get().input.focused);
    assert!(!second_state.get().input.focused);
    assert!(!root_state.get().focused);
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    assert!(first_state.get().input.pressed);
    assert!(!second_state.get().input.pressed);
    assert!(!root_state.get().pressed);
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    assert!(!first_state.get().input.pressed);
    screen
        .get_by_role(Role::Button, "Canvas button")
        .a11y(A11yAction::Activate);
    assert_eq!(presses.get(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
