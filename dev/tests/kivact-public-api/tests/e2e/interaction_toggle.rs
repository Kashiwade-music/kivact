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
    assert_eq!(check_box.checked(), Some(CheckedState::Unchecked));
    assert_eq!(switch_node.checked(), Some(CheckedState::Checked));
    assert_eq!(button_node.checked(), Some(CheckedState::Unchecked));
    for node in [&check_box, &switch_node, &button_node] {
        assert_eq!(node.value(), None, "no On/Off text");
        assert_ne!(node.checked(), Some(CheckedState::Indeterminate));
        assert!(!node.is_disabled());
        let actions = node.a11y_actions();
        assert!(actions.contains(&A11yAction::Activate), "{actions:?}");
        assert!(actions.contains(&A11yAction::Toggle), "{actions:?}");
    }
    check.set(true);
    switch.set(false);
    button.set(true);
    screen.run_until_idle();
    assert_eq!(check_box.checked(), Some(CheckedState::Checked));
    assert_eq!(switch_node.checked(), Some(CheckedState::Unchecked));
    assert_eq!(button_node.checked(), Some(CheckedState::Checked));
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
    assert_eq!(button_node.checked(), Some(CheckedState::Unchecked));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The text a parent draws from a part state it reads through the handle.
fn open_text(label: &'static str, open: bool) -> String {
    format!("{label} open {open}")
}

#[test]
fn a_parent_reads_the_disclosure_content_state_across_unmount_and_remount() {
    use kivact::interaction::use_disclosure;
    let (screen, (open, late)) = mount(|| {
        let open = signal(true);
        let late = signal(false);
        let details = use_disclosure(open);
        // Read at setup, before the content first mounts.
        let early = details.content_state();
        (
            (open, late),
            column((
                control("Details").part(details.trigger()),
                show(details.is_open(), move || {
                    control("Body").part(details.content())
                }),
                text(move || open_text("early", early.get().open)),
                // Read for the first time after the content unmounted.
                show(late, move || {
                    let state = details.content_state();
                    text(move || open_text("late", state.get().open))
                }),
            )),
        )
    });
    screen.get_by_text(&open_text("early", true));
    open.set(false);
    screen.run_until_idle();
    screen.get_by_text(&open_text("early", false));
    late.set(true);
    screen.run_until_idle();
    screen.get_by_text(&open_text("late", false));
    open.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Details");
    screen.get_by_text(&open_text("early", true));
    screen.get_by_text(&open_text("late", true));
    open.set(false);
    screen.run_until_idle();
    screen.get_by_text(&open_text("late", false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_parent_reads_the_disclosure_trigger_state_after_the_trigger_unmounts() {
    use kivact::interaction::use_disclosure;
    let (screen, (open, has_trigger)) = mount(|| {
        let open = signal(false);
        let has_trigger = signal(true);
        let details = use_disclosure(open);
        let state = details.trigger_state();
        (
            (open, has_trigger),
            column((
                show(has_trigger, move || {
                    control("Details").part(details.trigger())
                }),
                text(move || open_text("trigger", state.get().open)),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Details").click();
    assert!(open.get_untracked());
    screen.get_by_text(&open_text("trigger", true));
    has_trigger.set(false);
    screen.run_until_idle();
    // Without a trigger the state falls back to the requested state.
    open.set(false);
    screen.run_until_idle();
    screen.get_by_text(&open_text("trigger", false));
    has_trigger.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Details").click();
    screen.get_by_text(&open_text("trigger", true));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_parent_reads_the_popup_content_state_across_closing_and_reopening() {
    use kivact::interaction::use_popup;
    use kivact::view::{Placement, overlay};
    let (screen, open) = mount(|| {
        let open = signal(true);
        let popup = use_popup(open);
        let state = popup.content_state();
        let anchor = node_ref();
        (
            open,
            column((
                control("Menu").node_ref(anchor).part(popup.trigger()),
                show(popup.is_open(), move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        control("Surface").part(popup.content()),
                    )
                }),
                text(move || open_text("surface", state.get().open)),
            )),
        )
    });
    screen.get_by_text(&open_text("surface", true));
    open.set(false);
    screen.run_until_idle();
    screen.get_by_text(&open_text("surface", false));
    open.set(true);
    screen.run_until_idle();
    screen.get_by_text(&open_text("surface", true));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
