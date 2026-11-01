//! Role: Tests for control contracts and binding behavior.
//!
//! Boolean binding, owner lifetime and modifier precedence extracted from
//! the removed component kit and expressed with public primitives.

use crate::support::mount;
use kivact::Binding;
use kivact::binding;
use kivact::prelude::*;
use kivact::test::render;

#[test]
fn boolean_binding_uses_the_same_invoke_for_click_space_and_accessibility() {
    let (screen, state) = mount(|| {
        let state = signal(false);
        let binding: Binding<bool> = state.into();
        (
            state,
            node()
                .role(Role::CheckBox)
                .focusable(true)
                .name("Wi-Fi")
                .style(style().size(120, 32))
                .checked(binding)
                .on_press(move || binding.set(!binding.get_untracked())),
        )
    });
    let checkbox = screen.get_by_role(Role::CheckBox, "Wi-Fi");
    checkbox.click();
    assert_eq!(checkbox.is_checked(), Some(true));
    checkbox.press_key(Key::Space);
    assert!(!state.get_untracked());
    checkbox.toggle();
    assert!(state.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn controlled_boolean_offers_a_write_without_claiming_acceptance() {
    let (screen, asked) = mount(|| {
        let asked = signal(false);
        let binding = binding(true.into_value(), move |value| asked.set(value));
        (
            asked,
            node()
                .role(Role::CheckBox)
                .focusable(true)
                .name("controlled")
                .style(style().size(120, 32))
                .checked(binding)
                .on_press(move || binding.set(!binding.get_untracked())),
        )
    });
    let checkbox = screen.get_by_role(Role::CheckBox, "controlled");
    checkbox.toggle();
    assert!(!asked.get_untracked());
    assert_eq!(checkbox.is_checked(), Some(true));
}

#[test]
fn locally_created_state_belongs_to_each_mounted_row() {
    let (screen, rows) = mount(|| {
        let rows = signal(vec![1u32, 2]);
        (
            rows,
            column(for_each(
                rows,
                |id| *id,
                |id, _| {
                    let state = signal(false);
                    node()
                        .role(Role::CheckBox)
                        .focusable(true)
                        .name(format!("row {id}"))
                        .style(style().size(120, 32))
                        .checked(state)
                        .on_press(move || state.update(|v| *v = !*v))
                },
            )),
        )
    });
    screen.get_by_role(Role::CheckBox, "row 1").toggle();
    assert_eq!(
        screen.get_by_role(Role::CheckBox, "row 2").is_checked(),
        Some(false)
    );
    screen.get_by_role(Role::CheckBox, "row 2").toggle();
    rows.set(vec![2]);
    screen.run_until_idle();
    rows.set(vec![1, 2]);
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_role(Role::CheckBox, "row 1").is_checked(),
        Some(false)
    );
    assert_eq!(
        screen.get_by_role(Role::CheckBox, "row 2").is_checked(),
        Some(true)
    );
}

#[test]
fn later_semantic_modifiers_override_the_authored_defaults() {
    let screen = render(|| {
        node()
            .role(Role::Button)
            .focusable(true)
            .name("original")
            .child(text("original"))
            .role(Role::CheckBox)
            .focusable(false)
            .name("overridden")
    });
    let element = screen.get_by_role(Role::CheckBox, "overridden");
    screen.press_tab(false);
    assert!(!element.is_focused());
    screen.get_by_text("original");
}
