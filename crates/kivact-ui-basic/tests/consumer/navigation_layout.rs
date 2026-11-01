//! Role: Layout and navigation tests: resizable, carousel, tabs, scroller and sidebar.

use super::navigation::{
    account_items, bar_menus, edit_items, light, nav_panel, open_button, options, page,
    right_click_area,
};
use super::{assert_clean, distance, pixel, pixel_in};
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen};
use kivact::ui::ButtonVariant;
use kivact::view::{Orientation, for_each};
use kivact_ui_basic as basic;

fn near(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}

/// Drag `element` by `dx` along x from its own center (points are local to the element).
fn drag_by(element: &kivact::test::Element<'_>, dx: f32) {
    let rect = element.rect().expect("laid out");
    let from = kivact::draw::Point::new(rect.width / 2.0, rect.height / 2.0);
    element.drag(from, kivact::draw::Point::new(from.x + dx, from.y));
}

fn mount_resizable(
    orientation: Orientation,
    start: Vec<f32>,
    grip: bool,
) -> (Screen, Signal<Vec<f32>>) {
    kivact::test::render_with_handle(light(), move || {
        let sizes = signal(start);
        (
            sizes,
            page(
                kivact::ui::split(
                    vec![
                        text("One").test_id("one").into_view(),
                        text("Two").test_id("two").into_view(),
                        text("Three").test_id("three").into_view(),
                    ],
                    sizes,
                )
                .orientation(orientation)
                .grip(grip)
                .style(style().width(600).height(300)),
            ),
        )
    })
}

#[test]
fn split_drags_and_keys_move_the_boundary_within_min_sizes() {
    let (screen, sizes) = mount_resizable(Orientation::Horizontal, vec![0.25, 0.25, 0.5], true);
    let handles = screen.all_by_role(Role::Splitter, "Resize");
    assert_eq!(handles.len(), 2);
    assert_eq!(
        handles[0].orientation(),
        kivact::view::Orientation::Vertical
    );
    let width = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").width;
    // Panels share the room by their fractions.
    assert!(near(width("one") / width("three"), 0.5, 0.02));
    let first = screen
        .all_by_role(Role::Splitter, "Resize")
        .into_iter()
        .next()
        .expect("two handles");
    assert_eq!(first.value_range().2.map(|v| v.round()), Some(25.0));
    let grip_rect = first.rect().expect("laid out");
    assert!(
        grip_rect.width >= 4.0,
        "the hit area is wider than the line"
    );

    // Keys: ArrowRight grows the first panel by 1 %, taking from the second.
    first.focus();
    first.press_key(Key::ArrowRight);
    let now = sizes.get_untracked();
    assert!(
        near(now[0], 0.26, 0.002) && near(now[1], 0.24, 0.002),
        "{now:?}"
    );
    assert!(near(now[2], 0.5, 0.002));
    first.press_key(Key::Home);
    let now = sizes.get_untracked();
    assert!(near(now[0], 0.1, 0.002), "min size 10 %: {now:?}");
    assert!(near(now[0] + now[1], 0.5, 0.002), "the pair keeps its sum");
    first.press_key(Key::End);
    let now = sizes.get_untracked();
    assert!(
        near(now[1], 0.1, 0.002),
        "the next panel keeps its minimum: {now:?}"
    );

    // A drag moves by the pointer's travel.
    first.press_key(Key::Enter);
    let now = sizes.get_untracked();
    assert!(near(now[0], now[1], 0.002), "Enter shares the pair equally");
    let total = screen.get_by_test_id("one").rect().expect("laid out").width / now[0];
    drag_by(&first, 60.0);
    let after = sizes.get_untracked();
    let expected = now[0] + 60.0 / total;
    assert!(near(after[0], expected, 0.01), "{after:?} vs {expected}");
    // Far past the end: stops at the next panel's minimum.
    drag_by(&first, 5000.0);
    let after = sizes.get_untracked();
    assert!(near(after[1], 0.1, 0.01), "{after:?}");
    assert_clean(&screen);
}

