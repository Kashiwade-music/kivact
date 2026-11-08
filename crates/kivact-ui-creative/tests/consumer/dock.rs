//! Role: Tests for the dock view: panels, tabs, splitters, drags and menus.
//!
//! The dock view: retained panels, tab strip, rename, maximize, splitters, tab
//! drags and container menus. The layout rules are in `dock_model`.

use crate::dock_model::{Layout, container_of, editor_layout, weights};
use crate::support::near;
use kivact::geometry::{Point, Rect};
use kivact::interaction::MenuEntry;
use kivact::prelude::*;
use kivact::test::{Element, Screen};
use kivact::view::{A11yAction, MouseButton};
use kivact_ui_creative as ui;
use std::cell::Cell;
use std::rc::Rc;
use ui::{DockAxis, DockNode};

struct Probe {
    layout: Signal<Layout>,
    mounts: Rc<Cell<u32>>,
    closed: Signal<Vec<&'static str>>,
}

fn mount_dock() -> (Screen, Probe) {
    kivact::test::render_with_handle(crate::support::options(), || {
        let layout = signal(editor_layout());
        let mounts = Rc::new(Cell::new(0));
        let closed = signal(Vec::new());
        let chosen = signal(Vec::new());
        let created = mounts.clone();
        let view = ui::dock(layout, move |id: &'static str| {
            created.set(created.get() + 1);
            let count = signal(0);
            kivact::ui::button(move || format!("{id} count {}", count.get()))
                .variant(kivact::ui::ButtonVariant::Secondary)
                .on_press(move || count.update(|c| *c += 1))
        })
        .menu(
            |_tabs| {
                vec![
                    MenuEntry::item("new-viewer", "New Viewer"),
                    MenuEntry::submenu(
                        "arrange",
                        "Arrange",
                        vec![MenuEntry::item("tidy", "Tidy up")],
                    ),
                ]
            },
            move |choice: (&'static str, &'static str)| chosen.update(|list| list.push(choice)),
        )
        .on_close(move |id: &'static str| closed.update(|list| list.push(id)))
        // Inside the test window's default 800x600 logical size.
        .style(style().size(780, 580));
        (
            Probe {
                layout,
                mounts,
                closed,
            },
            view,
        )
    })
}

/// Drag `element` from its center to the window point `to`.
fn drag_to(element: &Element<'_>, to: Point) {
    let rect = element.rect().expect("dragged element is laid out");
    element.drag(
        Point::new(rect.width / 2.0, rect.height / 2.0),
        Point::new(to.x - rect.x, to.y - rect.y),
    );
}

fn panel(screen: &Screen, title: &str) -> Rect {
    screen
        .get_by_role(Role::TabPanel, title)
        .rect()
        .expect("panel is laid out")
}

