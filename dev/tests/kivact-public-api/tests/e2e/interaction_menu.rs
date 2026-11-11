//! Role: Tests for menu and context menu interactions.
//!
//! The unstyled `Menu` from a button and as a context menu: opening, keys,
//! typeahead, submenus, choosing, focus return, changing entries and semantics.

use kivact::interaction::MenuEntry;
use kivact::view::View;
use std::time::Duration;

use crate::support::mount_with;
use kivact::geometry::Point;
use kivact::interaction::{MenuApi, MenuItemKind, MenuOrigin, MenuRow, use_menu};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{Locator, MirrorMode, RenderOptions, Screen};
use kivact::view::{A11yAction, Placement, overlay};
use kivact::view::{Modifiers, MouseButton, overlay_at};

type Entry = MenuEntry<&'static str>;

fn real_window() -> RenderOptions {
    let policy = kivact::interaction::InteractionPolicy {
        menu: kivact::interaction::MenuPolicy {
            disabled_focus: kivact::interaction::DisabledMenuFocus::Skip,
            ..Default::default()
        },
        ..Default::default()
    };
    RenderOptions::from_env()
        .mirror(MirrorMode::Required)
        .theme(kivact::style::Theme::default().with(policy))
}

fn action(name: &'static str) -> Node {
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .child(text(name))
        .style(style().size(120, 28))
}

fn edit_entries() -> Vec<Entry> {
    vec![
        Entry::item("cut", "Cut")
            .with_shortcut("Ctrl X")
            .disabled(true),
        Entry::item("copy", "Copy").with_shortcut("Ctrl C"),
        Entry::separator(),
        Entry::item("paste", "Paste"),
        Entry::item("delete", "Delete").disabled(true),
        Entry::item("all", "Select all"),
    ]
}

fn nested_entries() -> Vec<Entry> {
    vec![
        Entry::item("new", "New"),
        Entry::submenu(
            "export",
            "Export",
            vec![
                Entry::item("png", "PNG"),
                Entry::item("svg", "SVG").disabled(true),
                Entry::item("pdf", "PDF"),
            ],
        ),
        Entry::item("quit", "Quit"),
    ]
}

/// The rows of one level: items (with their submenus) and separators.
fn level<K: ItemKey>(m: MenuApi<K>) -> Node {
    let rows = for_each(
        m.entries(),
        |(row, _)| row.clone(),
        move |entry| {
            let row = entry.with(|(row, _)| row.clone());
            match row {
                MenuRow::Separator(_) | MenuRow::Label(_) => node()
                    .role(Role::Separator)
                    .style(style().height(1).margin_y(4))
                    .into_view(),
                MenuRow::Item { key, kind } => {
                    let submenu = kind == MenuItemKind::Submenu;
                    let label = memo(move || entry.with(|(_, entry)| entry.label()));
                    let part = m.item(key.clone());
                    let anchor = node_ref();
                    let row = node()
                        .part(part)
                        .node_ref(anchor)
                        .style(style().height(28).padding_x(8).align_items(Align::Center))
                        .child(text(label));
                    if submenu {
                        let sub = m.submenu(key);
                        row.child(sub.while_open(move || {
                            overlay(anchor, Placement::End, level(sub))
                                .with_node(move |node| node.part(sub.content()))
                                .name(label)
                        }))
                        .into_view()
                    } else {
                        row.into_view()
                    }
                }
            }
        },
    );
    column(rows).style(style().min_width(160).padding(4))
}

/// A menu button named `name` and its menu, named like it.
pub(super) fn menu_button<K: ItemKey>(name: &'static str, m: MenuApi<K>) -> View {
    let trigger = m.trigger();
    let anchor = node_ref();
    (
        action(name).part(trigger).node_ref(anchor),
        m.while_open(move || {
            overlay(anchor, Placement::Below, level(m))
                .offset(4)
                .with_node(move |node| node.part(m.content()))
                .name(name)
        }),
    )
        .into_view()
}

/// `area` opening `m` as a context menu named `name`: at the pointer for a
/// right click, below the area otherwise.
fn context_area<K: ItemKey>(name: &'static str, area: Node, m: MenuApi<K>) -> View {
    let trigger = m.context_trigger();
    let anchor = node_ref();
    let origin = m.origin();
    (
        area.part(trigger).node_ref(anchor),
        m.while_open(move || {
            let content = level(m);
            let placed = match origin.get_untracked() {
                Some(MenuOrigin::Pointer(point)) => overlay_at(point, Placement::Below, content),
                _ => overlay(anchor, Placement::Below, content),
            };
            placed
                .with_node(move |node| node.part(m.content()))
                .name(name)
        }),
    )
        .into_view()
}