#[test]
fn split_stacks_panels_with_up_and_down_keys() {
    let (screen, sizes) = mount_resizable(Orientation::Vertical, vec![0.5, 0.25, 0.25], false);
    let handle = screen
        .all_by_role(Role::Splitter, "Resize")
        .into_iter()
        .next()
        .expect("handles");
    assert_eq!(handle.orientation(), kivact::view::Orientation::Horizontal);
    let one = screen.get_by_test_id("one").rect().expect("laid out");
    let two = screen.get_by_test_id("two").rect().expect("laid out");
    assert!(two.y > one.y, "stacked top to bottom");
    handle.focus();
    handle.press_key(Key::ArrowDown);
    assert!(near(sizes.get_untracked()[0], 0.51, 0.002));
    // The arrows of the other axis do nothing.
    handle.press_key(Key::ArrowRight);
    assert!(near(sizes.get_untracked()[0], 0.51, 0.002));
    handle.press_key(Key::ArrowUp);
    assert!(near(sizes.get_untracked()[0], 0.5, 0.002));
    assert_clean(&screen);
}

fn mount_carousel(
    looped: bool,
    per_view: f32,
    orientation: Orientation,
) -> (Screen, Signal<usize>) {
    kivact::test::render_with_handle(light(), move || {
        let index = signal(0usize);
        (
            index,
            page(
                basic::carousel(
                    (0..5)
                        .map(|n| text(format!("Slide {n}")).test_id(format!("s{n}")))
                        .collect::<Vec<_>>(),
                    index,
                )
                .orientation(orientation)
                .items_per_view(per_view)
                .looping(looped)
                .test_id("carousel")
                .style(style().width(240).height(120).margin(48)),
            ),
        )
    })
}

#[test]
fn carousel_steps_with_buttons_and_keys_and_stops_at_the_ends() {
    let (screen, index) = mount_carousel(false, 1.0, Orientation::Horizontal);
    let x = |n: usize| {
        screen
            .get_by_test_id(&format!("s{n}"))
            .rect()
            .expect("laid out")
            .x
    };
    let root = screen.get_by_test_id("carousel").rect().expect("laid out");
    let group = screen.get_by_role(Role::Group, "Carousel");
    assert!(group.exists());
    let previous = screen.get_by_role(Role::Button, "Previous slide");
    let next = screen.get_by_role(Role::Button, "Next slide");
    assert!(previous.is_disabled(), "nothing before the first slide");
    assert!(!next.is_disabled());
    // The buttons sit outside the box, 48 px away.
    assert!(next.rect().expect("laid out").x > root.x + root.width);
    assert!(previous.rect().expect("laid out").x < root.x);
    assert!(near(x(0), root.x, 1.0), "the first slide fills the box");
    let step = x(1) - x(0);
    assert!(near(step, 256.0, 1.0), "one slide and the gap: {step}");
    screen.get_by_role(Role::Group, "Slide 2 / 5");

    next.click();
    assert_eq!(index.get_untracked(), 1);
    assert!(near(x(1), root.x, 1.0), "slide 1 slid into view");
    assert!(!previous.is_disabled());
    previous.click();
    assert_eq!(index.get_untracked(), 0);

    // Arrow keys step while the carousel has the focus.
    group.focus();
    group.press_key(Key::ArrowRight);
    group.press_key(Key::ArrowRight);
    assert_eq!(index.get_untracked(), 2);
    group.press_key(Key::ArrowLeft);
    assert_eq!(index.get_untracked(), 1);
    index.set(4);
    screen.run_until_idle();
    assert!(next.is_disabled(), "nothing after the last slide");
    group.press_key(Key::ArrowRight);
    assert_eq!(index.get_untracked(), 4);
    // An index past the end shows the last position.
    index.set(40);
    screen.run_until_idle();
    assert!(next.is_disabled());
    assert_clean(&screen);
}

#[test]
fn carousel_loops_and_shows_several_slides_per_view() {
    let (screen, index) = mount_carousel(true, 1.0, Orientation::Horizontal);
    let previous = screen.get_by_role(Role::Button, "Previous slide");
    let next = screen.get_by_role(Role::Button, "Next slide");
    assert!(!previous.is_disabled(), "a loop has no first slide");
    previous.click();
    assert_eq!(index.get_untracked(), 4, "wraps to the last slide");
    next.click();
    assert_eq!(index.get_untracked(), 0, "and back to the first");

    let (screen, index) = mount_carousel(false, 2.0, Orientation::Horizontal);
    let root = screen.get_by_test_id("carousel").rect().expect("laid out");
    let s0 = screen.get_by_test_id("s0").rect().expect("laid out");
    let s1 = screen.get_by_test_id("s1").rect().expect("laid out");
    assert!(
        s1.x + s1.width <= root.x + root.width + 1.0,
        "two slides fit the box"
    );
    assert!(s1.x > s0.x);
    let next = screen.get_by_role(Role::Button, "Next slide");
    index.set(3);
    screen.run_until_idle();
    assert!(
        next.is_disabled(),
        "the last position still shows a full set"
    );
    assert_clean(&screen);
}

