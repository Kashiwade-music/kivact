//! Role: Verify the tabs' overflow on real windows.
//! The scroll that keeps the selected tab in view, and the "More tabs" menu
//! that takes the tabs past the box.

use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen};
use kivact::ui::TabOverflow;
use kivact::view::Orientation;

/// The signals a test drives: the box's size, the selection, the first tab's label and the theme.
#[derive(Clone, Copy)]
struct Handles {
    width: Signal<f64>,
    height: Signal<f64>,
    selected: Signal<usize>,
    first: Signal<String>,
    theme: Signal<Theme>,
}

/// Eight tabs, "Tab 1" to "Tab 8", in a box of `width` by `height`, selecting `initial`.
fn mount(
    overflow: TabOverflow,
    orientation: Orientation,
    width: f64,
    height: f64,
    initial: usize,
) -> (Screen, Handles) {
    let (screen, handles) = kivact::test::render_with_handle(
        RenderOptions::with_theme(Theme::light())
            .window_size(2400.0, 800.0)
            .reduced_motion(),
        move || {
            let handles = Handles {
                width: signal(width),
                height: signal(height),
                selected: signal(initial),
                first: signal("Tab 1".to_string()),
                theme: signal(Theme::light()),
            };
            let view = provide_theme(handles.theme, move || app(handles, overflow, orientation));
            (handles, view)
        },
    );
    screen.run_until_idle();
    (screen, handles)
}

fn app(h: Handles, overflow: TabOverflow, orientation: Orientation) -> View {
    let first = h.first;
    let items = move || {
        (1..=8usize)
            .map(|n| {
                let label = if n == 1 {
                    first.get()
                } else {
                    format!("Tab {n}")
                };
                ListItem::new(n, label)
            })
            .collect::<Vec<_>>()
    };
    let vertical = orientation == Orientation::Vertical;
    let strip = tabs("Tabs", items, h.selected)
        .orientation(orientation)
        .overflow(overflow)
        .test_id("tabs")
        .style(move || {
            if vertical {
                style().height(pct(100))
            } else {
                style()
            }
        });
    column(strip)
        .style(move || style().width(h.width.get()).height(h.height.get()))
        .test_id("box")
        .into_view()
}

/// Whether the tab named `name` lies inside the box, from its left edge to its right one.
fn inside(screen: &Screen, name: &str) -> bool {
    let bounds = screen
        .get_by_test_id("box")
        .rect()
        .expect("the box is laid out");
    let tab = screen
        .get_by_role(Role::Tab, name)
        .rect()
        .expect("the tab is laid out");
    tab.x >= bounds.x - 0.5 && tab.x + tab.width <= bounds.x + bounds.width + 0.5
}

/// Whether a tab named `name` is shown (a tab past the room of the strip is laid out but hidden).
fn tab_shown(screen: &Screen, name: &str) -> bool {
    screen
        .all_by_role(Role::Tab, name)
        .iter()
        .any(|tab| tab.is_visible())
}

/// How many of the eight tabs are shown.
fn visible(screen: &Screen) -> usize {
    (1..=8)
        .filter(|n| tab_shown(screen, &format!("Tab {n}")))
        .count()
}

fn has_more_button(screen: &Screen) -> bool {
    screen
        .all_by_role(Role::Button, "More tabs")
        .iter()
        .any(|button| button.is_visible())
}

#[test]
fn scroll_overflow_shows_an_initially_last_selection() {
    let (screen, _) = mount(TabOverflow::Scroll, Orientation::Horizontal, 200.0, 60.0, 8);
    assert!(inside(&screen, "Tab 8"));
}

#[test]
fn scroll_overflow_follows_an_external_selection() {
    let (screen, h) = mount(TabOverflow::Scroll, Orientation::Horizontal, 200.0, 60.0, 1);
    assert!(inside(&screen, "Tab 1"));
    h.selected.set(8);
    screen.run_until_idle();
    assert!(inside(&screen, "Tab 8"));
    h.selected.set(1);
    screen.run_until_idle();
    assert!(inside(&screen, "Tab 1"));
}

#[test]
fn scroll_overflow_keeps_the_selection_visible_after_a_resize() {
    let (screen, h) = mount(TabOverflow::Scroll, Orientation::Horizontal, 200.0, 60.0, 8);
    h.width.set(120.0);
    screen.run_until_idle();
    assert!(inside(&screen, "Tab 8"));
}

#[test]
fn menu_overflow_is_stable_across_flushes() {
    let (screen, _) = mount(TabOverflow::Menu, Orientation::Horizontal, 200.0, 60.0, 8);
    let count = visible(&screen);
    assert!(count < 8, "the box is too narrow for every tab");
    assert!(has_more_button(&screen));
    for _ in 0..3 {
        screen.run_until_idle();
        assert_eq!(visible(&screen), count);
    }
}

#[test]
fn menu_overflow_follows_a_resize_both_ways() {
    let (screen, h) = mount(TabOverflow::Menu, Orientation::Horizontal, 200.0, 60.0, 8);
    let at_200 = visible(&screen);
    h.width.set(120.0);
    screen.run_until_idle();
    assert!(visible(&screen) < at_200);
    h.width.set(2000.0);
    screen.run_until_idle();
    assert_eq!(visible(&screen), 8);
    assert!(!has_more_button(&screen));
    h.width.set(200.0);
    screen.run_until_idle();
    assert_eq!(visible(&screen), at_200);
}

#[test]
fn menu_overflow_follows_label_and_theme_changes() {
    let (screen, h) = mount(TabOverflow::Menu, Orientation::Horizontal, 200.0, 60.0, 8);
    let at_start = visible(&screen);
    h.first
        .set("A first tab label that is far too long for the box".to_string());
    screen.run_until_idle();
    assert!(visible(&screen) < at_start);
    h.first.set("Tab 1".to_string());
    screen.run_until_idle();
    assert_eq!(visible(&screen), at_start);
    // The tab recipe pads each tab by the medium space; a wider space makes the tabs wider.
    h.theme
        .set(Theme::light().set(&kivact::style::space::MD_VAR, 40.0));
    screen.run_until_idle();
    assert!(visible(&screen) < at_start);
}

#[test]
fn menu_overflow_works_for_vertical_tabs() {
    let (screen, _) = mount(TabOverflow::Menu, Orientation::Vertical, 200.0, 120.0, 8);
    assert!(has_more_button(&screen));
    screen.get_by_role(Role::Button, "More tabs").click();
    screen.run_until_idle();
    assert!(!screen.all_by_role(Role::MenuItemRadio, "Tab 8").is_empty());
}

#[test]
fn menu_overflow_moves_the_selected_tab_into_more() {
    let (screen, h) = mount(TabOverflow::Menu, Orientation::Horizontal, 200.0, 60.0, 8);
    h.width.set(120.0);
    screen.run_until_idle();
    assert!(!tab_shown(&screen, "Tab 8"));
    screen.get_by_role(Role::Button, "More tabs").click();
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_role(Role::MenuItemRadio, "Tab 8").checked(),
        Some(CheckedState::Checked)
    );
    // Tabs 1 and 2 still fit the strip, so the menu lists the tabs from 3 on.
    screen.get_by_role(Role::MenuItemRadio, "Tab 3").click();
    screen.run_until_idle();
    assert_eq!(h.selected.get(), 3);
}
