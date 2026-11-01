//! Role: Caller overrides, sidebar parts and the right-to-left navigation menu tests.

use super::navigation::{light, nav_panel, page};
use super::{CALLER_FILL, assert_caller_won, assert_clean, caller_style};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::ui::ListItem;
use kivact::view::View;
use kivact_ui_basic as basic;

fn width_of(screen: &kivact::test::Screen, id: &str) -> f32 {
    screen.get_by_test_id(id).rect().expect("laid out").width
}

/// The caller's width, padding and fill (and name) on the navigation and layout
/// builders: they come after the constructor's own style, so they must win.
#[test]
fn caller_styles_and_names_win_on_navigation_components() {
    let (screen, ()) = kivact::test::render_with_handle(light(), || {
        let index = signal(0usize);
        let sizes = signal(vec![0.5f32, 0.5]);
        let tab = signal("a");
        let open = signal(true);
        let loose: Vec<View> = vec![
            basic::carousel(
                vec![text("One").into_view(), text("Two").into_view()],
                index,
            )
            .name("Custom carousel")
            .style(caller_style().margin_x(60))
            .test_id("carousel")
            .into_view(),
            kivact::ui::split(vec![text("A").into_view(), text("B").into_view()], sizes)
                .style(caller_style().height(60))
                .test_id("resizable")
                .into_view(),
            tabs(
                "Tabs",
                vec![ListItem::new("a", "A"), ListItem::new("b", "B")],
                tab,
            )
            .variant(basic::TabsVariant::Line)
            .style(caller_style())
            .test_id("line-tabs")
            .into_view(),
            basic::navigation_menu(vec![basic::NavItem::link("Home", || {})])
                .name("Custom nav")
                .style(caller_style())
                .test_id("nav-menu")
                .into_view(),
            basic::navigation_menu_link("Real nav link")
                .on_press(|| {})
                .name("Custom nav link")
                .style(caller_style())
                .test_id("nav-link")
                .into_view(),
        ];
        (
            (),
            page((
                column(loose).style(style().gap(16).align_items(Align::Start)),
                column(basic::sidebar_provider(
                    open,
                    basic::sidebar()
                        .collapsible(basic::SidebarCollapsible::None)
                        .style(caller_style())
                        .test_id("sidebar")
                        .content(move || {
                            basic::sidebar_group("Group")
                                .style(caller_style())
                                .test_id("group")
                                .content(move || {
                                    basic::sidebar_menu(
                                        basic::sidebar_menu_button("Real button")
                                            .name("Custom button")
                                            .style(caller_style())
                                            .test_id("menu-button")
                                            .sub(move || {
                                                basic::sidebar_menu_sub_button("Real sub")
                                                    .name("Custom sub")
                                                    .style(caller_style())
                                                    .test_id("sub-button")
                                            }),
                                    )
                                })
                        }),
                    column((
                        basic::sidebar_trigger()
                            .name("Custom trigger")
                            .style(style().padding_x(24).background(rgb(CALLER_FILL)))
                            .test_id("trigger"),
                        text("Page"),
                    )),
                ))
                .style(style().width(800).height(300)),
            )),
        )
    });
    screen.get_by_role(Role::Group, "Custom carousel");
    screen.get_by_role(Role::Group, "Custom nav");
    screen.get_by_role(Role::Link, "Custom nav link");
    screen.get_by_role(Role::Button, "Custom button");
    screen.get_by_role(Role::Link, "Custom sub");
    screen.get_by_role(Role::Button, "Custom trigger");
    for id in [
        "carousel",
        "resizable",
        "line-tabs",
        "nav-menu",
        "nav-link",
        "sidebar",
        "group",
        "menu-button",
        "sub-button",
    ] {
        assert_caller_won(&screen, id, true);
    }
    assert_caller_won(&screen, "trigger", false);
    assert_clean(&screen);
}