#[test]
fn vertical_carousel_stacks_slides_and_answers_up_and_down() {
    let (screen, index) = mount_carousel(false, 1.0, Orientation::Vertical);
    let y = |n: usize| {
        screen
            .get_by_test_id(&format!("s{n}"))
            .rect()
            .expect("laid out")
            .y
    };
    assert!(y(1) > y(0), "stacked");
    let group = screen.get_by_role(Role::Group, "Carousel");
    group.focus();
    group.press_key(Key::ArrowDown);
    assert_eq!(index.get_untracked(), 1);
    group.press_key(Key::ArrowRight);
    assert_eq!(index.get_untracked(), 1, "the other axis does nothing");
    group.press_key(Key::ArrowUp);
    assert_eq!(index.get_untracked(), 0);
    assert_clean(&screen);
}

#[test]
fn line_tabs_select_move_and_mark_the_selected_tab_with_an_underline() {
    use kivact::ui::ListItem;
    let (screen, current) = kivact::test::render_with_handle(light(), || {
        let current = signal("overview");
        (
            current,
            page(
                tabs(
                    "Tabs",
                    vec![
                        ListItem::new("overview", "Overview"),
                        ListItem::new("analytics", "Analytics"),
                        ListItem::new("reports", "Reports").disabled(true),
                        ListItem::new("settings", "Settings"),
                    ],
                    current,
                )
                .variant(basic::TabsVariant::Line)
                .content(|key| text(format!("Panel {key}"))),
            ),
        )
    });
    screen.get_by_role(Role::TabList, "Tabs");
    let overview = screen.get_by_role(Role::Tab, "Overview");
    let analytics = screen.get_by_role(Role::Tab, "Analytics");
    assert!(overview.is_selected());
    assert!(!analytics.is_selected());
    assert!(screen.get_by_role(Role::Tab, "Reports").is_disabled());
    screen.get_by_text("Panel overview");
    // The underline sits under the selected tab only.
    let under = |tab: &kivact::test::Element<'_>| {
        let rect = tab.rect().expect("laid out");
        pixel(
            &screen,
            rect.x + rect.width / 2.0,
            rect.y + rect.height + 3.5,
        )
    };
    assert!(distance(under(&overview), 0x0a0a0a) <= 4, "selected");
    assert!(distance(under(&analytics), 0xffffff) <= 4, "not selected");
    // A line tab has no filled background, selected or not.
    let rect = overview.rect().expect("laid out");
    assert!(
        distance(
            pixel(&screen, rect.x + 3.0, rect.y + rect.height / 2.0),
            0xffffff
        ) <= 2
    );

    analytics.click();
    assert_eq!(current.get_untracked(), "analytics");
    screen.get_by_text("Panel analytics");
    assert!(distance(under(&analytics), 0x0a0a0a) <= 4);
    assert!(distance(under(&overview), 0xffffff) <= 4);
    // Arrows skip the disabled tab and the selection follows the focus.
    analytics.focus();
    analytics.press_key(Key::ArrowRight);
    assert_eq!(current.get_untracked(), "settings");
    screen.press_key(Key::ArrowLeft);
    assert_eq!(current.get_untracked(), "analytics");
    assert_clean(&screen);
}

fn messages(count: usize) -> Vec<String> {
    (0..count).map(|n| format!("Message number {n}")).collect()
}

fn mount_scroller(count: usize) -> (Screen, Signal<Vec<String>>) {
    kivact::test::render_with_handle(light(), move || {
        let log = signal(messages(count));
        (
            log,
            page(
                basic::message_scroller(column(for_each(
                    log,
                    |line| line.clone(),
                    |_, line| text(line.get()).style(style().height(40)),
                )))
                .test_id("scroller")
                .style(style().width(320).height(200)),
            ),
        )
    })
}

