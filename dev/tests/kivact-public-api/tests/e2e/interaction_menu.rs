//! Role: Tests for menu and context menu interactions.
//!
//! The unstyled `Menu` from a button and as a context menu: opening, keys,
//! typeahead, submenus, choosing, focus return, changing entries and semantics.

use kivact::primitive::NodeExt;
use kivact::ui::UiRecipes;
use kivact::ui::{ListItem, Look, MenuEntry};
use kivact::view::View;
use std::hash::Hash;
use std::time::Duration;

use crate::support::mount_with;
use kivact::draw::Point;
use kivact::interaction::{
    ListPolicy, ListSelection, MenuApi, MenuOrigin, MenuRow, use_list, use_menu,
};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{Element, RenderOptions, Screen, WindowMode};
use kivact::view::{A11yAction, Placement, overlay};
use kivact::view::{Modifiers, MouseButton, overlay_at};

mod entries;

type Entry = MenuEntry<&'static str>;

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        ..RenderOptions::from_env()
    }
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
        Entry::item("cut", "Cut").shortcut("Ctrl X").disabled(true),
        Entry::item("copy", "Copy").shortcut("Ctrl C"),
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
fn level<K: Clone + Eq + Hash + 'static>(m: MenuApi<K>) -> Node {
    let rows = for_each(
        m.entries(),
        |(row, _)| row.clone(),
        move |row, entry| match row {
            MenuRow::Separator(_) | MenuRow::Label(_) => node()
                .role(Role::Separator)
                .style(style().height(1).margin_y(4))
                .into_view(),
            MenuRow::Item { key, submenu, .. } => {
                let label = memo(move || entry.with(|(_, entry)| entry.label()));
                let part = m.item(key.clone());
                let anchor = part.node_ref();
                let row = node()
                    .part(part)
                    .style(style().height(28).padding_x(8).align_items(Align::Center))
                    .child(text(label));
                if submenu {
                    let sub = m.submenu(key);
                    row.child(sub.show(move || {
                        overlay(anchor, Placement::End, level(sub))
                            .part(sub.content())
                            .name(label)
                    }))
                    .into_view()
                } else {
                    row.into_view()
                }
            }
        },
    );
    column(rows).style(style().min_width(160).padding(4))
}

/// A menu button named `name` and its menu, named like it.
fn menu_button<K: Clone + Eq + Hash + 'static>(name: &'static str, m: MenuApi<K>) -> View {
    let trigger = m.trigger();
    let anchor = trigger.node_ref();
    (
        action(name).part(trigger),
        m.show(move || {
            overlay(anchor, Placement::Below, level(m))
                .offset(4)
                .part(m.content())
                .name(name)
        }),
    )
        .into_view()
}

/// `area` opening `m` as a context menu named `name`: at the pointer for a
/// right click, below the area otherwise.
fn context_area<K: Clone + Eq + Hash + 'static>(
    name: &'static str,
    area: Node,
    m: MenuApi<K>,
) -> View {
    let trigger = m.context_trigger();
    let anchor = trigger.node_ref();
    let origin = m.origin();
    (
        area.part(trigger),
        m.show(move || {
            let content = level(m);
            let placed = match origin.get_untracked() {
                Some(MenuOrigin::Pointer(point)) => overlay_at(point, Placement::Below, content),
                _ => overlay(anchor, Placement::Below, content),
            };
            placed.part(m.content()).name(name)
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
        let edit = use_menu(entries, move |key: &'static str| {
            chosen.update(|chosen| chosen.push(key))
        });
        (chosen, column((menu_button("Edit", edit), action("After"))))
    })
}

fn item<'a>(screen: &'a Screen, name: &str) -> Element<'a> {
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

