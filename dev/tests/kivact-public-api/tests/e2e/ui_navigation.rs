//! Role: Verify core pagination, stepper, breadcrumb and exclusive navigation behavior.
use crate::support::mount;
use kivact::prelude::*;
use kivact::ui::{PaginationMode, breadcrumb, navigation_menu, pagination, stepper};
use kivact::view::CurrentItem;
use std::time::Duration;

#[test]
fn pagination_moves_pages_and_keeps_focus() {
    let (screen, page) = mount(|| {
        let page = signal(0usize);
        (page, pagination(20usize, page))
    });
    let next = screen.get_by_role(Role::Button, "Next page");
    next.click();
    assert_eq!(page.get_untracked(), 1);
    assert!(next.is_focused());
    next.press_key(Key::Enter);
    assert_eq!(page.get_untracked(), 2);
    screen.get_by_role(Role::Button, "4").click();
    assert_eq!(page.get_untracked(), 3);
    assert_eq!(
        screen.get_by_role(Role::Button, "4").current_item(),
        Some(CurrentItem::Page)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn stepper_marks_the_current_step() {
    let (screen, (current, width)) = mount(|| {
        let current = signal(1usize);
        let width = signal(600.0);
        (
            (current, width),
            stepper(
                vec![
                    StepperItem::new("Account"),
                    StepperItem::new("Address").description("Delivery address"),
                    StepperItem::new("Confirm"),
                ],
                current,
            )
            .style(move || style().width(width.get())),
        )
    });
    assert_eq!(
        screen.get_by_role(Role::ListItem, "Address").current_item(),
        Some(CurrentItem::Step)
    );
    current.set(2);
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_role(Role::ListItem, "Address").current_item(),
        None
    );
    assert_eq!(
        screen.get_by_role(Role::ListItem, "Confirm").current_item(),
        Some(CurrentItem::Step)
    );
    width.set(180.0);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::ListItem, "Account").is_none());
    assert!(screen.get_by_role(Role::ListItem, "Confirm").is_visible());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn breadcrumb_marks_the_current_page() {
    let (screen, activated) = mount(|| {
        let activated = signal(None);
        (
            activated,
            breadcrumb(vec![
                ListItem::new(0, "Home"),
                ListItem::new(1, "Guide"),
                ListItem::new(2, "Navigation"),
            ])
            .on_activate(move |key| activated.set(Some(key))),
        )
    });
    assert_eq!(
        screen.get_by_role(Role::Text, "Navigation").current_item(),
        Some(CurrentItem::Page)
    );
    screen.get_by_role(Role::Link, "Guide").click();
    assert_eq!(activated.get_untracked(), Some(1));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn navigation_menu_opens_on_hover_and_pins_on_click() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                navigation_menu("Main")
                    .link("docs", "Docs", || button("Read guide"))
                    .link("about", "About", || text("About content")),
                button("Outside"),
            )),
        )
    });
    let docs = screen.get_by_role(Role::Button, "Docs");
    docs.hover();
    assert!(screen.get_by_role(Role::Button, "Read guide").is_visible());
    docs.click();
    screen.get_by_role(Role::Button, "Outside").hover();
    screen.advance(Duration::from_millis(200));
    assert!(screen.get_by_role(Role::Button, "Read guide").is_visible());
    docs.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Button, "Read guide").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pagination_input_commits_and_clamps_the_one_based_draft() {
    let (screen, page) = mount(|| {
        let page = signal(0usize);
        (page, pagination(4usize, page).mode(PaginationMode::Input))
    });
    let field = screen.get_by_role(Role::TextBox, "Page");
    field.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    field.type_text("99");
    field.press_key(Key::Enter);
    assert_eq!(page.get_untracked(), 3);
    assert_eq!(field.value().as_deref(), Some("4"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn navigation_actions_and_rich_panel_links_share_the_existing_close_owner() {
    let (screen, clicks) = mount(|| {
        let clicks = signal(0usize);
        (
            clicks,
            navigation_menu("Site")
                .link("resources", "Resources", move || {
                    kivact::ui::navigation_menu_link("Guide")
                        .description("Read the guide")
                        .icon(icon("chevron-right"))
                        .on_press(move || clicks.update(|n| *n += 1))
                })
                .action("home", "Home", move || clicks.update(|n| *n += 10)),
        )
    });
    screen.get_by_role(Role::Button, "Resources").click();
    screen.get_by_role(Role::Link, "Guide").click();
    assert_eq!(clicks.get_untracked(), 1);
    assert!(screen.query_by_role(Role::Link, "Guide").is_none());
    screen.get_by_role(Role::Link, "Home").press_key(Key::Enter);
    assert_eq!(clicks.get_untracked(), 11);
    assert!(screen.query_by_role(Role::Link, "Guide").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