/// Whether `message` is inside the scroller's box.
fn in_view(screen: &Screen, message: &str) -> bool {
    let area = screen.get_by_test_id("scroller").rect().expect("laid out");
    let rect = screen.get_by_text(message).rect().expect("laid out");
    rect.y >= area.y - 1.0 && rect.y + rect.height <= area.y + area.height + 1.0
}

#[test]
fn message_scroller_follows_new_messages_until_the_user_scrolls_away() {
    let (screen, log) = mount_scroller(30);
    let scroller = screen.get_by_test_id("scroller");
    let button = || screen.query_by_role(Role::Button, "Scroll to end");
    // Starts at the end, following, without the button.
    screen.get_by_text("Message number 29");
    assert!(button().is_none(), "the end is in view");
    // A new message keeps the end in view.
    log.update(|log| log.push("Message number 30".to_string()));
    screen.run_until_idle();
    assert!(in_view(&screen, "Message number 30"));
    assert!(!in_view(&screen, "Message number 0"));

    // Scrolling up leaves the end: following stops and the button appears.
    scroller.scroll(0.0, -400.0);
    screen.run_until_idle();
    let jump = button().expect("the end is out of view");
    log.update(|log| log.push("Message number 31".to_string()));
    screen.run_until_idle();
    assert!(
        !in_view(&screen, "Message number 31"),
        "the view stays where the user put it"
    );
    assert!(button().is_some(), "still not at the end");

    // The button goes back to the end and follows again.
    jump.click();
    screen.run_until_idle();
    assert!(button().is_none());
    assert!(in_view(&screen, "Message number 31"));
    log.update(|log| log.push("Message number 32".to_string()));
    screen.run_until_idle();
    assert!(in_view(&screen, "Message number 32"));
    assert_clean(&screen);
}

fn app_sidebar(
    collapsible: basic::SidebarCollapsible,
    variant: basic::SidebarVariant,
    side: basic::SidebarSide,
) -> basic::Sidebar {
    basic::sidebar()
        .collapsible(collapsible)
        .variant(variant)
        .side(side)
        .rail(true)
        .test_id("sidebar")
        .header(move || text("Acme Inc"))
        .content(move || {
            basic::sidebar_group("Application")
                .action(move || {
                    kivact::ui::button("Add project")
                        .hide_label(true)
                        .icon(kivact::control::icon("plus"))
                        .variant(ButtonVariant::Ghost)
                })
                .content(move || {
                    basic::sidebar_menu((
                        basic::sidebar_menu_button("Home")
                            .icon(kivact::control::icon("house"))
                            .active(true)
                            .test_id("home"),
                        basic::sidebar_menu_button("Inbox")
                            .icon(kivact::control::icon("inbox"))
                            .badge("12")
                            .test_id("inbox"),
                        basic::sidebar_menu_button("Settings")
                            .icon(kivact::control::icon("settings"))
                            .sub(move || {
                                (
                                    basic::sidebar_menu_sub_button("Profile").active(true),
                                    basic::sidebar_menu_sub_button("Billing"),
                                )
                            })
                            .test_id("settings"),
                    ))
                })
        })
        .footer(move || basic::sidebar_menu_button("Account").icon(kivact::control::icon("user")))
}

fn mount_sidebar(
    collapsible: basic::SidebarCollapsible,
    variant: basic::SidebarVariant,
    side: basic::SidebarSide,
    mode: kivact::style::ThemeMode,
) -> (Screen, Signal<bool>) {
    kivact::test::render_with_handle(options(mode), move || {
        let open = signal(true);
        (
            open,
            column(
                basic::sidebar_provider(
                    open,
                    app_sidebar(collapsible, variant, side),
                    column((
                        basic::sidebar_trigger().test_id("trigger"),
                        text("Page content"),
                    ))
                    .style(style().padding(8).gap(8)),
                )
                .test_id("provider"),
            )
            .style(style().width(800).height(500)),
        )
    })
}

fn width_of(screen: &Screen, id: &str) -> f32 {
    screen.get_by_test_id(id).rect().expect("laid out").width
}

