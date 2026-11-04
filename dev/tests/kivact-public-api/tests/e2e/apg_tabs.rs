//! Role: Verify APG tab entry after manual focus navigation.
use crate::support::mount;
use kivact::interaction::{ListActivation, ListItem};
use kivact::prelude::*;

#[test]
fn tabs_tab_reentry_focuses_the_selected_tab() {
    let (screen, selected) = mount(|| {
        let selected = signal("a");
        (
            selected,
            column((
                node()
                    .role(Role::Button)
                    .name("Before")
                    .focusable(true)
                    .child(text("Before")),
                tabs(
                    "Sections",
                    vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Beta")],
                    selected,
                )
                .activation(ListActivation::Manual),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Before").focus();
    screen.press_tab(false);
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::Tab, "Beta").is_focused());
    assert_eq!(selected.get_untracked(), "a");
    screen.press_tab(true);
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Tab, "Alpha").is_focused());
    assert_eq!(selected.get_untracked(), "a");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
