//! Role: Tests for control checked states and accessibility.
//!
//! Mixed controls retain their application's binary value and publish the
//! three-state accessibility value through the same public modifiers.

use crate::support::mount;
use kivact::prelude::*;
use kivact::primitive::NodeExt;

#[test]
fn indeterminate_checked_state_is_reactive_and_independent_of_binary_value() {
    let (screen, (checked, mixed)) = mount(|| {
        let checked = signal(false);
        let mixed = signal(true);
        let control = node()
            .role(Role::CheckBox)
            .name("Select all rows")
            .checked(checked)
            .indeterminate(mixed)
            .focusable(true)
            .style(style().size(160, 36))
            .on_press(move || {
                mixed.set(false);
                checked.update(|value| *value = !*value);
            });
        ((checked, mixed), control)
    });
    let control = screen.get_by_role(Role::CheckBox, "Select all rows");
    assert_eq!(control.is_checked(), Some(false));
    assert!(control.is_indeterminate());
    assert!(screen.accessibility_tree().contains("indeterminate"));
    control.click();
    assert_eq!(control.is_checked(), Some(true));
    assert!(!control.is_indeterminate());
    mixed.set(true);
    screen.run_until_idle();
    assert!(control.is_indeterminate());
    assert_eq!(control.is_checked(), Some(true));
    mixed.set(false);
    checked.set(false);
    screen.run_until_idle();
    assert!(!control.is_indeterminate());
    assert_eq!(control.is_checked(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    assert!(!screen.accessibility_tree().contains("indeterminate"));
}
