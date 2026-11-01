//! Role: Tests for the dock view: panels, tabs, splitters, drags and menus.
//!
//! The dock view: retained panels, tab strip, rename, maximize, splitters, tab
//! drags and container menus. The layout rules are in `dock_model`.

use super::dock_model::{Layout, container_of, editor_layout, weights};
use super::*;
use kivact::draw::Rect;
use kivact::test::Element;
use kivact::view::callback;
use std::cell::Cell;
use ui::{DockAxis, DockNode};

struct Probe {
    layout: Signal<Layout>,
    mounts: Rc<Cell<u32>>,
    closed: Signal<Vec<&'static str>>,
    chosen: Signal<Vec<(&'static str, &'static str)>>,
}

fn mount_dock() -> (Screen, Probe) {
    kivact::test::render_with_handle(crate::options(), || {
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
                chosen,
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

fn ctrl() -> Modifiers {
    Modifiers {
        ctrl: true,
        ..Modifiers::default()
    }
}

#[test]
fn panels_are_created_once_per_key_and_survive_moves_splits_and_maximizing() {
    let (screen, probe) = mount_dock();
    assert_eq!(probe.mounts.get(), 6, "one panel view per tab key");
    screen.get_by_role(Role::Tab, "Text").click();
    screen.get_by_role(Role::Button, "text count 0").click();
    assert!(probe.layout.get_untracked().maximized.is_none());
    screen.get_by_role(Role::Button, "Text panel menu").click();
    screen.get_by_role(Role::MenuItem, "Maximize Panel").click();
    assert_eq!(probe.layout.get_untracked().maximized, Some("text"));
    screen
        .get_by_role(Role::Tab, "Text")
        .press_key_with(Key::Space, ctrl());
    assert_eq!(probe.layout.get_untracked().maximized, None);
    screen.get_by_role(Role::Button, "Text panel menu").click();
    screen.get_by_role(Role::MenuItem, "Split Right").click();
    assert_eq!(
        container_of(&probe.layout.get_untracked(), "text"),
        ["text"]
    );
    assert!(
        screen
            .get_by_role(Role::Button, "text count 1")
            .is_visible(),
        "the panel's own state survived"
    );
    assert_eq!(probe.mounts.get(), 6, "no panel view was created again");
    clean(&screen);
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
    clean(&screen);
}

#[test]
fn f2_and_double_click_rename_in_place_and_escape_cancels() {
    let (screen, probe) = mount_dock();
    screen.get_by_role(Role::Tab, "Viewer 1").double_click();
    let field = screen.get_by_role(Role::TextBox, "Rename panel");
    assert!(field.is_focused());
    field.type_text("Hero");
    screen.press_key(Key::Enter);
    assert_eq!(
        probe.layout.get_untracked().tab(&"viewer").unwrap().title,
        "Hero"
    );
    let hero = screen.get_by_role(Role::Tab, "Hero");
    assert!(hero.is_focused(), "focus returns to the tab");
    hero.press_key(Key::F(2));
    screen
        .get_by_role(Role::TextBox, "Rename panel")
        .type_text("Discarded");
    screen.press_key(Key::Escape);
    assert_eq!(
        probe.layout.get_untracked().tab(&"viewer").unwrap().title,
        "Hero"
    );
    assert!(
        screen
            .query_by_role(Role::TextBox, "Rename panel")
            .is_none()
    );
    screen.get_by_role(Role::Tab, "Hero").press_key(Key::F(2));
    let field = screen.get_by_role(Role::TextBox, "Rename panel");
    assert_eq!(
        field.value().as_deref(),
        Some("Hero"),
        "the title is the draft"
    );
    field.press_key_with(Key::Char('a'), ctrl());
    field.type_text("   ");
    screen.press_key(Key::Enter);
    assert_eq!(
        probe.layout.get_untracked().tab(&"viewer").unwrap().title,
        "Hero",
        "a blank name is not adopted"
    );
    clean(&screen);
}

#[test]
fn maximize_by_icon_double_click_and_ctrl_space_and_escape_restores() {
    let (screen, probe) = mount_dock();
    let viewer = screen.get_by_role(Role::Tab, "Viewer 1");
    let tab_rect = viewer.rect().expect("tab");
    // The icon is the first thing in the tab, after its 8 px padding.
    let icon_at = Point::new(tab_rect.x + 8.0 + 8.0, tab_rect.center().y);
    screen.click_at(icon_at.x, icon_at.y);
    screen.click_at(icon_at.x, icon_at.y);
    assert_eq!(
        probe.layout.get_untracked().maximized,
        Some("viewer"),
        "a double click on the icon maximizes"
    );
    assert!(
        screen
            .query_by_role(Role::TextBox, "Rename panel")
            .is_none(),
        "and does not rename"
    );
    // The maximized container moved the tab; the icon restores it again.
    let tab_rect = screen.get_by_role(Role::Tab, "Viewer 1").rect().unwrap();
    let icon_at = Point::new(tab_rect.x + 8.0 + 8.0, tab_rect.center().y);
    screen.click_at(icon_at.x, icon_at.y);
    screen.click_at(icon_at.x, icon_at.y);
    assert_eq!(probe.layout.get_untracked().maximized, None);

    screen
        .get_by_role(Role::Tab, "Viewer 1")
        .press_key_with(Key::Space, ctrl());
    assert_eq!(probe.layout.get_untracked().maximized, Some("viewer"));
    let hidden = |role: Role, name: &str| {
        screen
            .query_by_role(role, name)
            .is_none_or(|element| !element.is_visible())
    };
    assert!(hidden(Role::Group, "Project 1 container"));
    assert!(hidden(Role::Splitter, "Project 1 and Viewer 1"));
    assert!(hidden(Role::Splitter, "Project 1 and Inspector 1"));
    let full = panel(&screen, "Viewer 1");
    assert!(full.width > 770.0 && full.height > 540.0, "{full:?}");
    screen
        .get_by_role(Role::Tab, "Viewer 1")
        .press_key(Key::Escape);
    assert_eq!(probe.layout.get_untracked().maximized, None);
    assert!(
        screen
            .get_by_role(Role::Group, "Project 1 container")
            .is_visible()
    );
    screen.get_by_role(Role::Tab, "Viewer 1").press_key_with(
        Key::Space,
        Modifiers {
            meta: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(probe.layout.get_untracked().maximized, Some("viewer"));
    screen
        .get_by_role(Role::Tab, "Viewer 1")
        .press_key_with(Key::Space, ctrl());
    assert_eq!(probe.layout.get_untracked().maximized, None);
    screen
        .get_by_role(Role::Button, "Viewer 1 panel menu")
        .click();
    screen.get_by_role(Role::MenuItem, "Maximize Panel").click();
    assert_eq!(probe.layout.get_untracked().maximized, Some("viewer"));
    screen
        .get_by_role(Role::Button, "Viewer 1 panel menu")
        .click();
    screen.get_by_role(Role::MenuItem, "Restore Panel").click();
    assert_eq!(probe.layout.get_untracked().maximized, None);
    clean(&screen);
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
    splitter.a11y_with_value(A11yAction::SetValue, "50");
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
    splitter.press_key(Key::Enter);
    let row = top_row(&probe.layout.get_untracked());
    assert!(
        near(row[0], row[1], 1e-9),
        "Enter equalizes the two neighbors"
    );

    // Double click equalizes too; a drag moves by the pointer's travel.
    splitter.a11y_with_value(A11yAction::SetValue, "30");
    splitter.double_click();
    let row = top_row(&probe.layout.get_untracked());
    assert!(near(row[0], row[1], 1e-9));
    let start = splitter.rect().expect("splitter").center();
    screen.mouse_move(start.x, start.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(start.x + 50.0, start.y);
    assert!((splitter.rect().unwrap().center().x - start.x - 50.0).abs() < 1.5);
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
    clean(&screen);
}

#[test]
fn the_splitter_range_follows_the_window_size_of_the_neighbors() {
    let (screen, _probe) = mount_dock();
    let wide = screen.get_by_role(Role::Splitter, "Project 1 and Viewer 1");
    let (min_row, max_row, _) = wide.value_range();
    let tall = screen.get_by_role(Role::Splitter, "Project 1 and Inspector 1");
    let (min_root, max_root, _) = tall.value_range();
    // 48 px of the two neighbors' length: the longer the pair, the smaller the share.
    let row_pair = 780.0 * 0.75;
    let root_pair = 780.0;
    assert!((min_row.unwrap() - 48.0 / row_pair * 100.0).abs() < 0.5);
    assert!((min_root.unwrap() - 48.0 / root_pair * 100.0).abs() < 0.5);
    assert!((max_row.unwrap() - (100.0 - 48.0 / row_pair * 100.0)).abs() < 0.5);
    assert!((max_root.unwrap() - (100.0 - 48.0 / root_pair * 100.0)).abs() < 0.5);
    clean(&screen);
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
    clean(&screen);
}

#[test]
fn tab_drops_into_a_center_a_strip_index_and_an_outer_edge() {
    let (screen, probe) = mount_dock();
    let viewer = panel(&screen, "Viewer 1");
    drag_to(
        &screen.get_by_role(Role::Tab, "Timeline 1"),
        viewer.center(),
    );
    let layout = probe.layout.get_untracked();
    assert_eq!(container_of(&layout, "viewer"), ["viewer", "timeline"]);
    assert_eq!(
        layout.container(&"viewer").unwrap().active,
        Some("timeline")
    );
    assert_eq!(container_of(&layout, "render"), ["render"]);
    assert!(screen.get_by_role(Role::Tab, "Timeline 1").is_selected());

    let first = screen
        .get_by_role(Role::Tab, "Viewer 1")
        .rect()
        .expect("viewer tab");
    drag_to(
        &screen.get_by_role(Role::Tab, "Project 1"),
        Point::new(first.x + 3.0, first.center().y),
    );
    assert_eq!(
        container_of(&probe.layout.get_untracked(), "viewer"),
        ["project", "viewer", "timeline"]
    );
    assert_eq!(
        container_of(&probe.layout.get_untracked(), "text"),
        ["text"]
    );

    // The outer 16 px of the dock is an edge, over the content.
    let before = probe.layout.get_untracked();
    drag_to(
        &screen.get_by_role(Role::Tab, "Render Manager"),
        Point::new(6.0, 300.0),
    );
    let layout = probe.layout.get_untracked();
    assert_ne!(layout, before);
    let DockNode::Split { children, .. } = &layout.root else {
        panic!("root stays a split");
    };
    assert_eq!(
        children.len(),
        4,
        "a whole-height container joined the merged row"
    );
    assert!(matches!(&children[0].node, DockNode::Tabs(t) if t.contains(&"render")));
    assert!(panel(&screen, "Render Manager").x < 100.0);
    assert!(panel(&screen, "Render Manager").height > 500.0);
    assert_eq!(probe.mounts.get(), 6);
    clean(&screen);
}

#[test]
fn tab_drops_that_change_nothing_or_land_outside_are_dropped() {
    let (screen, probe) = mount_dock();
    let before = probe.layout.get_untracked();
    // On its own container's center.
    let project = panel(&screen, "Project 1");
    drag_to(
        &screen.get_by_role(Role::Tab, "Project 1"),
        project.center(),
    );
    assert_eq!(probe.layout.get_untracked(), before);
    // Beside its own lone container.
    let inspector = panel(&screen, "Inspector 1");
    drag_to(
        &screen.get_by_role(Role::Tab, "Inspector 1"),
        Point::new(inspector.x + inspector.width * 0.1, inspector.center().y),
    );
    assert_eq!(probe.layout.get_untracked(), before);
    // Outside the dock.
    drag_to(
        &screen.get_by_role(Role::Tab, "Timeline 1"),
        Point::new(790.0, 590.0),
    );
    assert_eq!(probe.layout.get_untracked(), before);
    // A pointer that stays within 5 px is not a drag.
    let timeline = screen.get_by_role(Role::Tab, "Timeline 1").rect().unwrap();
    screen.get_by_role(Role::Tab, "Timeline 1").drag(
        Point::new(timeline.width / 2.0, timeline.height / 2.0),
        Point::new(timeline.width / 2.0 + 2.0, timeline.height / 2.0),
    );
    assert_eq!(probe.layout.get_untracked(), before);
    clean(&screen);
}

#[test]
fn escape_cancels_a_tab_drag() {
    let (screen, probe) = mount_dock();
    let before = probe.layout.get_untracked();
    let viewer = panel(&screen, "Viewer 1");
    let tab = screen.get_by_role(Role::Tab, "Timeline 1");
    let from = tab.rect().expect("tab").center();
    screen.mouse_move(from.x, from.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(viewer.center().x, viewer.center().y);
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(probe.layout.get_untracked(), before);
    // The drag can start again afterwards.
    // A second press at the same point in the same instant would be a double click.
    screen.advance(Duration::from_secs(1));
    screen.mouse_move(from.x, from.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(from.x + 40.0, from.y - 40.0);
    screen.mouse_move(viewer.center().x, viewer.center().y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        container_of(&probe.layout.get_untracked(), "viewer"),
        ["viewer", "timeline"]
    );
    clean(&screen);
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
    clean(&screen);
}

#[test]
fn the_container_menu_moves_tabs_without_dragging() {
    let (screen, probe) = mount_dock();
    open(&screen, "Project 1 panel menu");
    choose(&screen, &["Move To", "Inspector 1", "Into Tabs"]);
    assert_eq!(
        container_of(&probe.layout.get_untracked(), "inspector"),
        ["inspector", "project"]
    );

    open(&screen, "Text panel menu");
    choose(&screen, &["Move To", "Timeline 1", "Left"]);
    let layout = probe.layout.get_untracked();
    assert_eq!(container_of(&layout, "text"), ["text"]);
    assert!(panel(&screen, "Text").x < panel(&screen, "Timeline 1").x);

    open(&screen, "Timeline 1 panel menu");
    choose(&screen, &["Split Right"]);
    assert_eq!(
        container_of(&probe.layout.get_untracked(), "timeline"),
        ["timeline"]
    );
    assert_eq!(
        container_of(&probe.layout.get_untracked(), "render"),
        ["render"]
    );

    // The moved tab became the active one, so it names its container's menu.
    open(&screen, "Project 1 panel menu");
    choose(&screen, &["Move Tab Left"]);
    assert_eq!(
        container_of(&probe.layout.get_untracked(), "inspector"),
        ["project", "inspector"]
    );
    open(&screen, "Project 1 panel menu");
    assert!(
        screen
            .get_by_role(Role::MenuItem, "Move Tab Left")
            .is_disabled()
    );
    choose(&screen, &["Move Tab Right"]);
    assert_eq!(
        container_of(&probe.layout.get_untracked(), "inspector"),
        ["inspector", "project"]
    );

    // Every other drag destination, the dock's own edges, is in the menu too.
    open(&screen, "Render Manager panel menu");
    choose(&screen, &["Move To", "Dock Edge", "Bottom"]);
    let layout = probe.layout.get_untracked();
    assert_eq!(layout.tab_ids().last(), Some(&"render"));
    let DockNode::Split { axis, .. } = &layout.root else {
        panic!("root stays a split");
    };
    assert_eq!(*axis, DockAxis::Vertical);
    assert_eq!(probe.mounts.get(), 6);
    clean(&screen);
}

#[test]
fn split_commands_need_two_tabs_and_an_unmaximized_dock() {
    let (screen, probe) = mount_dock();
    open(&screen, "Viewer 1 panel menu");
    for name in ["Split Left", "Split Right", "Split Top", "Split Bottom"] {
        assert!(
            screen.get_by_role(Role::MenuItem, name).is_disabled(),
            "{name}: a lone tab has nothing to split from"
        );
    }
    screen.press_key(Key::Escape);
    open(&screen, "Timeline 1 panel menu");
    for name in ["Split Left", "Split Right", "Split Top", "Split Bottom"] {
        assert!(!screen.get_by_role(Role::MenuItem, name).is_disabled());
    }
    choose(&screen, &["Split Top"]);
    assert_eq!(
        container_of(&probe.layout.get_untracked(), "timeline"),
        ["timeline"],
        "the active tab moved to the new container"
    );
    probe.layout.update(|layout| {
        layout.maximize(&"render");
    });
    screen.run_until_idle();
    open(&screen, "Render Manager panel menu");
    assert!(
        screen
            .get_by_role(Role::MenuItem, "Split Left")
            .is_disabled()
    );
    assert!(screen.get_by_role(Role::MenuItem, "Move To").is_disabled());
    assert_eq!(
        screen
            .get_by_role(Role::MenuItem, "Restore Panel")
            .description(),
        "Ctrl+Space"
    );
    clean(&screen);
}

#[test]
fn application_entries_come_first_and_report_the_active_tab_and_key() {
    let (screen, probe) = mount_dock();
    screen.get_by_role(Role::Tab, "Text").click();
    open(&screen, "Text panel menu");
    let y = |name: &str| {
        screen
            .get_by_role(Role::MenuItem, name)
            .rect()
            .expect("menu item")
            .y
    };
    assert!(y("New Viewer") < y("Arrange"));
    assert!(
        y("Arrange") < y("Close Panel"),
        "the application's entries come before the built-in ones"
    );
    choose(&screen, &["New Viewer"]);
    assert_eq!(probe.chosen.get_untracked(), [("text", "new-viewer")]);
    open(&screen, "Viewer 1 panel menu");
    choose(&screen, &["Arrange", "Tidy up"]);
    assert_eq!(
        probe.chosen.get_untracked(),
        [("text", "new-viewer"), ("viewer", "tidy")]
    );
    assert_eq!(probe.layout.get_untracked(), {
        let mut layout = editor_layout();
        layout.activate(&"text");
        layout
    });
    clean(&screen);
}

#[test]
fn a_dock_without_a_menu_has_only_the_built_in_commands() {
    let (screen, layout) = kivact::test::render_with_handle(crate::options(), || {
        let layout = signal(editor_layout());
        (
            layout,
            ui::dock(layout, |id: &'static str| text(id)).style(style().size(780, 580)),
        )
    });
    open(&screen, "Project 1 panel menu");
    let y = |name: &str| {
        screen
            .get_by_role(Role::MenuItem, name)
            .rect()
            .expect("menu item")
            .y
    };
    assert!(
        screen.query_by_role(Role::MenuItem, "New Viewer").is_none(),
        "no application entries"
    );
    assert!(y("Close Panel") < y("Close Other Panels"));
    choose(&screen, &["Close Panel"]);
    assert_eq!(layout.get_untracked().tab_ids().len(), 5);
    clean(&screen);
}

#[test]
fn the_texts_follow_the_apps_messages() {
    use kivact::i18n::Messages;
    let screen = render_with(
        RenderOptions {
            messages: Messages::new()
                .with("creative.dock.menu", "{title} のメニュー")
                .with("creative.dock.splitter", "{leading} と {trailing}")
                .with("creative.dock.close_panel", "閉じる"),
            theme: ui::theme(kivact::style::ThemeMode::Dark),
            ..RenderOptions::from_env()
        },
        || {
            ui::dock(signal(editor_layout()), |id: &'static str| text(id))
                .style(style().size(780, 580))
        },
    );
    screen.get_by_role(Role::Splitter, "Project 1 と Viewer 1");
    screen
        .get_by_role(Role::Button, "Project 1 のメニュー")
        .click();
    screen.get_by_role(Role::MenuItem, "閉じる");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dock_reports_splitter_edits_without_reporting_tab_commands() {
    use kivact::ui::EditPhase::{Begin, Cancel, Commit};
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (layout, shown)) = kivact::test::render_with_handle(crate::options(), move || {
        let layout = signal(editor_layout());
        let shown = signal(true);
        // The observer belongs to the parent, so disposal can report Cancel.
        let observer = callback(move |phase| {
            seen.borrow_mut()
                .push((phase, top_row(&layout.get_untracked())[0]));
        });
        (
            (layout, shown),
            show(shown, move || {
                ui::dock(layout, |id| text(id))
                    .on_edit(move |value| observer.try_call(value))
                    .style(style().size(780, 580))
            }),
        )
    });
    screen.get_by_role(Role::Tab, "Text").click();
    screen.get_by_role(Role::Tab, "Project 1").click();
    assert!(
        log.borrow().is_empty(),
        "tab activation is outside the splitter edit contract"
    );
    let initial = top_row(&layout.get_untracked())[0];
    screen
        .get_by_role(Role::Splitter, "Project 1 and Viewer 1")
        .press_key(Key::ArrowRight);
    let after_key = top_row(&layout.get_untracked())[0];
    assert!(after_key > initial);
    assert_eq!(log.borrow()[0].0, Begin);
    assert!(near(log.borrow()[0].1, initial, 1e-9));
    assert_eq!(log.borrow()[1].0, Commit);
    assert!(near(log.borrow()[1].1, after_key, 1e-9));
    for mode in 0..3 {
        screen.advance(Duration::from_secs(1));
        let initial = top_row(&layout.get_untracked())[0];
        let start = screen
            .get_by_role(Role::Splitter, "Project 1 and Viewer 1")
            .rect()
            .expect("dock splitter layout")
            .center();
        screen.mouse_move(start.x, start.y);
        screen.mouse_down(MouseButton::Primary);
        assert_eq!(log.borrow().last().expect("Begin").0, Begin);
        assert!(near(log.borrow().last().expect("Begin").1, initial, 1e-9));
        screen.mouse_move(start.x + 20.0, start.y);
        assert!(top_row(&layout.get_untracked())[0] > initial);
        match mode {
            0 => screen.mouse_up(MouseButton::Primary),
            1 => screen.press_key(Key::Escape),
            _ => {
                shown.set(false);
                screen.run_until_idle();
            }
        }
        let current = top_row(&layout.get_untracked())[0];
        if mode > 0 {
            assert!(near(current, initial, 1e-9));
            screen.mouse_up(MouseButton::Primary);
        }
        assert_eq!(
            log.borrow().last().expect("terminal phase").0,
            if mode == 0 { Commit } else { Cancel }
        );
        assert!(near(
            log.borrow().last().expect("terminal phase").1,
            current,
            1e-9
        ));
    }
    assert_eq!(
        log.borrow()
            .iter()
            .map(|(phase, _)| *phase)
            .collect::<Vec<_>>(),
        [Begin, Commit, Begin, Commit, Begin, Cancel, Begin, Cancel]
    );
    clean(&screen);
}
