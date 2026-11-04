//! Role: Verify APG radio-group entry follows the checked radio.
use crate::support::mount;
use kivact::interaction::ListItem;
use kivact::prelude::*;

#[test]
fn radio_group_tab_reentry_focuses_the_checked_radio() {
    let (screen, selected) = mount(|| {
        let selected = signal(Some("a"));
        (
            selected,
            column((
                node()
                    .role(Role::Button)
                    .name("Before")
                    .focusable(true)
                    .child(text("Before")),
                radio_group(
                    "Choices",
                    vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Beta")],
                    selected,
                ),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Before").focus();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::RadioButton, "Alpha").is_focused());
    screen.press_tab(true);
    selected.set(Some("b"));
    screen.run_until_idle();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::RadioButton, "Beta").is_focused());
    assert_eq!(
        screen.get_by_role(Role::RadioButton, "Beta").checked(),
        Some(CheckedState::Checked)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