#[test]
fn sidebar_toggles_by_trigger_shortcut_and_rail_between_256_and_48() {
    let (screen, open) = mount_sidebar(
        basic::SidebarCollapsible::Icon,
        basic::SidebarVariant::Sidebar,
        basic::SidebarSide::Left,
        kivact::style::ThemeMode::Light,
    );
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
    let ctrl = kivact::view::Modifiers {
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
    assert_clean(&screen);
}

#[test]
fn icon_sidebar_keeps_icons_shows_tooltips_and_hides_labels() {
    let (screen, open) = mount_sidebar(
        basic::SidebarCollapsible::Icon,
        basic::SidebarVariant::Sidebar,
        basic::SidebarSide::Left,
        kivact::style::ThemeMode::Light,
    );
    // Expanded: labels, badge and the sub-list show; no tooltip appears.
    screen.get_by_text("Home");
    screen.get_by_text("12");
    screen.get_by_role(Role::Link, "Profile");
    screen.get_by_text("Application");
    let home = screen.get_by_test_id("home");
    assert!(home.is_selected(), "the active item is selected");
    home.hover();
    screen.advance(std::time::Duration::from_millis(700));
    assert!(screen.query_by_role(Role::Tooltip, "Home").is_none());
    let home_rect = home.rect().expect("laid out");
    assert_eq!(home_rect.height, 32.0);
    assert!(home_rect.width > 200.0);

    open.set(false);
    screen.run_until_idle();
    let home = screen.get_by_test_id("home");
    let rect = home.rect().expect("laid out");
    assert_eq!((rect.width, rect.height), (32.0, 32.0), "size-8");
    let hidden = |text: &str| !screen.query_by_text(text).is_some_and(|e| e.is_visible());
    assert!(hidden("Home"), "label hidden");
    assert!(hidden("12"), "badge hidden");
    assert!(hidden("Application"));
    assert!(
        screen.query_by_role(Role::Link, "Profile").is_none(),
        "sub-list hidden"
    );
    // The button keeps its name and its press.
    screen.get_by_role(Role::Button, "Inbox");
    // Hovering a collapsed button shows its label in a tooltip on its right.
    screen.mouse_move(900.0, 600.0);
    screen.advance(std::time::Duration::from_millis(700));
    assert!(screen.query_by_role(Role::Tooltip, "Home").is_none());
    home.hover();
    screen.advance(std::time::Duration::from_millis(700));
    let tip = screen.get_by_role(Role::Tooltip, "Home");
    assert!(
        tip.rect().expect("laid out").x >= rect.x + rect.width,
        "on the right"
    );
    assert_clean(&screen);
}

#[test]
fn offcanvas_sidebar_slides_out_and_a_fixed_sidebar_ignores_the_toggle() {
    let (screen, open) = mount_sidebar(
        basic::SidebarCollapsible::Offcanvas,
        basic::SidebarVariant::Sidebar,
        basic::SidebarSide::Left,
        kivact::style::ThemeMode::Light,
    );
    assert_eq!(width_of(&screen, "sidebar"), 256.0);
    screen.get_by_test_id("trigger").click();
    assert!(!open.get_untracked());
    assert_eq!(width_of(&screen, "sidebar"), 0.0);
    assert!(
        screen
            .query_by_text("Acme Inc")
            .is_none_or(|e| !e.is_visible()),
        "nothing is left to tab to"
    );
    // The rail stays, to bring the sidebar back.
    let rail = screen
        .all_by_role(Role::Button, "Toggle Sidebar")
        .into_iter()
        .find(|e| e.rect().is_some_and(|r| r.width == 16.0))
        .expect("the rail stays");
    rail.click();
    assert!(open.get_untracked());
    assert_eq!(width_of(&screen, "sidebar"), 256.0);
    assert_clean(&screen);

    let (fixed, open) = mount_sidebar(
        basic::SidebarCollapsible::None,
        basic::SidebarVariant::Sidebar,
        basic::SidebarSide::Left,
        kivact::style::ThemeMode::Light,
    );
    fixed.get_by_test_id("trigger").click();
    assert!(open.get_untracked());
    assert_eq!(width_of(&fixed, "sidebar"), 256.0);
    assert_clean(&fixed);
}

#[test]
fn sidebar_menu_buttons_mark_the_active_item_and_press() {
    let (screen, hits) = kivact::test::render_with_handle(light(), || {
        let open = signal(true);
        let hits = signal(0usize);
        (
            hits,
            column(basic::sidebar_provider(
                open,
                basic::sidebar().content(move || {
                    basic::sidebar_menu((
                        basic::sidebar_menu_button("Home")
                            .active(true)
                            .test_id("home")
                            .on_press(move || hits.update(|n| *n += 1)),
                        basic::sidebar_menu_button("Inbox").test_id("inbox"),
                        basic::sidebar_menu_button("Large")
                            .size(basic::SidebarMenuButtonSize::Lg)
                            .test_id("large"),
                        basic::sidebar_menu_button("Small")
                            .size(basic::SidebarMenuButtonSize::Sm)
                            .test_id("small"),
                        basic::sidebar_menu_skeleton(),
                    ))
                }),
                text("Page"),
            ))
            .style(style().width(800).height(400)),
        )
    });
    let height = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").height;
    assert_eq!(
        (height("home"), height("large"), height("small")),
        (32.0, 48.0, 28.0)
    );
    // The active item is filled with sidebar-accent, the others are plain.
    assert!(distance(pixel_in(&screen, "home", 4.0), 0xf5f5f5) <= 2);
    assert!(distance(pixel_in(&screen, "inbox", 4.0), 0xfafafa) <= 2);
    screen.get_by_test_id("home").click();
    assert_eq!(hits.get_untracked(), 1);
    // A hovered, inactive item takes the accent too.
    screen.get_by_test_id("inbox").hover();
    assert!(distance(pixel_in(&screen, "inbox", 4.0), 0xf5f5f5) <= 2);
    assert_clean(&screen);
}

#[test]
fn sidebar_variants_and_sides_lay_out_cleanly_in_light_and_dark() {
    for mode in [
        kivact::style::ThemeMode::Light,
        kivact::style::ThemeMode::Dark,
    ] {
        for (collapsible, variant, side) in [
            (
                basic::SidebarCollapsible::Icon,
                basic::SidebarVariant::Sidebar,
                basic::SidebarSide::Left,
            ),
            (
                basic::SidebarCollapsible::Offcanvas,
                basic::SidebarVariant::Floating,
                basic::SidebarSide::Right,
            ),
            (
                basic::SidebarCollapsible::None,
                basic::SidebarVariant::Inset,
                basic::SidebarSide::Left,
            ),
        ] {
            let (screen, open) = mount_sidebar(collapsible, variant, side, mode);
            screen.assert_layout();
            assert_clean(&screen);
            if collapsible != basic::SidebarCollapsible::None {
                open.set(false);
                screen.run_until_idle();
                screen.assert_layout();
                assert_clean(&screen);
            }
        }
    }
}

#[test]
fn menubar_carousel_and_sidebar_follow_a_right_to_left_direction() {
    let mut seen = Vec::new();
    for dir in [
        kivact::view::ReadingDirection::Ltr,
        kivact::view::ReadingDirection::Rtl,
    ] {
        let (screen, index) =
            kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), move || {
                let index = signal(0usize);
                let open = signal(true);
                (
                    index,
                    kivact::view::provide_direction(dir, move || {
                        page((
                            basic::menubar(bar_menus(), |_key: &'static str| {})
                                .style(style().width(300)),
                            basic::carousel(
                                vec![
                                    text("Alpha").test_id("alpha").into_view(),
                                    text("Beta").test_id("beta").into_view(),
                                ],
                                index,
                            )
                            .test_id("carousel")
                            .style(style().width(200).margin_x(48)),
                            column(basic::sidebar_provider(
                                open,
                                basic::sidebar().content(move || {
                                    basic::sidebar_menu(
                                        basic::sidebar_menu_button("Inbox")
                                            .icon(
                                                kivact::control::icon("inbox")
                                                    .test_id("inbox-icon"),
                                            )
                                            .badge("12")
                                            .test_id("inbox"),
                                    )
                                }),
                                text("Page"),
                            ))
                            .style(style().width(500).height(120)),
                        ))
                    }),
                )
            });
        let x = |text: &str| screen.get_by_text(text).rect().expect("laid out").x;
        let bar = (x("File") < x("Edit")) as u8;
        let slides = (x("Alpha") < x("Beta")) as u8;
        let prev = screen.get_by_role(Role::Button, "Previous slide");
        let next = screen.get_by_role(Role::Button, "Next slide");
        let buttons = (prev.rect().expect("laid out").x < next.rect().expect("laid out").x) as u8;
        let icon = screen
            .get_by_test_id("inbox-icon")
            .rect()
            .expect("laid out")
            .x;
        let icon_first = (icon < x("Inbox")) as u8;
        let badge_first = (x("12") < x("Inbox")) as u8;
        // Arrow keys mirror: with the carousel focused, the key toward the
        // start of the line steps back.
        let group = screen.get_by_role(Role::Group, "Carousel");
        group.focus();
        group.press_key(Key::ArrowRight);
        seen.push((
            bar,
            slides,
            buttons,
            icon_first,
            badge_first,
            index.get_untracked(),
        ));
        assert_clean(&screen);
    }
    assert_eq!(seen[0], (1, 1, 1, 1, 0, 1), "left to right");
    assert_eq!(
        seen[1],
        (0, 0, 0, 0, 1, 0),
        "right to left reverses the rows, and ArrowRight steps back"
    );
}

