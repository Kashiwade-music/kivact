//! Role: Tests that a node reference activates the control at or below it.
//!
//! `NodeRef::activate` on real windows: pressing a label activates its
//! check box (focus and toggle), a reference to a wrapper reaches the first
//! control inside it (a text field takes only the focus, a button is
//! pressed), and a disabled control, an unbound reference and a control
//! behind an open modal are left alone without an error.

use crate::support::mount;
use kivact::prelude::*;
use kivact::ui::{button, checkbox, text_input};
use kivact::view::{Placement, overlay};

#[test]
fn pressing_a_label_focuses_and_toggles_its_check_box() {
    let (screen, agreed) = mount(|| {
        let agreed = signal(false);
        let target = node_ref();
        let view = column((
            text("I agree to the terms")
                .on_press(move || target.activate())
                .test_id("label"),
            checkbox("Agree", agreed).hide_label(true).node_ref(target),
        ));
        (agreed, view)
    });
    screen.get_by_test_id("label").click();
    assert!(agreed.get_untracked(), "the label toggled the check box");
    let check = screen.get_by_role(Role::CheckBox, "Agree");
    assert_eq!(check.is_checked(), Some(true));
    assert!(check.is_focused(), "the check box took the focus");
    screen.get_by_test_id("label").click();
    assert!(!agreed.get_untracked(), "a second press toggles it back");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_reference_to_a_wrapper_activates_the_first_control_inside_it() {
    let (screen, (name, presses)) = mount(|| {
        let name = signal(String::new());
        let presses = signal(0u32);
        let field = node_ref();
        let actions = node_ref();
        let view = column((
            text("Name")
                .on_press(move || field.activate())
                .test_id("name"),
            node()
                .node_ref(field)
                .child(column((text("decoration"), text_input("Name", name)))),
            text("Run")
                .on_press(move || actions.activate())
                .test_id("run"),
            row((
                text("not a control"),
                button("Go").on_press(move || presses.update(|n| *n += 1)),
                button("Second").on_press(move || presses.update(|n| *n += 100)),
            ))
            .node_ref(actions),
        ));
        ((name, presses), view)
    });
    screen.get_by_test_id("name").click();
    let input = screen.get_by_role(Role::TextBox, "Name");
    assert!(input.is_focused(), "the text field took the focus");
    screen.type_text("Ada");
    assert_eq!(name.get_untracked(), "Ada");
    screen.get_by_test_id("run").click();
    assert_eq!(
        presses.get_untracked(),
        1,
        "only the first button is pressed"
    );
    assert!(screen.get_by_role(Role::Button, "Go").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_disabled_control_an_unbound_reference_and_a_control_behind_a_modal_stay_untouched() {
    let (screen, (checked, open)) = mount(|| {
        let checked = signal(false);
        let open = signal(false);
        let disabled = node_ref();
        let unbound = node_ref();
        let blocked = node_ref();
        let anchor = node_ref();
        let view = column((
            text("Disabled")
                .on_press(move || disabled.activate())
                .test_id("disabled"),
            checkbox("Off", checked).disabled(true).node_ref(disabled),
            text("Unbound")
                .on_press(move || unbound.activate())
                .test_id("unbound"),
            checkbox("Behind", checked).node_ref(blocked),
            node().node_ref(anchor),
            show(open, move || {
                overlay(
                    anchor,
                    Placement::Center,
                    text("Inside")
                        .on_press(move || blocked.activate())
                        .test_id("inside"),
                )
                .modal(true)
            }),
        ));
        ((checked, open), view)
    });
    screen.get_by_test_id("disabled").click();
    screen.get_by_test_id("unbound").click();
    assert!(!checked.get_untracked());
    open.set(true);
    screen.run_until_idle();
    screen.get_by_test_id("inside").click();
    assert!(!checked.get_untracked(), "the check box behind the modal");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
