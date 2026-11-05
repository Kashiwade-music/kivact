//! Role: Cover APG Breadcrumb navigation landmarks and current-page semantics.
//! Audit: https://www.w3.org/WAI/ARIA/apg/patterns/breadcrumb/
//! Breadcrumbs have no special keyboard interaction; links use native activation.
use crate::support::mount;
use kivact::prelude::*;
use kivact::view::CurrentItem;

#[test]
fn breadcrumb_links_activate_with_the_keyboard_and_the_page_is_not_a_tab_stop() {
    let (screen, selected) = mount(|| {
        let selected = signal(0);
        (
            selected,
            column((
                breadcrumb(vec![ListItem::new(1, "Home"), ListItem::new(2, "Settings")])
                    .on_activate(move |key| selected.set(key)),
                button("After trail"),
            )),
        )
    });
    assert!(
        screen
            .get_by_role(Role::Navigation, "Breadcrumb")
            .is_visible()
    );
    let home = screen.get_by_role(Role::Link, "Home");
    home.focus();
    home.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), 1);
    home.press_key(Key::Tab);
    assert!(screen.get_by_role(Role::Button, "After trail").is_focused());
    assert_eq!(
        screen.get_by_role(Role::Text, "Settings").current_item(),
        Some(CurrentItem::Page)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
