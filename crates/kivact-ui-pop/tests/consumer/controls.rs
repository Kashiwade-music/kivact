//! Role: Buttons, checks, switches, toggles, accordion and slider through the public API.

use kivact::draw::Point;
use kivact::prelude::*;
use kivact::view::A11yAction;

#[test]
fn button_states_and_user_modifiers_use_public_contract() {
    let (screen, count) = kivact::test::render_with_handle(crate::support::options(), || {
        let count = signal(0);
        (
            count,
            column((
                kivact::ui::button("Save")
                    .on_press(move || count.update(|v| *v += 1))
                    .style(style().height(40))
                    .test_id("save"),
                kivact::ui::button("Disabled")
                    .disabled(true)
                    .on_press(move || count.update(|v| *v += 100)),
            )),
        )
    });
    let button = screen.get_by_role(Role::Button, "Save");
    button.click();
    button.press_key(Key::Enter);
    assert_eq!(count.get_untracked(), 2);
    assert_eq!(button.rect().map(|r| r.height), Some(40.0));
    assert!(screen.get_by_role(Role::Button, "Disabled").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn checkbox_switch_and_mixed_states_are_accessible() {
    let (screen, (checked, on)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let checked = signal(false);
            let on = signal(true);
            (
                (checked, on),
                column((
                    kivact::ui::checkbox("Reports", checked),
                    kivact::ui::checkbox("Partial", kivact::signal(false)).indeterminate(true),
                    kivact::ui::switch("Sync", on),
                )),
            )
        });
    let check = screen.get_by_role(Role::CheckBox, "Reports");
    check.press_key(Key::Space);
    assert!(checked.get_untracked());
    assert!(
        screen
            .get_by_role(Role::CheckBox, "Partial")
            .is_indeterminate()
    );
    screen.get_by_role(Role::Switch, "Sync").click();
    assert!(!on.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn toggles_and_accordion_follow_the_shared_toggle_rules() {
    let (screen, (check, switch, bold, open)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let check = signal(false);
            let switch = signal(false);
            let bold = signal(false);
            let open = signal(false);
            (
                (check, switch, bold, open),
                column((
                    kivact::ui::checkbox("Reports", check),
                    kivact::ui::switch("Sync", switch),
                    kivact::ui::toggle("Bold", bold),
                    kivact::ui::disclosure("Details", open, || text("Body")),
                    kivact::ui::checkbox("Locked", kivact::signal(false)).disabled(true),
                )),
            )
        });
    let check_box = screen.get_by_role(Role::CheckBox, "Reports");
    let switch_node = screen.get_by_role(Role::Switch, "Sync");
    let button = screen.get_by_role(Role::Button, "Bold");
    check_box.press_key(Key::Enter);
    switch_node.press_key(Key::Enter);
    assert!(!check.get_untracked() && !switch.get_untracked());
    button.press_key(Key::Enter);
    assert!(bold.get_untracked(), "Enter flips a toggle button");
    assert_eq!(button.is_checked(), Some(true));
    for node in [&check_box, &switch_node, &button] {
        node.press_key(Key::Space);
        assert_eq!(node.value(), None, "role and checked, no On/Off text");
    }
    assert!(check.get_untracked() && switch.get_untracked() && !bold.get_untracked());
    check_box.a11y(A11yAction::Toggle);
    switch_node.a11y(A11yAction::Activate);
    assert!(!check.get_untracked() && !switch.get_untracked());
    let locked = screen.get_by_role(Role::CheckBox, "Locked");
    assert!(locked.is_disabled());
    locked.force_click();
    locked.a11y(A11yAction::Toggle);
    assert_eq!(locked.is_checked(), Some(false));

    let details = screen.get_by_role(Role::Button, "Details");
    assert_eq!(details.is_expanded(), Some(false));
    details.press_key(Key::Enter);
    assert!(open.get_untracked());
    details.press_key(Key::Space);
    assert!(!open.get_untracked());
    details.a11y(A11yAction::Toggle);
    assert_eq!(details.is_expanded(), Some(true));
    details.a11y(A11yAction::Collapse);
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn slider_drag_keyboard_and_accessibility_share_one_binding() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), || {
        let value = signal(30.0f64);
        (
            value,
            kivact::ui::slider("Volume", value)
                .range(
                    kivact::ui::ScalarRange::new(0.0, 100.0)
                        .and_then(|range| range.with_step(1.0))
                        .expect("finite ordered range"),
                )
                .test_id("volume")
                .style(style().width(300)),
        )
    });
    let slider = screen.get_by_role(Role::Slider, "Volume");
    slider.press_key(Key::ArrowRight);
    assert_eq!(value.get_untracked(), 31.0);
    slider.a11y(A11yAction::Increment);
    assert_eq!(value.get_untracked(), 32.0);
    slider.a11y_with_value(A11yAction::SetValue, "37.5");
    assert_eq!(value.get_untracked(), 38.0);
    slider.a11y_with_value(A11yAction::SetValue, "NaN");
    assert_eq!(value.get_untracked(), 38.0);
    slider.a11y_with_value(A11yAction::SetValue, "-1000");
    assert_eq!(value.get_untracked(), 0.0);
    screen
        .get_by_test_id("volume")
        .drag(Point::new(40.0, 10.0), Point::new(220.0, 10.0));
    assert!(value.get_untracked() > 60.0);
    assert_eq!(slider.value_range().2, Some(value.get_untracked()));
    slider.press_key(Key::End);
    assert_eq!(value.get_untracked(), 100.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
