//! Role: Verify the sidebar's widths, its trigger, its shortcut and its rail.

use kivact::prelude::*;
use kivact::test::Screen;
use kivact::ui::{
    SidebarCollapsible, SidebarFrame, SidebarSide, sidebar, sidebar_group, sidebar_menu,
    sidebar_menu_button, sidebar_menu_sub_button, sidebar_provider, sidebar_trigger,
};
use kivact::view::Modifiers;

fn app_sidebar() -> impl IntoView {
    sidebar()
        .rail(true)
        .test_id("sidebar")
        .header(text("Acme Inc"))
        .content(sidebar_group("Application").content(sidebar_menu(vec![
                sidebar_menu_button("Home")
                    .icon(kivact::view::icon("check"))
                    .active(true)
                    .test_id("home")
                    .into_view(),
                sidebar_menu_button("Inbox")
                    .icon(kivact::view::icon("copy"))
                    .badge("12")
                    .test_id("inbox")
                    .into_view(),
                sidebar_menu_button("Settings")
                    .icon(kivact::view::icon("calendar"))
                    .sub(column((
                        sidebar_menu_sub_button("Profile").active(true),
                        sidebar_menu_sub_button("Billing"),
                    )))
                    .test_id("settings")
                    .into_view(),
            ])))
        .footer(sidebar_menu_button("Account").icon(kivact::view::icon("dot")))
}

fn mount_sidebar() -> (Screen, Signal<bool>) {
    kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(Theme::light())
            .window_size(900.0, 700.0)
            .reduced_motion(),
        move || {
            let open = signal(true);
            (
                open,
                column(
                    sidebar_provider(
                        open,
                        app_sidebar().into_view(),
                        column((sidebar_trigger().test_id("trigger"), text("Page content")))
                            .style(style().padding(8).gap(8))
                            .into_view(),
                    )
                    .collapsible(SidebarCollapsible::Icon)
                    .frame(SidebarFrame::Sidebar)
                    .side(SidebarSide::Left)
                    .test_id("provider"),
                )
                .style(style().width(800).height(500)),
            )
        },
    )
}

fn width_of(screen: &Screen, id: &str) -> f64 {
    screen.get_by_test_id(id).rect().expect("laid out").width
}

#[test]
fn sidebar_toggles_by_trigger_shortcut_and_rail_between_256_and_48() {
    let (screen, open) = mount_sidebar();
    assert_eq!(width_of(&screen, "sidebar"), 256.0);
    assert_eq!(
        screen.all_by_role(Role::Button, "Toggle Sidebar").len(),
        2,
        "the trigger and the rail"
    );
    let trigger = screen.get_by_test_id("trigger");
    trigger.click();
    assert!(!open.get_untracked());
    assert_eq!(width_of(&screen, "sidebar"), 48.0);
    // The page took the freed room.
    let page = screen.get_by_text("Page content").rect().expect("laid out");
    assert!(page.x < 100.0, "{page:?}");
    trigger.click();
    assert!(open.get_untracked());
    assert_eq!(width_of(&screen, "sidebar"), 256.0);

    // Ctrl+B toggles while the focus is inside the provider.
    trigger.focus();
    let ctrl = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    trigger.press_key_with(Key::Char('b'), ctrl);
    assert!(!open.get_untracked());
    assert_eq!(width_of(&screen, "sidebar"), 48.0);
    trigger.press_key_with(Key::Char('b'), ctrl);
    assert!(open.get_untracked());
    // Plain B does nothing.
    trigger.press_key(Key::Char('b'));
    assert!(open.get_untracked());

    // The rail toggles too; it is not a tab stop.
    let rail = screen
        .all_by_role(Role::Button, "Toggle Sidebar")
        .into_iter()
        .find(|e| e.rect().is_some_and(|r| r.width == 16.0))
        .expect("the rail is 16 px wide");
    rail.click();
    assert!(!open.get_untracked());
    rail.click();
    assert!(open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}