fn center(element: &Element<'_>) -> Point {
    let rect = element.rect().expect("invariant: the element is laid out");
    Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

fn near(a: f32, b: f32) -> bool {
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
fn letters_move_to_the_next_label_that_starts_with_them_and_join_within_500_ms() {
    let (screen, _) = mount_button(vec![
        Entry::item("apple", "Apple"),
        Entry::item("apricot", "Apricot"),
        Entry::item("banana", "Banana"),
        Entry::item("blueberry", "Blueberry"),
        Entry::item("cherry", "Cherry").disabled(true),
        Entry::item("date", "Date"),
    ]);
    screen
        .get_by_role(Role::Button, "Edit")
        .press_key(Key::ArrowDown);
    assert!(item(&screen, "Apple").is_focused());

    screen.press_key(Key::Char('b'));
    assert!(item(&screen, "Banana").is_focused());
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('b'));
    assert!(
        item(&screen, "Blueberry").is_focused(),
        "after a pause one letter searches after the focused item"
    );
    screen.press_key(Key::Char('l'));
    assert!(
        item(&screen, "Blueberry").is_focused(),
        "\"bl\" within 500 ms is one prefix"
    );
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('c'));
    assert!(
        item(&screen, "Blueberry").is_focused(),
        "the disabled Cherry never matches"
    );
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('a'));
    assert!(item(&screen, "Apple").is_focused(), "it wraps around");
    screen.press_key(Key::Char('p'));
    screen.press_key(Key::Char('r'));
    assert!(
        item(&screen, "Apricot").is_focused(),
        "\"apr\" typed in a row"
    );
    screen.advance(Duration::from_millis(400));
    screen.press_key(Key::Char('i'));
    assert!(
        item(&screen, "Apricot").is_focused(),
        "\"apri\" still matches within 500 ms of the last letter"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// List and Menu use the same private search policy.
#[test]
fn list_typeahead_joins_cycles_and_skips_disabled_items() {
    let (screen, _) = mount_with(real_window(), || {
        let items = vec![
            ListItem::new("apple", "Apple"),
            ListItem::new("apricot", "Apricot"),
            ListItem::new("avocado", "AVOCADO").disabled(true),
            ListItem::new("banana", "Banana"),
            ListItem::new("blueberry", "Blueberry"),
        ];
        let list = use_list(items.clone(), ListSelection::Single(signal(None).into())).policy(
            ListPolicy {
                typeahead: true,
                ..ListPolicy::default()
            },
        );
        (
            (),
            column(
                items
                    .into_iter()
                    .map(|item| {
                        node()
                            .part(list.item(item.key()))
                            .style(style().size(120, 28))
                    })
                    .collect::<Vec<_>>(),
            )
            .part(list.list()),
        )
    });
    let focused = |label: &str| screen.get_by_role(Role::Option, label).is_focused();
    screen.press_tab(false);
    assert!(focused("Apple"));
    screen.press_key(Key::Char('A'));
    assert!(focused("Apricot"));
    screen.advance(Duration::from_millis(100));
    screen.press_key(Key::Char('a'));
    assert!(focused("Apple"), "repeat cycles and skips disabled Avocado");
    screen.get_by_role(Role::Option, "Apricot").click();
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('a'));
    assert!(focused("Apple"));
    screen.press_key(Key::Char('p'));
    assert!(focused("Apple"), "joined prefix includes the current item");
    screen.press_key(Key::Char('r'));
    assert!(focused("Apricot"));
    screen.advance(Duration::from_millis(500));
    screen.press_key(Key::Char('b'));
    assert!(focused("Apricot"), "exactly 500ms still joins the prefix");
    screen.advance(Duration::from_millis(501));
    screen.press_key(Key::Char('b'));
    assert!(focused("Banana"), "after 500ms a new prefix starts");
    screen.press_key(Key::Char('b'));
    assert!(focused("Blueberry"));
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('z'));
    assert!(focused("Blueberry"), "missing labels leave focus unchanged");
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
fn hover_focuses_an_item_and_opens_a_submenu_after_140_ms() {
    let (screen, chosen) = mount_button(nested_entries());
    screen.get_by_role(Role::Button, "Edit").click();
    assert!(item(&screen, "New").is_focused());

    item(&screen, "Quit").hover();
    assert!(item(&screen, "Quit").is_focused(), "hover moves the focus");
    let export = item(&screen, "Export");
    export.hover();
    assert!(export.is_focused());
    screen.advance(Duration::from_millis(139));
    assert!(gone(&screen, "PNG"), "not yet");
    screen.advance(Duration::from_millis(1));
    assert!(!gone(&screen, "PNG"), "opened at 140 ms");
    assert!(export.is_focused(), "opening by hover leaves the focus");
    assert_eq!(export.is_expanded(), Some(true));

    item(&screen, "PDF").hover();
    assert!(item(&screen, "PDF").is_focused());
    item(&screen, "Quit").hover();
    assert!(gone(&screen, "PNG"), "a plain item closes the submenu");
    assert!(item(&screen, "Quit").is_focused());

    export.hover();
    screen.advance(Duration::from_millis(100));
    item(&screen, "New").hover();
    screen.advance(Duration::from_millis(200));
    assert!(gone(&screen, "PNG"), "leaving early cancels the deadline");

    export.hover();
    export.click();
    assert!(
        item(&screen, "PNG").is_focused(),
        "a click opens the submenu at once"
    );
    assert!(chosen.get_untracked().is_empty());
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
fn a_failing_callback_is_reported_once_and_keeps_the_menu_open_for_a_retry() {
    let (screen, attempts) = mount_with(real_window(), || {
        let attempts = signal(0usize);
        let actions = use_menu(vec![Entry::item("save", "Save")], move |_: &'static str| {
            attempts.update(|count| *count += 1);
            if attempts.get_untracked() == 1 {
                Err(UiError::msg("save failed"))
            } else {
                Ok(())
            }
        });
        (attempts, menu_button("Actions", actions))
    });
    let trigger = screen.get_by_role(Role::Button, "Actions");
    trigger.click();
    item(&screen, "Save").a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), 1);
    assert_eq!(screen.errors().len(), 1);
    assert!(!gone(&screen, "Save"), "still open");
    item(&screen, "Save").a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), 2);
    assert_eq!(screen.errors().len(), 1);
    assert!(gone(&screen, "Save"));
    assert!(trigger.is_focused());
}