#[test]
fn tabs_activate_by_click_arrow_keys_and_invoke_and_publish_roles() {
    let (screen, probe) = mount_dock();
    let list = screen.get_by_role(Role::TabList, "Project 1 tabs");
    assert_eq!(list.role(), Some(Role::TabList));
    screen.get_by_role(Role::Group, "Project 1 container");
    screen.get_by_role(Role::Group, "Dock workspace");
    let project = screen.get_by_role(Role::Tab, "Project 1");
    let text_tab = screen.get_by_role(Role::Tab, "Text");
    assert!(project.is_selected());
    assert!(!text_tab.is_selected());
    assert!(
        !screen
            .get_by_role(Role::TabPanel, "Project 1")
            .rect()
            .is_none()
    );
    project.press_key(Key::ArrowRight);
    assert!(text_tab.is_selected());
    assert_eq!(
        probe
            .layout
            .get_untracked()
            .container(&"project")
            .unwrap()
            .active,
        Some("text")
    );
    assert!(
        screen
            .query_by_role(Role::TabPanel, "Project 1")
            .is_none_or(|p| !p.is_visible()),
        "the inactive panel is hidden"
    );
    screen
        .get_by_role(Role::Tab, "Text")
        .press_key(Key::ArrowLeft);
    assert!(project.is_selected());
    text_tab.a11y(A11yAction::Activate);
    assert!(text_tab.is_selected());
    assert_eq!(probe.mounts.get(), 6);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn top_row(layout: &Layout) -> Vec<f64> {
    let DockNode::Split { children, .. } = &layout.root else {
        panic!("root stays a split");
    };
    let DockNode::Split { children, .. } = &children[0].node else {
        panic!("the left column stays a split");
    };
    weights(&children[0].node)
}

#[test]
fn splitters_set_value_step_reset_and_drag_between_two_neighbors() {
    let (screen, probe) = mount_dock();
    let splitter = screen.get_by_role(Role::Splitter, "Project 1 and Viewer 1");
    assert_eq!(splitter.role(), Some(Role::Splitter));
    assert_eq!(splitter.orientation(), kivact::view::Orientation::Vertical);
    let (min, max, now) = splitter.value_range();
    assert!((now.unwrap() - 30.0).abs() < 1e-6, "{now:?}");
    assert!(min.unwrap() > 0.0 && min.unwrap() < 20.0, "{min:?}");
    assert!(max.unwrap() < 100.0 && max.unwrap() > 80.0, "{max:?}");
    // Equal weights put the splitter in the middle of its row.
    splitter.a11y_set_value("50");
    let row = top_row(&probe.layout.get_untracked());
    assert!(near(row[0], row[1], 1e-9));
    let middle = 780.0 * 0.75 / 2.0;
    assert!((splitter.rect().unwrap().center().x - middle).abs() < 1.0);
    splitter.press_key(Key::ArrowRight);
    let row = top_row(&probe.layout.get_untracked());
    assert!(near(row[0] / (row[0] + row[1]), 0.51, 1e-9), "{row:?}");
    splitter.press_key(Key::ArrowLeft);
    splitter.press_key(Key::ArrowLeft);
    let row = top_row(&probe.layout.get_untracked());
    assert!(near(row[0] / (row[0] + row[1]), 0.49, 1e-9));
    splitter.press_key(Key::ArrowUp);
    assert!(
        near(top_row(&probe.layout.get_untracked())[0], row[0], 1e-9),
        "the vertical arrows are not used by a side-by-side splitter"
    );
    splitter.press_key(Key::Home);
    let row = top_row(&probe.layout.get_untracked());
    let share = row[0] / (row[0] + row[1]);
    let min_share = splitter.value_range().0.unwrap() / 100.0;
    assert!(
        near(share, min_share, 1e-9),
        "Home keeps 48 px for the leading pane"
    );
    let pane = panel(&screen, "Project 1");
    assert!(pane.width > 39.0 && pane.width < 48.0, "{pane:?}");
    splitter.press_key(Key::End);
    let row = top_row(&probe.layout.get_untracked());
    let share = row[0] / (row[0] + row[1]);
    assert!(near(share, splitter.value_range().1.unwrap() / 100.0, 1e-9));
    assert!(panel(&screen, "Viewer 1").width > 39.0);
    let expanded = row;
    splitter.press_key(Key::Enter);
    let row = top_row(&probe.layout.get_untracked());
    assert!(
        near(row[0] / (row[0] + row[1]), min_share, 1e-9),
        "Enter collapses to the minimum"
    );
    assert!(near(row[0] + row[1], expanded[0] + expanded[1], 1e-9));
    splitter.press_key(Key::Enter);
    let restored = top_row(&probe.layout.get_untracked());
    assert!(near(restored[0], expanded[0], 1e-9));
    assert!(
        near(restored[1], expanded[1], 1e-9),
        "Enter restores both neighbors"
    );

    assert_splitter_pointer_changes(&screen, &probe, &splitter);
    // A stacked splitter uses the vertical arrows.
    let stacked = screen.get_by_role(Role::Splitter, "Project 1 and Timeline 1");
    assert_eq!(stacked.orientation(), kivact::view::Orientation::Horizontal);
    stacked.press_key(Key::ArrowDown);
    stacked.press_key(Key::ArrowDown);
    let layout = probe.layout.get_untracked();
    let DockNode::Split { children, .. } = &layout.root else {
        panic!("root stays a split");
    };
    let column = weights(&children[0].node);
    assert!(
        near(column[0] / (column[0] + column[1]), 0.62, 1e-9),
        "{column:?}"
    );
    assert_eq!(probe.mounts.get(), 6);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tab_drags_split_right_and_bottom_and_keep_panels_mounted() {
    let (screen, probe) = mount_dock();
    screen.get_by_role(Role::Tab, "Text").click();
    screen.get_by_role(Role::Button, "text count 0").click();
    let viewer = panel(&screen, "Viewer 1");
    drag_to(
        &screen.get_by_role(Role::Tab, "Text"),
        Point::new(viewer.x + viewer.width * 0.92, viewer.center().y),
    );
    let layout = probe.layout.get_untracked();
    assert_eq!(container_of(&layout, "text"), ["text"]);
    assert_eq!(
        layout.tab_ids(),
        [
            "project",
            "viewer",
            "text",
            "timeline",
            "render",
            "inspector"
        ]
    );
    assert!(
        screen
            .get_by_role(Role::Button, "text count 1")
            .is_visible()
    );
    assert!(panel(&screen, "Text").x > viewer.x);

    let inspector = panel(&screen, "Inspector 1");
    drag_to(
        &screen.get_by_role(Role::Tab, "Render Manager"),
        Point::new(inspector.center().x, inspector.y + inspector.height * 0.92),
    );
    let layout = probe.layout.get_untracked();
    let DockNode::Split { children, .. } = &layout.root else {
        panic!("root stays a split: {layout:?}");
    };
    assert!(matches!(
        &children[1].node,
        DockNode::Split { axis: DockAxis::Vertical, children } if children.len() == 2
    ));
    assert_eq!(container_of(&layout, "render"), ["render"]);
    assert!(panel(&screen, "Render Manager").y > panel(&screen, "Inspector 1").y);
    assert_eq!(probe.mounts.get(), 6, "moves retain every panel view");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn open(screen: &Screen, button: &str) {
    screen.get_by_role(Role::Button, button).click();
}

fn choose(screen: &Screen, path: &[&str]) {
    let (last, parents) = path.split_last().expect("a path");
    for parent in parents {
        screen
            .get_by_role(Role::MenuItem, parent)
            .a11y(A11yAction::Activate);
    }
    screen.get_by_role(Role::MenuItem, last).click();
}

#[test]
fn close_buttons_and_commands_remove_closable_tabs_and_report_each() {
    let (screen, probe) = mount_dock();
    assert!(
        screen
            .query_by_role(Role::Button, "Close Inspector 1")
            .is_none_or(|close| !close.is_visible()),
        "a tab that is not closable has no close button"
    );
    screen
        .get_by_role(Role::Button, "Close Render Manager")
        .click();
    screen.get_by_role(Role::Button, "Close Timeline 1").click();
    assert_eq!(probe.closed.get_untracked(), ["render", "timeline"]);
    let layout = probe.layout.get_untracked();
    let DockNode::Split { children, .. } = &layout.root else {
        panic!("root stays a split");
    };
    assert_eq!(
        children.len(),
        3,
        "the emptied column collapsed into the row"
    );
    assert_eq!(probe.mounts.get(), 6, "closing creates no panel views");

    open(&screen, "Project 1 panel menu");
    choose(&screen, &["Close Other Panels"]);
    assert_eq!(probe.closed.get_untracked(), ["render", "timeline", "text"]);
    open(&screen, "Inspector 1 panel menu");
    assert!(
        screen
            .get_by_role(Role::MenuItem, "Close Panel")
            .is_disabled(),
        "the inspector cannot be closed"
    );
    assert!(
        screen
            .get_by_role(Role::MenuItem, "Close Container")
            .is_disabled()
    );
    screen.press_key(Key::Escape);
    open(&screen, "Project 1 panel menu");
    choose(&screen, &["Close Panel"]);
    assert_eq!(
        probe.closed.get_untracked(),
        ["render", "timeline", "text", "project"]
    );
    assert_eq!(
        probe.layout.get_untracked().tab_ids(),
        ["viewer", "inspector"]
    );

    // Close Container closes every closable tab of the container.
    probe.layout.set(editor_layout());
    screen.run_until_idle();
    probe.closed.set(Vec::new());
    open(&screen, "Project 1 panel menu");
    choose(&screen, &["Close Container"]);
    assert_eq!(probe.closed.get_untracked(), ["project", "text"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn assert_splitter_pointer_changes(screen: &Screen, probe: &Probe, splitter: &Element<'_>) {
    // Double click equalizes; a drag moves by the pointer's travel.
    splitter.a11y_set_value("30");
    splitter.double_click();
    let row = top_row(&probe.layout.get_untracked());
    assert!(near(row[0], row[1], 1e-9));
    let start = splitter.rect().expect("splitter").center();
    screen.mouse_move(start.x, start.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(start.x + 50.0, start.y);
    assert!(
        (splitter
            .rect()
            .expect("splitter stays laid out during drag")
            .center()
            .x
            - start.x
            - 50.0)
            .abs()
            < 1.5
    );
    screen.mouse_up(MouseButton::Primary);
    let moved = probe.layout.get_untracked();
    assert!(top_row(&moved)[0] > top_row(&moved)[1]);
    // Escape restores the weights of the drag's start.
    let start = splitter.rect().expect("splitter").center();
    screen.mouse_move(start.x, start.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(start.x - 80.0, start.y);
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(probe.layout.get_untracked(), moved);
}

#[test]
fn a_tab_press_that_stays_inside_the_threshold_neither_drags_nor_captures() {
    let (screen, probe) = mount_dock();
    // A click on a tab activates it; activate Text first so the press below
    // tests only that a press inside the slop neither drags nor captures.
    screen.get_by_role(Role::Tab, "Text").click();
    let before = probe.layout.get_untracked();
    let at = screen
        .get_by_role(Role::Tab, "Text")
        .rect()
        .expect("laid out")
        .center();
    screen.mouse_move(at.x, at.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(at.x + 3.0, at.y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        probe.layout.get_untracked(),
        before,
        "no move inside the slop"
    );
    // Never captured: a later move past the threshold is an ordinary hover.
    screen.mouse_move(at.x + 60.0, at.y);
    assert_eq!(probe.layout.get_untracked(), before);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
