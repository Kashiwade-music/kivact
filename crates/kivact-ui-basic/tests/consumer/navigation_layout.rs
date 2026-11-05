//! Role: Layout and navigation tests: resizable, carousel, tabs, scroller and sidebar.

use super::navigation::{light, options, page};
use crate::support::{assert_clean, distance, pixel};
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::ui::ButtonVariant;
use kivact::view::Orientation;
use kivact_ui_basic as basic;

fn near(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() <= tolerance
}

/// Drag `element` by `dx` along x from its own center (points are local to the element).
fn drag_by(element: &kivact::test::Element<'_>, dx: f64) {
    let rect = element.rect().expect("laid out");
    let from = kivact::geometry::Point::new(rect.width / 2.0, rect.height / 2.0);
    element.drag(from, kivact::geometry::Point::new(from.x + dx, from.y));
}

fn mount_resizable(
    orientation: Orientation,
    start: Vec<f64>,
    grip: bool,
) -> (Screen, Signal<Vec<f64>>) {
    kivact::test::render_with_handle(light(), move || {
        let sizes = signal(start);
        (
            sizes,
            page(
                kivact::ui::split(
                    sizes,
                    vec![
                        text("One").test_id("one").into_view(),
                        text("Two").test_id("two").into_view(),
                        text("Three").test_id("three").into_view(),
                    ],
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

    let expanded = sizes.get_untracked();
    first.press_key(Key::Enter);
    let collapsed = sizes.get_untracked();
    assert!(
        near(collapsed[0], 0.1, 0.002),
        "Enter keeps the leading minimum"
    );
    assert!(near(collapsed[0] + collapsed[1], 0.5, 0.002));
    assert!(near(collapsed[2], expanded[2], 0.002));
    first.press_key(Key::Enter);
    let restored = sizes.get_untracked();
    for (actual, expected) in restored.iter().zip(&expanded) {
        assert!(
            near(*actual, *expected, 0.002),
            "Enter restores the previous sizes"
        );
    }

    // Double click equalizes; a drag moves by the pointer's travel.
    first.double_click();
    let now = sizes.get_untracked();
    assert!(
        near(now[0], now[1], 0.002),
        "double click shares the pair equally"
    );
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

fn mount_carousel(
    looped: bool,
    per_view: f64,
    orientation: Orientation,
) -> (Screen, Signal<usize>) {
    kivact::test::render_with_handle(light(), move || {
        let index = signal(0usize);
        let carousel = kivact::ui::carousel(
            index,
            (0..5)
                .map(|n| text(format!("Slide {n}")).test_id(format!("s{n}")))
                .collect::<Vec<_>>(),
        )
        .orientation(orientation)
        .items_per_view(per_view);
        let carousel = if looped { carousel.looping() } else { carousel };
        (
            index,
            page(
                carousel
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
fn line_tabs_select_move_and_mark_the_selected_tab_with_an_underline() {
    use kivact::interaction::ListItem;
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
        .header(text("Acme Inc"))
        .content({
            basic::sidebar_group("Application")
                .action({
                    kivact::ui::button("Add project")
                        .hide_label(true)
                        .icon(kivact::view::icon("plus"))
                        .variant(ButtonVariant::Ghost)
                })
                .content({
                    basic::sidebar_menu((
                        basic::sidebar_menu_button("Home")
                            .icon(kivact::view::icon("house"))
                            .active(true)
                            .test_id("home"),
                        basic::sidebar_menu_button("Inbox")
                            .icon(kivact::view::icon("inbox"))
                            .badge("12")
                            .test_id("inbox"),
                        basic::sidebar_menu_button("Settings")
                            .icon(kivact::view::icon("settings"))
                            .sub({
                                (
                                    basic::sidebar_menu_sub_button("Profile").active(true),
                                    basic::sidebar_menu_sub_button("Billing"),
                                )
                            })
                            .test_id("settings"),
                    ))
                })
        })
        .footer(basic::sidebar_menu_button("Account").icon(kivact::view::icon("user")))
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

fn width_of(screen: &Screen, id: &str) -> f64 {
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