/// Every navigation component once.
fn gallery() -> impl IntoView {
    let index = signal(0usize);
    let sizes = signal(vec![0.3f32, 0.7]);
    let tab = signal("overview");
    let open = signal(true);
    let links = signal(0usize);
    page((
        basic::menubar(bar_menus(), |_key: &'static str| {}).style(style().width(320)),
        row((
            menu(
                "Profile menu",
                account_items("top", true),
                |_key: &'static str| {},
            )
            .at_least_trigger_width(true)
            .attach(open_button("Profile menu")),
            menu("Edit", edit_items(), |_key: &'static str| {}).attach_context(right_click_area()),
        ))
        .style(style().gap(16)),
        basic::navigation_menu(vec![
            basic::NavItem::menu(
                "Getting started",
                nav_panel(&["Introduction", "Installation"], links),
            ),
            basic::NavItem::link("Docs", || {}),
        ]),
        tabs(
            "Tabs",
            vec![
                ListItem::new("overview", "Overview"),
                ListItem::new("reports", "Reports"),
            ],
            tab,
        )
        .variant(basic::TabsVariant::Line)
        .content(|key| text(format!("Panel {key}"))),
        kivact::ui::split(
            vec![text("One").into_view(), text("Two").into_view()],
            sizes,
        )
        .grip(true)
        .style(
            style()
                .width(400)
                .height(80)
                .border(1, basic::Color::Border),
        ),
        basic::carousel(
            (1..=3)
                .map(|n| {
                    // Slides outside the box are cut by design.
                    text(format!("Slide {n}")).test_id("layout-check-ignore")
                })
                .collect::<Vec<_>>(),
            index,
        )
        .style(style().width(240).margin_x(48)),
        basic::message_scroller(column(
            messages(12)
                .into_iter()
                .map(|line| text(line).style(style().height(24)))
                .collect::<Vec<_>>(),
        ))
        .style(style().width(320).height(100)),
        column(basic::sidebar_provider(
            open,
            app_sidebar(
                basic::SidebarCollapsible::Icon,
                basic::SidebarVariant::Sidebar,
                basic::SidebarSide::Left,
            ),
            text("Page"),
        ))
        .style(style().width(500).height(260)),
    ))
}

#[test]
fn navigation_gallery_lays_out_cleanly_in_light_and_dark() {
    for mode in [
        kivact::style::ThemeMode::Light,
        kivact::style::ThemeMode::Dark,
    ] {
        let (screen, ()) = kivact::test::render_with_handle(
            RenderOptions {
                height: 1500.0,
                ..options(mode)
            },
            || ((), gallery()),
        );
        let layout = |screen: &Screen| {
            let check = kivact::test::LayoutCheck::default().ignore_test_id("layout-check-ignore");
            let found = screen.check_layout_with(&check);
            assert!(
                found.is_empty(),
                "{} layout violation(s):
  {}",
                found.len(),
                found
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(
                        "
  "
                    )
            );
        };
        layout(&screen);
        assert_clean(&screen);
        // With a dropdown and a hover panel open the overlays are laid out too.
        screen
            .get_by_role(Role::Button, "Profile menu")
            .press_key(Key::ArrowDown);
        layout(&screen);
        screen.press_key(Key::Escape);
        screen.get_by_role(Role::Button, "Getting started").hover();
        screen.get_by_role(Role::Link, "Introduction");
        layout(&screen);
        assert_clean(&screen);
    }
}
