//! Role: Verify spinbutton step focus and invalid text semantics from APG.
use crate::support::mount;
use kivact::interaction::ScalarRange;
use kivact::prelude::*;
use kivact::ui;

#[test]
fn spinbutton_step_buttons_keep_focus_on_the_editable_field() {
    let (screen, value) = mount(|| {
        let value = signal(2.0);
        (
            value,
            column((
                ui::number_input("Quantity", value, ScalarRange::new(0.0, 10.0).unwrap()),
                ui::button("Next"),
            )),
        )
    });
    let field = screen.get_by_role(Role::SpinButton, "Quantity");
    screen.get_by_role(Role::Button, "Increase").click();
    assert!(field.is_focused());
    assert_eq!(value.get_untracked(), 3.0);
    screen.get_by_role(Role::Button, "Decrease").click();
    assert!(field.is_focused());
    assert_eq!(value.get_untracked(), 2.0);
    field.press_key(Key::Tab);
    assert!(screen.get_by_role(Role::Button, "Next").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn spinbutton_invalid_typed_values_are_exposed_until_corrected_or_blurred() {
    let (screen, value) = mount(|| {
        let value = signal(2.0);
        (
            value,
            column((
                ui::number_input("Quantity", value, ScalarRange::new(0.0, 10.0).unwrap()),
                ui::button("Next"),
            )),
        )
    });
    let field = screen.get_by_role(Role::SpinButton, "Quantity");
    field.click();
    field.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    field.type_text("99");
    assert!(field.is_invalid());
    assert_eq!(value.get_untracked(), 10.0);
    field.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    field.type_text("4");
    assert!(!field.is_invalid());
    assert_eq!(value.get_untracked(), 4.0);
    field.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    field.type_text("bad");
    assert!(field.is_invalid());
    assert_eq!(value.get_untracked(), 4.0);
    field.press_key(Key::Tab);
    assert!(!field.is_invalid());
    assert_eq!(field.value().as_deref(), Some("4"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
