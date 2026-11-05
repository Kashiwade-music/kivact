//! Role: Verify each required APG accordion keyboard and expansion difference.
// Source: https://www.w3.org/WAI/ARIA/apg/patterns/accordion/
use crate::support::mount;
use kivact::prelude::*;

#[test]
fn accordion_enter_and_space_toggle_panels_and_publish_expanded() {
    let (screen, ()) = mount(|| {
        (
            (),
            kivact::ui::disclosure_group(signal(Vec::<u8>::new()))
                .panel(1, "First", || button("Inside"))
                .panel(2, "Second", || text("Second body")),
        )
    });
    let first = screen.get_by_role(Role::Button, "First");
    first.focus();
    screen.press_key(Key::Enter);
    assert_eq!(first.is_expanded(), Some(true));
    screen.press_key(Key::Space);
    assert_eq!(first.is_expanded(), Some(false));
    screen.press_key(Key::Space);
    screen.get_by_role(Role::Button, "Second").click();
    assert_eq!(
        first.is_expanded(),
        Some(true),
        "multiple selection preserves another expanded panel"
    );
    assert_eq!(
        screen.get_by_role(Role::Button, "Second").is_expanded(),
        Some(true)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn accordion_tab_sequence_includes_headers_and_expanded_panel_controls() {
    let (screen, ()) = mount(|| {
        (
            (),
            kivact::ui::disclosure_group(signal(Some(1)))
                .panel(1, "First", || button("Inside"))
                .panel(2, "Second", || text("Second body")),
        )
    });
    screen.get_by_role(Role::Button, "First").focus();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "Inside").is_focused());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "Second").is_focused());
    screen.press_tab(true);
    assert!(screen.get_by_role(Role::Button, "Inside").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