/// "Edit" opens the menu; "After" is the next tab stop after it.
fn mount_button(
    entries: impl IntoValue<Vec<Entry>> + 'static,
) -> (Screen, Signal<Vec<&'static str>>) {
    mount_with(real_window(), move || {
        let chosen = signal(Vec::new());
        let edit = use_menu(entries)
            .on_activate(move |key: &'static str| chosen.update(|chosen| chosen.push(key)));
        (chosen, column((menu_button("Edit", edit), action("After"))))
    })
}

fn item<'a>(screen: &'a Screen, name: &str) -> Locator<'a> {
    screen.get_by_role(Role::MenuItem, name)
}

fn gone(screen: &Screen, name: &str) -> bool {
    screen.query_by_role(Role::MenuItem, name).is_none()
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

fn center(element: &Locator<'_>) -> Point {
    let rect = element.rect().expect("invariant: the element is laid out");
    Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1.5
}

#[test]
fn a_menu_button_opens_on_its_first_or_last_enabled_item_and_walks_over_enabled_items() {
    let (screen, chosen) = mount_button(edit_entries());
    let trigger = screen.get_by_role(Role::Button, "Edit");
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());

    trigger.press_key(Key::ArrowDown);
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(screen.get_by_role(Role::Menu, "Edit").exists());
    assert!(item(&screen, "Copy").is_focused(), "the first enabled item");
    screen.press_key(Key::ArrowDown);
    assert!(
        item(&screen, "Paste").is_focused(),
        "the separator is skipped"
    );
    screen.press_key(Key::ArrowDown);
    assert!(
        item(&screen, "Select all").is_focused(),
        "the disabled item is skipped"
    );
    screen.press_key(Key::ArrowDown);
    assert!(item(&screen, "Copy").is_focused(), "wraps at the end");
    screen.press_key(Key::ArrowUp);
    assert!(
        item(&screen, "Select all").is_focused(),
        "wraps at the start"
    );
    screen.press_key(Key::Home);
    assert!(item(&screen, "Copy").is_focused());
    screen.press_key(Key::End);
    assert!(item(&screen, "Select all").is_focused());

    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(
        trigger.is_focused(),
        "Escape gives focus back to the trigger"
    );

    trigger.press_key(Key::ArrowUp);
    assert!(
        item(&screen, "Select all").is_focused(),
        "ArrowUp opens on the last enabled item"
    );
    screen.press_key(Key::Escape);

    trigger.press_key(Key::Enter);
    assert!(item(&screen, "Copy").is_focused());
    screen.press_key(Key::Escape);
    trigger.press_key(Key::Space);
    assert!(item(&screen, "Copy").is_focused());
    screen.press_key(Key::Escape);

    trigger.click();
    assert!(item(&screen, "Copy").is_focused(), "a press opens it");
    trigger.click();
    assert!(
        screen.query_by_role(Role::Menu, "Edit").is_none(),
        "a second press closes it"
    );
    trigger.a11y(A11yAction::Activate);
    assert!(item(&screen, "Copy").is_focused(), "so does Activate");
    trigger.a11y(A11yAction::Collapse);
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    assert!(trigger.is_focused());

    assert!(chosen.get_untracked().is_empty());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn right_and_left_move_between_levels_and_escape_closes_one_level_at_a_time() {
    let (screen, chosen) = mount_button(nested_entries());
    let trigger = screen.get_by_role(Role::Button, "Edit");
    trigger.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    let export = item(&screen, "Export");
    assert!(export.is_focused());
    assert_eq!(export.is_expanded(), Some(false));
    assert!(gone(&screen, "PNG"));

    screen.press_key(Key::ArrowRight);
    assert_eq!(export.is_expanded(), Some(true));
    assert!(
        item(&screen, "PNG").is_focused(),
        "Right focuses the first enabled item of the submenu"
    );
    screen.press_key(Key::ArrowDown);
    assert!(item(&screen, "PDF").is_focused(), "SVG is disabled");
    screen.press_key(Key::ArrowLeft);
    assert!(export.is_focused(), "Left returns to the parent item");
    assert!(gone(&screen, "PNG"));
    assert_eq!(export.is_expanded(), Some(false));

    screen.press_key(Key::ArrowRight);
    assert!(item(&screen, "PNG").is_focused());
    screen.press_key(Key::Escape);
    assert!(gone(&screen, "PNG"), "Escape closes only the submenu");
    assert!(export.is_focused());
    assert!(screen.get_by_role(Role::Menu, "Edit").exists());
    screen.press_key(Key::ArrowDown);
    assert!(item(&screen, "Quit").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(
        item(&screen, "Quit").is_focused(),
        "Right on a plain item does nothing"
    );
    screen.press_key(Key::ArrowUp);
    screen.press_key(Key::Enter);
    assert!(
        item(&screen, "PNG").is_focused(),
        "Enter on a submenu item opens it instead of choosing"
    );
    assert!(chosen.get_untracked().is_empty());
    screen.press_key(Key::Escape);
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    assert!(trigger.is_focused(), "Escape at the top closes the menu");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn choosing_calls_back_with_the_key_closes_every_level_and_returns_the_focus() {
    let (screen, chosen) = mount_button(nested_entries());
    let trigger = screen.get_by_role(Role::Button, "Edit");

    trigger.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), ["pdf"]);
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    assert!(gone(&screen, "PDF"), "every level is closed");
    assert!(trigger.is_focused());

    trigger.click();
    // The pointer rests on an item before it presses, as a hand does: the
    // hover is what makes the item the one tab stop.
    item(&screen, "Quit").hover();
    item(&screen, "Quit").click();
    assert_eq!(chosen.get_untracked(), ["pdf", "quit"]);
    assert!(trigger.is_focused(), "a click returns focus too");

    trigger.click();
    item(&screen, "New").a11y(A11yAction::Select);
    assert_eq!(chosen.get_untracked(), ["pdf", "quit", "new"]);
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());

    trigger.press_key(Key::ArrowDown);
    item(&screen, "Export").hover();
    screen.advance(Duration::from_millis(140));
    let svg = item(&screen, "SVG");
    assert!(svg.is_disabled());
    assert!(
        svg.a11y_actions().is_empty(),
        "a disabled item offers no action"
    );
    svg.a11y(A11yAction::Activate);
    assert_eq!(
        chosen.get_untracked().len(),
        3,
        "a disabled item is not chosen"
    );
    assert!(!gone(&screen, "SVG"));
    item(&screen, "Export").a11y(A11yAction::Activate);
    assert_eq!(
        chosen.get_untracked().len(),
        3,
        "a submenu item is not chosen"
    );
    assert!(item(&screen, "PNG").is_focused(), "it opens instead");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn roles_checked_expanded_description_and_the_separator_are_published() {
    let (screen, _) = mount_button(vec![
        Entry::item("cut", "Cut").with_shortcut("Ctrl X"),
        Entry::checkbox("grid", "Show grid", true),
        Entry::checkbox("snap", "Snap", false),
        Entry::separator(),
        Entry::submenu("more", "More", vec![Entry::item("x", "Xray")]),
    ]);
    screen
        .get_by_role(Role::Button, "Edit")
        .press_key(Key::ArrowDown);
    let menu = screen.get_by_role(Role::Menu, "Edit");
    assert_eq!(menu.role(), Some(Role::Menu));

    let cut = item(&screen, "Cut");
    assert_eq!(
        cut.description(),
        "Ctrl X",
        "the shortcut is the description"
    );
    assert_eq!(cut.checked(), None);
    assert_eq!(cut.is_expanded(), None);
    assert_eq!(
        screen
            .get_by_role(Role::MenuItemCheckBox, "Show grid")
            .description(),
        ""
    );

    let grid = screen.get_by_role(Role::MenuItemCheckBox, "Show grid");
    assert_eq!(grid.checked(), Some(CheckedState::Checked));
    assert_eq!(
        screen.get_by_role(Role::MenuItemCheckBox, "Snap").checked(),
        Some(CheckedState::Unchecked)
    );
    assert!(screen.query_by_role(Role::MenuItem, "Show grid").is_none());

    assert_eq!(screen.all_by_role(Role::Separator, "").len(), 1);
    let more = item(&screen, "More");
    assert_eq!(more.is_expanded(), Some(false));
    more.hover();
    more.click();
    assert_eq!(more.is_expanded(), Some(true));
    assert_eq!(
        screen.get_by_role(Role::Menu, "More").role(),
        Some(Role::Menu),
        "the submenu is a menu named after its item"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

struct ContextRig {
    open: Value<bool>,
    origin: Value<Option<MenuOrigin>>,
}

/// An area with a focusable child, and the edit menu it opens.
fn mount_context() -> (Screen, ContextRig) {
    mount_with(real_window(), || {
        let edit = use_menu(edit_entries()).on_activate(|_: &'static str| ());
        let area = node()
            .role(Role::Group)
            .name("Canvas")
            .focusable(true)
            .style(style().size(260, 160).margin(60))
            .child(action("Inner"));
        (
            ContextRig {
                open: edit.is_open(),
                origin: edit.origin(),
            },
            context_area("Canvas", area, edit),
        )
    })
}

#[test]
fn four_entrances_open_a_context_menu_at_the_pointer_or_at_the_area() {
    let (screen, rig) = mount_context();
    let area = screen.get_by_role(Role::Group, "Canvas");
    let inner = screen.get_by_role(Role::Button, "Inner");
    let rect = area.rect().expect("invariant: the area is laid out");
    let middle = center(&area);
    assert_eq!(area.is_expanded(), None, "the area is not a disclosure");
    assert_eq!(rig.origin.get_untracked(), None);

    // A right click: on release, except on macOS where the press opens it.
    screen.mouse_move(middle.x, middle.y);
    screen.mouse_down(MouseButton::Secondary);
    if !cfg!(target_os = "macos") {
        assert!(
            !rig.open.get_untracked(),
            "not before the button is released"
        );
        screen.mouse_up(MouseButton::Secondary);
    }
    assert!(rig.open.get_untracked());
    match rig.origin.get_untracked() {
        Some(MenuOrigin::Pointer(at)) => {
            assert!(near(at.x, middle.x) && near(at.y, middle.y), "{at:?}");
        }
        other => panic!("expected the pointer origin, got {other:?}"),
    }
    let content = screen.get_by_role(Role::Menu, "Canvas");
    let at = content.rect().expect("invariant: the menu is laid out");
    assert!(
        near(at.x, middle.x) && near(at.y, middle.y),
        "at the pointer"
    );
    if cfg!(target_os = "macos") {
        screen.mouse_up(MouseButton::Secondary);
    }
    screen.press_key(Key::Escape);
    assert!(!rig.open.get_untracked());
    assert_eq!(rig.origin.get_untracked(), None, "closed again");

    // Shift+F10, from a descendant that has the focus.
    inner.focus();
    screen.press_key_with(Key::F(10), shift());
    assert!(rig.open.get_untracked());
    assert_eq!(rig.origin.get_untracked(), Some(MenuOrigin::Anchor));
    let at = screen
        .get_by_role(Role::Menu, "Canvas")
        .rect()
        .expect("invariant: the menu is laid out");
    assert!(
        near(at.x, rect.x) && near(at.y, rect.y + rect.height),
        "below the area: {at:?} vs {rect:?}"
    );
    screen.press_key(Key::Escape);
    assert!(!rig.open.get_untracked());
    assert!(inner.is_focused(), "Escape returns to who had the focus");

    // The Menu key.
    screen.press_key(Key::ContextMenu);
    assert_eq!(rig.origin.get_untracked(), Some(MenuOrigin::Anchor));
    screen.press_key(Key::Escape);
    assert!(!rig.open.get_untracked());

    // Assistive technology.
    area.a11y(A11yAction::ShowMenu);
    assert_eq!(rig.origin.get_untracked(), Some(MenuOrigin::Anchor));
    screen.press_key(Key::Escape);
    assert!(!rig.open.get_untracked());

    // ArrowDown and plain F10 are not entrances.
    inner.focus();
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::F(10));
    assert!(!rig.open.get_untracked());
    assert_eq!(area.is_expanded(), None);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_submenu_item_the_pointer_rests_on_during_a_held_press_opens_after_the_delay() {
    let (screen, _chosen) = mount_with(real_window(), || {
        let chosen = signal(Vec::new());
        let file = use_menu(nested_entries())
            .on_activate(move |key: &'static str| chosen.update(|c| c.push(key)));
        (chosen, column((menu_button("File", file), action("After"))))
    });
    screen
        .get_by_role(Role::Button, "File")
        .press_key(Key::ArrowDown);
    let new = center(&item(&screen, "New"));
    let export = center(&item(&screen, "Export"));
    // A primary press held from another item, then a rest on the submenu item.
    screen.mouse_move(new.x, new.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(export.x, export.y);
    screen.advance(Duration::from_millis(200)); // > SubmenuTiming::default().open (140 ms)
    assert_eq!(
        item(&screen, "Export").is_expanded(),
        Some(true),
        "engine hover opens the submenu while the press is held"
    );
    assert!(
        item(&screen, "Export").is_focused(),
        "the hovered item takes the focus"
    );
    screen.mouse_up(MouseButton::Primary);
    assert!(screen.get_by_role(Role::Menu, "Export").exists());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