#[test]
fn closing_in_the_turn_of_a_hover_or_a_key_sends_no_focus_request_to_a_removed_row() {
    let (screen, chosen) = mount_button(vec![
        Entry::item("first", "First"),
        Entry::item("grid", "Show grid").checked(true),
        Entry::item("last", "Last"),
    ]);
    let trigger = screen.get_by_role(Role::Button, "Edit");

    // A click moves the pointer onto the row, presses and releases in one turn:
    // the row it chooses is the one that is already focused ..
    trigger.click();
    assert!(item(&screen, "First").is_focused());
    item(&screen, "First").click();
    assert_eq!(chosen.get_untracked(), ["first"]);
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    // .. or one that is not.
    trigger.click();
    screen
        .get_by_role(Role::MenuItemCheckBox, "Show grid")
        .click();
    assert_eq!(chosen.get_untracked(), ["first", "grid"]);
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    assert!(trigger.is_focused());

    // Move to the checked row with the keys and leave with Escape.
    trigger.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(
        screen
            .get_by_role(Role::MenuItemCheckBox, "Show grid")
            .is_focused()
    );
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tab_closes_the_menu_and_leaves_the_focus_where_tab_put_it() {
    let (screen, _) = mount_button(edit_entries());
    let trigger = screen.get_by_role(Role::Button, "Edit");
    trigger.press_key(Key::ArrowDown);
    screen.press_tab(false);
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    assert!(screen.get_by_role(Role::Button, "After").is_focused());

    trigger.press_key(Key::ArrowDown);
    assert!(item(&screen, "Copy").is_focused());
    screen.press_tab(true);
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn rows_are_matched_by_key_when_the_entries_change_while_the_menu_is_open() {
    let base = || {
        vec![
            Entry::item("a", "Alpha"),
            Entry::item("b", "Beta"),
            Entry::submenu(
                "s",
                "More",
                vec![Entry::item("x", "Xray"), Entry::item("y", "Yankee")],
            ),
            Entry::item("c", "Gamma"),
        ]
    };
    let (screen, (entries, chosen)) = mount_with(real_window(), move || {
        let entries = signal(base());
        let chosen = signal(Vec::new());
        let edit = use_menu(entries, move |key: &'static str| {
            chosen.update(|chosen| chosen.push(key))
        });
        ((entries, chosen), menu_button("Edit", edit))
    });
    let trigger = screen.get_by_role(Role::Button, "Edit");

    trigger.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(item(&screen, "Beta").is_focused());
    let mut inserted = vec![Entry::item("n", "New")];
    inserted.extend(base());
    entries.set(inserted.clone());
    screen.run_until_idle();
    assert!(
        item(&screen, "Beta").is_focused(),
        "the focus follows the key, not the position"
    );
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), ["b"]);

    trigger.press_key(Key::ArrowDown);
    assert!(item(&screen, "New").is_focused(), "New is first now");
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(item(&screen, "Beta").is_focused());
    entries.set(
        base()
            .into_iter()
            .filter(|e| e != &Entry::item("b", "Beta"))
            .collect(),
    );
    screen.run_until_idle();
    assert!(
        screen.query_by_role(Role::Menu, "Edit").is_none(),
        "the level whose focused key vanished closes"
    );
    assert!(trigger.is_focused());
    assert_eq!(chosen.get_untracked(), ["b"], "no stale key is chosen");

    entries.set(base());
    screen.run_until_idle();
    trigger.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::ArrowDown);
    assert!(item(&screen, "Yankee").is_focused());
    entries.set(vec![
        Entry::item("a", "Alpha"),
        Entry::item("b", "Beta"),
        Entry::submenu("s", "More", vec![Entry::item("x", "Xray")]),
        Entry::item("c", "Gamma"),
    ]);
    screen.run_until_idle();
    assert!(gone(&screen, "Xray"), "the submenu closes");
    assert!(
        item(&screen, "More").is_focused(),
        "focus returns to its item"
    );
    assert!(screen.get_by_role(Role::Menu, "Edit").exists());
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), ["b", "c"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn roles_checked_expanded_description_and_the_separator_are_published() {
    let (screen, _) = mount_button(vec![
        Entry::item("cut", "Cut").shortcut("Ctrl X"),
        Entry::item("grid", "Show grid").checked(true),
        Entry::item("snap", "Snap").checked(false),
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
    assert_eq!(cut.is_checked(), None);
    assert_eq!(cut.is_expanded(), None);
    assert_eq!(
        screen
            .get_by_role(Role::MenuItemCheckBox, "Show grid")
            .description(),
        ""
    );

    let grid = screen.get_by_role(Role::MenuItemCheckBox, "Show grid");
    assert_eq!(grid.is_checked(), Some(true));
    assert_eq!(
        screen
            .get_by_role(Role::MenuItemCheckBox, "Snap")
            .is_checked(),
        Some(false)
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

#[test]
fn a_checked_item_only_calls_back_and_the_application_owns_the_value() {
    let (screen, (grid, chosen)) = mount_with(real_window(), || {
        let grid = signal(false);
        let chosen = signal(0usize);
        let view = use_menu(
            move || vec![Entry::item("grid", "Show grid").checked(grid.get())],
            move |_: &'static str| {
                chosen.update(|count| *count += 1);
            },
        );
        ((grid, chosen), menu_button("View", view))
    });
    screen.get_by_role(Role::Button, "View").click();
    let check = || screen.get_by_role(Role::MenuItemCheckBox, "Show grid");
    assert_eq!(check().is_checked(), Some(false));
    check().a11y(A11yAction::Activate);
    assert_eq!(chosen.get_untracked(), 1);
    assert!(!grid.get_untracked(), "the menu keeps no checked state");
    assert!(
        screen.query_by_role(Role::Menu, "View").is_none(),
        "choosing closes the menu"
    );
    grid.set(true);
    screen.get_by_role(Role::Button, "View").click();
    assert_eq!(check().is_checked(), Some(true));
}

#[test]
fn a_mapped_submenu_chooses_the_mapped_key() {
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    enum Command {
        App(&'static str),
        Builtin,
    }
    let (screen, chosen) = mount_with(real_window(), || {
        let chosen = signal(Vec::new());
        let mut entries = nested_entries()
            .into_iter()
            .map(|entry| entry.map_key(&Command::App))
            .collect::<Vec<_>>();
        entries.push(MenuEntry::separator());
        entries.push(MenuEntry::item(Command::Builtin, "Close"));
        let edit = use_menu(entries, move |key: Command| {
            chosen.update(|chosen| chosen.push(key))
        });
        (chosen, menu_button("Edit", edit))
    });
    screen.get_by_role(Role::Button, "Edit").click();
    item(&screen, "Export").hover();
    screen.advance(Duration::from_millis(140));
    assert!(
        item(&screen, "SVG").is_disabled(),
        "the mapped entry keeps its disabled state"
    );
    item(&screen, "PDF").click();
    assert_eq!(chosen.get_untracked(), [Command::App("pdf")]);
    screen.get_by_role(Role::Button, "Edit").click();
    item(&screen, "Close").click();
    assert_eq!(
        chosen.get_untracked(),
        [Command::App("pdf"), Command::Builtin]
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
        let edit = use_menu(edit_entries(), |_: &'static str| ());
        let area = node()
            .role(Role::Group)
            .name("Canvas")
            .focusable(true)
            .style(style().size(260, 160).margin(60))
            .child(action("Inner"));
        (
            ContextRig {
                open: edit.shown(),
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
fn a_context_menu_choice_returns_the_focus_to_the_element_that_had_it() {
    let (screen, chosen) = mount_with(real_window(), || {
        let chosen = signal(Vec::new());
        let edit = use_menu(edit_entries(), move |key: &'static str| {
            chosen.update(|chosen| chosen.push(key))
        });
        let area = node()
            .role(Role::Group)
            .name("Canvas")
            .focusable(true)
            .style(style().size(260, 160).margin(60))
            .child(action("Inner"));
        (chosen, context_area("Canvas", area, edit))
    });
    let inner = screen.get_by_role(Role::Button, "Inner");
    inner.focus();
    screen.press_key_with(Key::F(10), shift());
    assert!(
        screen.get_by_role(Role::Menu, "Canvas").exists(),
        "the menu is named after the area"
    );
    assert!(
        item(&screen, "Copy").is_focused(),
        "on its first enabled item"
    );
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), ["paste"]);
    assert!(screen.query_by_role(Role::Menu, "Canvas").is_none());
    assert!(inner.is_focused(), "back to the element that had the focus");

    screen.press_key(Key::ContextMenu);
    assert!(item(&screen, "Copy").is_focused());
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Menu, "Canvas").is_none());
    assert!(inner.is_focused());

    let area = screen.get_by_role(Role::Group, "Canvas");
    let middle = center(&area);
    area.click_with(MouseButton::Secondary);
    let first = item(&screen, "Copy");
    assert!(first.is_focused());
    let at = first.rect().expect("invariant: the item is laid out");
    assert!(
        at.x >= middle.x - 1.0 && at.x <= middle.x + 12.0,
        "opens at the pointer: {at:?} {middle:?}"
    );
    screen.press_key(Key::Escape);
    assert!(area.is_focused(), "the right click had focused the area");

    area.a11y(A11yAction::ShowMenu);
    assert!(item(&screen, "Copy").is_focused());
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), ["paste", "all"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_caller_opens_and_closes_a_menu_on_its_first_item_without_the_trigger() {
    let (screen, (edit, chosen)) = mount_with(real_window(), || {
        let chosen = signal(Vec::new());
        let edit = use_menu(edit_entries(), move |key: &'static str| {
            chosen.update(|chosen| chosen.push(key))
        });
        (
            (edit, chosen),
            column((menu_button("Edit", edit), action("After"))),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Edit");
    assert_eq!(trigger.is_expanded(), Some(false));

    edit.open();
    screen.run_until_idle();
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(item(&screen, "Copy").is_focused(), "the first enabled item");
    // Opening an open menu changes nothing.
    edit.open();
    screen.run_until_idle();
    assert!(item(&screen, "Copy").is_focused());

    edit.close();
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    assert_eq!(trigger.is_expanded(), Some(false));
    // Closing a closed menu does nothing.
    edit.close();
    screen.run_until_idle();

    // The focus still goes back to the trigger when the menu is left by Escape.
    edit.open();
    screen.run_until_idle();
    screen.press_key(Key::Escape);
    assert!(trigger.is_focused());
    assert!(chosen.get_untracked().is_empty());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_kept_open_check_item_toggles_without_closing_the_menu() {
    let (screen, chosen) = mount_button(vec![
        MenuEntry::item("bold", "Bold")
            .checked(false)
            .keep_open(true),
        MenuEntry::item("cut", "Cut"),
    ]);
    screen.get_by_role(Role::Button, "Edit").click();
    screen.get_by_role(Role::MenuItemCheckBox, "Bold").click();
    assert_eq!(chosen.get_untracked(), ["bold"]);
    let bold = screen.get_by_role(Role::MenuItemCheckBox, "Bold");
    assert!(bold.is_focused(), "the menu stays open on the item");
    screen.press_key(Key::Space);
    assert_eq!(chosen.get_untracked(), ["bold", "bold"]);
    item(&screen, "Cut").click();
    assert!(gone(&screen, "Cut"), "another item still closes it");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_destructive_entry_reaches_the_item_state() {
    fn row(
        _: &kivact::style::Theme,
        state: &Look<kivact::interaction::MenuItemState>,
    ) -> kivact::style::Style {
        style().size(if state.destructive { 200 } else { 120 }, 24)
    }
    let recipes = UiRecipes::default().menu(|table| table.item(|_| row));
    let (screen, ()) = mount_with(
        RenderOptions {
            theme: kivact::style::Theme::light().with(recipes),
            ..real_window()
        },
        || {
            let entries = vec![
                MenuEntry::item(1, "Rename"),
                MenuEntry::item(2, "Delete").destructive(true),
            ];
            let view = kivact::ui::menu("Actions", entries, |_: i32| {}).attach(action("Actions"));
            ((), view)
        },
    );
    screen.get_by_role(Role::Button, "Actions").click();
    let width = |name| item(&screen, name).rect().expect("laid out").width;
    assert_eq!(width("Rename"), 120.0);
    assert_eq!(width("Delete"), 200.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn escape_and_a_choice_return_the_focus_to_the_menu_button() {
    let (screen, chosen) = mount_button(edit_entries());
    let anchor = screen.get_by_role(Role::Button, "Edit");
    anchor.focus();
    anchor.press_key(Key::ArrowDown);
    screen.press_key(Key::Escape);
    assert!(gone(&screen, "Copy"));
    assert!(anchor.is_focused(), "Escape returns to the menu button");
    anchor.press_key(Key::ArrowDown);
    item(&screen, "Copy").a11y(A11yAction::Activate);
    assert_eq!(chosen.get_untracked(), ["copy"]);
    assert!(gone(&screen, "Copy"));
    assert!(anchor.is_focused(), "a choice returns to the menu button");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