#[test]
fn sidebar_sub_buttons_press_and_the_trigger_keeps_the_callers_callback() {
    let (screen, (open, profile, billing, toggled)) =
        kivact::test::render_with_handle(light(), || {
            let open = signal(true);
            let profile = signal(0usize);
            let billing = signal(0usize);
            let toggled = signal(0usize);
            (
                (open, profile, billing, toggled),
                column(basic::sidebar_provider(
                    open,
                    basic::sidebar().content(move || {
                        basic::sidebar_menu(
                            basic::sidebar_menu_button("Settings")
                                .icon(kivact::control::icon("settings"))
                                .sub(move || {
                                    (
                                        basic::sidebar_menu_sub_button("Profile")
                                            .on_press(move || profile.update(|n| *n += 1)),
                                        basic::sidebar_menu_sub_button("Billing")
                                            .on_press(move || billing.update(|n| *n += 1)),
                                    )
                                }),
                        )
                    }),
                    column((
                        basic::sidebar_trigger()
                            .on_press(move || toggled.update(|n| *n += 1))
                            .test_id("trigger"),
                        text("Page"),
                    )),
                ))
                .style(style().width(800).height(400)),
            )
        });
    let profile_link = screen.get_by_role(Role::Link, "Profile");
    profile_link.click();
    assert_eq!(profile.get_untracked(), 1);
    profile_link.focus();
    profile_link.press_key(Key::Enter);
    profile_link.press_key(Key::Space);
    assert_eq!(profile.get_untracked(), 3);
    screen.get_by_role(Role::Link, "Billing").click();
    assert_eq!((profile.get_untracked(), billing.get_untracked()), (3, 1));
    // The trigger toggles the sidebar and then also tells the caller.
    screen.get_by_test_id("trigger").click();
    assert!(!open.get_untracked());
    assert_eq!(toggled.get_untracked(), 1);
    screen.get_by_test_id("trigger").click();
    assert!(open.get_untracked());
    assert_eq!(toggled.get_untracked(), 2);
    assert_clean(&screen);
}

#[test]
fn sidebar_shortcut_works_with_nothing_focused_when_the_provider_is_the_root() {
    let (screen, open) = kivact::test::render_with_handle(light(), || {
        let open = signal(true);
        (
            open,
            basic::sidebar_provider(
                open,
                basic::sidebar()
                    .test_id("sidebar")
                    .content(move || text("Menu"))
                    .collapsible(basic::SidebarCollapsible::Icon),
                text("Page"),
            ),
        )
    });
    assert!(screen.focused().is_none(), "nothing has the focus");
    let ctrl = kivact::view::Modifiers {
        ctrl: true,
        ..Default::default()
    };
    screen.press_key_with(Key::Char('b'), ctrl);
    assert!(!open.get_untracked());
    assert_eq!(width_of(&screen, "sidebar"), 48.0);
    screen.press_key_with(Key::Char('b'), ctrl);
    assert!(open.get_untracked());
    assert_eq!(width_of(&screen, "sidebar"), 256.0);
    assert_clean(&screen);
}

#[test]
fn navigation_menu_follows_a_right_to_left_direction() {
    let mut seen = Vec::new();
    for dir in [
        kivact::view::ReadingDirection::Ltr,
        kivact::view::ReadingDirection::Rtl,
    ] {
        let (screen, ()) = kivact::test::render_with_handle(light(), move || {
            let links = signal(0usize);
            (
                (),
                page(kivact::view::provide_direction(dir, move || {
                    basic::navigation_menu(vec![
                        basic::NavItem::menu(
                            "Getting started",
                            nav_panel(&["Introduction"], links),
                        ),
                        basic::NavItem::menu("Components", nav_panel(&["Alert Dialog"], links)),
                        basic::NavItem::link("Docs", || {}),
                    ])
                })),
            )
        });
        let started = screen.get_by_role(Role::Button, "Getting started");
        let components = screen.get_by_role(Role::Button, "Components");
        let docs = screen.get_by_role(Role::Link, "Docs");
        let x = |e: &kivact::test::Element<'_>| e.rect().expect("laid out").x;
        let in_order = (x(&started) < x(&components) && x(&components) < x(&docs)) as u8;
        // ArrowRight, then ArrowLeft, from the first entry (Home goes back
        // to it: the entries are one tab stop that the arrows move).
        started.focus();
        screen.press_key(Key::ArrowRight);
        let right = (components.is_focused()) as u8;
        screen.press_key(Key::Home);
        screen.press_key(Key::ArrowLeft);
        let left_wraps = docs.is_focused() as u8;
        // Escape still closes a panel opened with ArrowDown.
        screen.press_key(Key::Home);
        assert!(started.is_focused());
        screen.press_key(Key::ArrowDown);
        screen.get_by_role(Role::Link, "Introduction");
        started.press_key(Key::Escape);
        assert!(screen.query_by_role(Role::Link, "Introduction").is_none());
        seen.push((in_order, right, left_wraps));
        assert_clean(&screen);
    }
    assert_eq!(
        seen[0],
        (1, 1, 1),
        "left to right: ArrowRight steps on, ArrowLeft from the first wraps to the last"
    );
    assert_eq!(
        seen[1],
        (0, 0, 0),
        "right to left reverses the row and the arrows: ArrowLeft steps on"
    );
}
