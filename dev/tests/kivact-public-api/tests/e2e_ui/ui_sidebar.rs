//! Role: Verify the sidebar's widths, its trigger, its shortcut, its rail, its resize handle and its sub lists.

use kivact::prelude::*;
use kivact::test::Screen;
use kivact::ui::{
    SidebarCollapsible, SidebarFrame, SidebarResize, SidebarSide, sidebar, sidebar_group,
    sidebar_menu, sidebar_menu_button, sidebar_menu_sub_button, sidebar_provider, sidebar_trigger,
    use_sidebar,
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

fn mount_resizable(width: f64) -> (Screen, Signal<f64>) {
    kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(Theme::light())
            .window_size(900.0, 700.0)
            .reduced_motion(),
        move || {
            let width = signal(width);
            (
                width,
                column(
                    sidebar_provider(
                        signal(true),
                        sidebar()
                            .header(text("Acme Inc"))
                            .test_id("sidebar")
                            .into_view(),
                        column(text("Page content")).into_view(),
                    )
                    .resize(SidebarResize {
                        width,
                        min: 200.0,
                        max: 400.0,
                    })
                    .test_id("provider"),
                )
                .style(style().width(800).height(500)),
            )
        },
    )
}

#[test]
fn a_resizable_sidebar_follows_its_width_between_min_and_max() {
    let (screen, width) = mount_resizable(300.0);
    assert_eq!(width_of(&screen, "sidebar"), 300.0);
    // The handle is a splitter; the arrow keys move it 8 px.
    let split = screen.get_by_role(Role::Splitter, "Resize sidebar");
    split.focus();
    split.press_key(Key::ArrowRight);
    screen.run_until_idle();
    assert_eq!(width.get_untracked(), 308.0);
    assert_eq!(width_of(&screen, "sidebar"), 308.0);
    // A width outside the bounds is kept between them.
    width.set(1000.0);
    screen.run_until_idle();
    assert_eq!(width_of(&screen, "sidebar"), 400.0);
    width.set(10.0);
    screen.run_until_idle();
    assert_eq!(width_of(&screen, "sidebar"), 200.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}

/// A page part that shows the sidebar state it reads with `use_sidebar`.
#[component]
fn sidebar_probe() -> impl Element {
    let handle = use_sidebar();
    text(move || match handle {
        Some(handle) if handle.icon_only() => "probe icon".to_string(),
        Some(_) => "probe full".to_string(),
        None => "probe none".to_string(),
    })
}

fn mount_probe() -> (Screen, Signal<bool>) {
    kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(Theme::light())
            .window_size(900.0, 700.0)
            .reduced_motion(),
        move || {
            let open = signal(true);
            (
                open,
                column(
                    sidebar_provider(open, app_sidebar().into_view(), sidebar_probe().into_view())
                        .collapsible(SidebarCollapsible::Icon)
                        .test_id("provider"),
                )
                .style(style().width(800).height(500)),
            )
        },
    )
}

#[test]
fn use_sidebar_reads_the_providers_icon_only_state() {
    let (screen, _) = mount_probe();
    screen.get_by_text("probe full");
    let rail = screen
        .all_by_role(Role::Button, "Toggle Sidebar")
        .into_iter()
        .find(|e| e.rect().is_some_and(|r| r.width == 16.0))
        .expect("the rail is 16 px wide");
    rail.click();
    screen.run_until_idle();
    screen.get_by_text("probe icon");
    rail.click();
    screen.run_until_idle();
    screen.get_by_text("probe full");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn mount_sub_menu() -> (Screen, Signal<bool>) {
    kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(Theme::light())
            .window_size(900.0, 700.0)
            .reduced_motion(),
        move || {
            let sub_open = signal(false);
            let menu = sidebar_menu(vec![
                sidebar_menu_button("Settings")
                    .icon(kivact::view::icon("calendar"))
                    .sub_open(sub_open)
                    .sub(column((
                        sidebar_menu_sub_button("Profile"),
                        sidebar_menu_sub_button("Billing"),
                    )))
                    .into_view(),
            ]);
            (
                sub_open,
                column(
                    sidebar_provider(
                        signal(true),
                        sidebar()
                            .content(sidebar_group("Application").content(menu))
                            .into_view(),
                        column(text("Page content")).into_view(),
                    )
                    .test_id("provider"),
                )
                .style(style().width(800).height(500)),
            )
        },
    )
}

#[test]
fn a_menu_button_with_sub_open_toggles_its_sub_items() {
    let (screen, sub_open) = mount_sub_menu();
    assert_eq!(
        screen.get_by_role(Role::Button, "Settings").is_expanded(),
        Some(false)
    );
    assert!(screen.query_by_role(Role::Link, "Profile").is_none());
    screen.get_by_role(Role::Button, "Settings").click();
    screen.run_until_idle();
    assert!(sub_open.get_untracked());
    assert_eq!(
        screen.get_by_role(Role::Button, "Settings").is_expanded(),
        Some(true)
    );
    assert!(screen.query_by_role(Role::Link, "Profile").is_some());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}

#[test]
fn a_disabled_menu_button_takes_no_press() {
    let (screen, hits) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(Theme::light())
            .window_size(900.0, 700.0)
            .reduced_motion(),
        move || {
            let hits = signal(0_u32);
            let menu = sidebar_menu(vec![
                sidebar_menu_button("Locked")
                    .icon(kivact::view::icon("calendar"))
                    .disabled(true)
                    .on_press(move || hits.update(|hits| *hits += 1))
                    .into_view(),
                sidebar_menu_button("Open")
                    .icon(kivact::view::icon("copy"))
                    .on_press(move || hits.update(|hits| *hits += 1))
                    .into_view(),
            ]);
            (
                hits,
                column(
                    sidebar_provider(
                        signal(true),
                        sidebar()
                            .content(sidebar_group("Application").content(menu))
                            .into_view(),
                        column(text("Page content")).into_view(),
                    )
                    .test_id("provider"),
                )
                .style(style().width(800).height(500)),
            )
        },
    );
    let locked = screen.get_by_role(Role::Button, "Locked");
    assert!(locked.is_disabled());
    // A disabled row takes no press even where the pointer lands on it.
    locked.force_click();
    assert_eq!(hits.get_untracked(), 0);
    // The enabled row next to it does.
    screen.get_by_role(Role::Button, "Open").click();
    assert_eq!(hits.get_untracked(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}
