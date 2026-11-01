//! Role: Tests for list navigation and roving tab stops.
//!
//! The unstyled `List`: a roving tab stop over the rows of a
//! multi-select list, the keys of an ARIA list box with multiple selection,
//! typeahead, the press, and the active row following the rows.

use kivact::primitive::NodeExt;
use std::time::Duration;

use crate::support::mount_with;
use kivact::interaction::{List, ListPolicy, ListSelection, use_list};
use kivact::prelude::*;
use kivact::test::{Element, RenderOptions, Screen, WindowMode};
use kivact::ui::ListItem;
use kivact::view::{KeyCode, Modifiers};

const NAMES: [&str; 12] = [
    "Apple", "Apricot", "Banana", "Cherry", "Date", "Elder", "Fig", "Grape", "Honeydew", "Kiwi",
    "Lemon", "Mango",
];

const SHIFT: Modifiers = Modifiers {
    shift: true,
    ctrl: false,
    alt: false,
    meta: false,
};
const CTRL: Modifiers = Modifiers {
    shift: false,
    ctrl: true,
    alt: false,
    meta: false,
};
const META: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: true,
};
const CTRL_SHIFT: Modifiers = Modifiers {
    shift: true,
    ctrl: true,
    alt: false,
    meta: false,
};

struct Handles {
    rows: Signal<Vec<&'static str>>,
    selected: Signal<Vec<&'static str>>,
    nav: List<&'static str>,
}

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        ..RenderOptions::from_env()
    }
}

fn mount_list(
    initial: Vec<&'static str>,
    configure: impl FnOnce(List<&'static str>) -> List<&'static str> + 'static,
) -> (Screen, Handles) {
    mount_list_kind(initial, false, configure)
}

fn mount_list_kind(
    initial: Vec<&'static str>,
    single: bool,
    configure: impl FnOnce(List<&'static str>) -> List<&'static str> + 'static,
) -> (Screen, Handles) {
    mount_with(real_window(), move || {
        let rows = signal(NAMES.to_vec());
        let selected = signal(initial);
        let selection = if single {
            ListSelection::Single(kivact::binding(
                memo(move || selected.with(|keys| keys.first().copied())),
                move |key: Option<&'static str>| selected.set(key.into_iter().collect()),
            ))
        } else {
            ListSelection::Multiple(selected.into())
        };

        let nav = configure(
            use_list(
                move || {
                    rows.get()
                        .into_iter()
                        .map(|key| ListItem::new(key, key))
                        .collect::<Vec<_>>()
                },
                selection,
            )
            .policy(ListPolicy {
                typeahead: true,
                ..ListPolicy::default()
            }),
        );
        let list = for_each(
            rows,
            |name| *name,
            move |name, _| {
                (node()
                    .role(Role::Option)
                    .name(name)
                    .selected(move || nav.selected().with(|keys| keys.contains(&name)))
                    .child(text(name))
                    .style(style().height(24).shrink(0)))
                .part(nav.item(name).role(Role::Option))
            },
        );
        let list = (scroll(list)
            .role(Role::ListBox)
            .name("Fruit")
            .test_id("list")
            .style(style().height(120)))
        .part(nav.list());
        (
            Handles {
                rows,
                selected,
                nav,
            },
            column((button("before"), list, button("after"))),
        )
    })
}

fn button(name: &'static str) -> Node {
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .child(text(name))
        .style(style().size(120, 28))
}

fn row<'a>(screen: &'a Screen, name: &str) -> Element<'a> {
    screen.get_by_role(Role::Option, name)
}

/// Click the row `name`: it is selected, active and focused.
fn focus_row(screen: &Screen, name: &str) {
    row(screen, name).click();
    assert!(row(screen, name).is_focused());
}

fn key_with(screen: &Screen, key: Key, modifiers: Modifiers) {
    screen.press_key_with(key, modifiers);
}

/// Hold a modifier key down (the following clicks carry it).
fn hold(screen: &Screen, code: KeyCode, modifiers: Modifiers) {
    screen.key_down(Key::Unidentified, code, modifiers);
}

fn release(screen: &Screen, code: KeyCode) {
    screen.key_up(Key::Unidentified, code, Modifiers::default());
}

fn no_errors(screen: &Screen) {
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_list_is_one_tab_stop_on_the_active_row() {
    let (screen, h) = mount_list(Vec::new(), |nav| nav);
    assert!(
        screen
            .get_by_role(Role::ListBox, "Fruit")
            .is_multiselectable()
    );
    // Nothing selected: the first row.
    assert_eq!(h.nav.active().get_untracked(), Some("Apple"));
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(row(&screen, "Apple").is_focused());
    screen.press_tab(false);
    assert!(
        screen.get_by_role(Role::Button, "after").is_focused(),
        "Tab leaves the list: the other rows are no tab stops"
    );
    screen.press_tab(true);
    assert!(
        row(&screen, "Apple").is_focused(),
        "Shift+Tab returns to the list"
    );
    screen.press_tab(true);
    assert!(screen.get_by_role(Role::Button, "before").is_focused());
    no_errors(&screen);
}

#[test]
fn the_selection_decides_the_first_tab_stop_and_the_focused_row_follows() {
    let (screen, h) = mount_list(vec!["Date", "Cherry"], |nav| nav);
    // The last selected row.
    assert_eq!(h.nav.active().get_untracked(), Some("Cherry"));
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(row(&screen, "Cherry").is_focused());
    screen.press_key(Key::ArrowDown);
    assert_eq!(h.nav.active().get_untracked(), Some("Date"));
    screen.get_by_role(Role::Button, "after").click();
    screen.press_tab(true);
    assert!(
        row(&screen, "Date").is_focused(),
        "the last focused row stays the tab stop"
    );
    no_errors(&screen);
}

#[test]
fn arrows_home_end_and_pages_move_and_select_and_stop_at_the_ends() {
    let (screen, h) = mount_list(Vec::new(), |nav| nav.page_size(5));
    focus_row(&screen, "Apple");
    assert_eq!(h.selected.get_untracked(), ["Apple"], "a press selects");
    screen.press_key(Key::ArrowUp);
    assert!(
        row(&screen, "Apple").is_focused(),
        "Up stops at the first row"
    );
    assert_eq!(h.selected.get_untracked(), ["Apple"]);
    screen.press_key(Key::ArrowDown);
    assert!(row(&screen, "Apricot").is_focused());
    assert_eq!(h.selected.get_untracked(), ["Apricot"]);
    screen.press_key(Key::ArrowLeft);
    assert!(
        row(&screen, "Apricot").is_focused(),
        "horizontal arrows idle"
    );
    screen.press_key(Key::PageDown);
    assert!(row(&screen, "Fig").is_focused(), "five rows down");
    assert_eq!(h.selected.get_untracked(), ["Fig"]);
    screen.press_key(Key::PageUp);
    assert!(row(&screen, "Apricot").is_focused(), "five rows up");
    screen.press_key(Key::PageUp);
    assert!(
        row(&screen, "Apple").is_focused(),
        "a page stops at the first row"
    );
    screen.press_key(Key::End);
    assert!(row(&screen, "Mango").is_focused());
    assert_eq!(h.selected.get_untracked(), ["Mango"]);
    screen.press_key(Key::ArrowDown);
    assert!(
        row(&screen, "Mango").is_focused(),
        "Down stops at the last row"
    );
    screen.press_key(Key::PageDown);
    assert!(row(&screen, "Mango").is_focused());
    screen.press_key(Key::Home);
    assert!(row(&screen, "Apple").is_focused());
    assert_eq!(h.selected.get_untracked(), ["Apple"]);
    assert!(
        row(&screen, "Apple").is_selected() && !row(&screen, "Apricot").is_selected(),
        "the rows publish the selection"
    );
    no_errors(&screen);
}

#[test]
fn the_default_page_is_ten_rows() {
    let (screen, h) = mount_list(Vec::new(), |nav| nav);
    focus_row(&screen, "Apple");
    screen.press_key(Key::PageDown);
    assert!(row(&screen, "Lemon").is_focused());
    assert_eq!(h.selected.get_untracked(), ["Lemon"]);
    no_errors(&screen);
}

#[test]
fn shift_extends_a_range_from_the_anchor() {
    let (screen, h) = mount_list(Vec::new(), |nav| nav.page_size(3));
    focus_row(&screen, "Apricot");
    key_with(&screen, Key::ArrowDown, SHIFT);
    key_with(&screen, Key::ArrowDown, SHIFT);
    assert_eq!(h.selected.get_untracked(), ["Apricot", "Banana", "Cherry"]);
    assert!(row(&screen, "Cherry").is_focused());
    key_with(&screen, Key::ArrowUp, SHIFT);
    assert_eq!(h.selected.get_untracked(), ["Apricot", "Banana"]);
    key_with(&screen, Key::End, SHIFT);
    assert_eq!(h.selected.get_untracked().len(), 11, "Apricot to Mango");
    key_with(&screen, Key::Home, SHIFT);
    assert_eq!(h.selected.get_untracked(), ["Apple", "Apricot"]);
    key_with(&screen, Key::PageDown, SHIFT);
    assert_eq!(
        h.selected.get_untracked(),
        ["Apricot", "Banana", "Cherry"],
        "the page from the focused row"
    );
    assert!(row(&screen, "Cherry").is_focused());
    no_errors(&screen);
}

#[test]
fn ctrl_or_meta_moves_the_focus_alone_and_space_toggles() {
    for command in [CTRL, META] {
        let (screen, h) = mount_list(Vec::new(), |nav| nav);
        focus_row(&screen, "Apple");
        key_with(&screen, Key::ArrowDown, command);
        key_with(&screen, Key::ArrowDown, command);
        assert!(row(&screen, "Banana").is_focused());
        assert_eq!(h.selected.get_untracked(), ["Apple"], "the selection stays");
        key_with(&screen, Key::Space, command);
        assert_eq!(h.selected.get_untracked(), ["Apple", "Banana"]);
        key_with(&screen, Key::End, command);
        key_with(&screen, Key::Space, command);
        assert_eq!(h.selected.get_untracked(), ["Apple", "Banana", "Mango"]);
        key_with(&screen, Key::Home, command);
        key_with(&screen, Key::Space, command);
        assert_eq!(
            h.selected.get_untracked(),
            ["Banana", "Mango"],
            "toggled off"
        );
        key_with(&screen, Key::PageDown, command);
        assert!(row(&screen, "Lemon").is_focused());
        assert_eq!(h.selected.get_untracked(), ["Banana", "Mango"]);
        no_errors(&screen);
    }
}

#[test]
fn space_selects_the_focused_row_and_leaves_a_selected_one_alone() {
    let (screen, h) = mount_list(Vec::new(), |nav| nav);
    focus_row(&screen, "Apple");
    key_with(&screen, Key::ArrowDown, CTRL);
    key_with(&screen, Key::ArrowDown, CTRL);
    screen.press_key(Key::Space);
    assert_eq!(
        h.selected.get_untracked(),
        ["Banana"],
        "replaces the selection"
    );
    h.selected.set(vec!["Apple", "Banana", "Cherry"]);
    screen.run_until_idle();
    screen.press_key(Key::Space);
    assert_eq!(
        h.selected.get_untracked(),
        ["Apple", "Banana", "Cherry"],
        "a selected row is left as it is"
    );
    no_errors(&screen);
}

#[test]
fn shift_space_extends_from_the_anchor_to_the_focused_row() {
    let (screen, h) = mount_list(Vec::new(), |nav| nav);
    focus_row(&screen, "Apricot");
    for _ in 0..3 {
        key_with(&screen, Key::ArrowDown, CTRL);
    }
    assert!(row(&screen, "Date").is_focused());
    assert_eq!(h.selected.get_untracked(), ["Apricot"]);
    key_with(&screen, Key::Space, SHIFT);
    assert_eq!(
        h.selected.get_untracked(),
        ["Apricot", "Banana", "Cherry", "Date"]
    );
    key_with(&screen, Key::ArrowUp, CTRL_SHIFT);
    assert!(row(&screen, "Cherry").is_focused());
    no_errors(&screen);
}

#[test]
fn ctrl_a_and_meta_a_select_every_row() {
    for command in [CTRL, META] {
        let (screen, h) = mount_list(vec!["Date"], |nav| nav);
        focus_row(&screen, "Date");
        key_with(&screen, Key::Char('a'), command);
        assert_eq!(h.selected.get_untracked(), NAMES);
        assert!(row(&screen, "Date").is_focused(), "the focus stays");
        h.selected.set(Vec::new());
        screen.run_until_idle();
        key_with(&screen, Key::Char('A'), command);
        assert_eq!(h.selected.get_untracked(), NAMES);
        no_errors(&screen);
    }
    // Only the rows shown are selected.
    let (screen, h) = mount_list(Vec::new(), |nav| nav);
    h.rows.set(vec!["Apple", "Fig", "Kiwi"]);
    screen.run_until_idle();
    focus_row(&screen, "Fig");
    key_with(&screen, Key::Char('a'), CTRL);
    assert_eq!(h.selected.get_untracked(), ["Apple", "Fig", "Kiwi"]);
    no_errors(&screen);
}

#[test]
fn typing_focuses_the_next_row_that_starts_with_it_when_the_rows_have_labels() {
    let (screen, h) = mount_list(Vec::new(), |nav| nav);
    focus_row(&screen, "Apple");
    screen.press_key(Key::Char('b'));
    assert!(row(&screen, "Banana").is_focused());
    assert_eq!(h.selected.get_untracked(), ["Banana"], "typing selects");
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('a'));
    assert!(row(&screen, "Apple").is_focused(), "the search wraps");
    screen.press_key(Key::Char('p'));
    screen.press_key(Key::Char('r'));
    assert!(row(&screen, "Apricot").is_focused(), "the typed prefix");
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('z'));
    assert!(row(&screen, "Apricot").is_focused(), "no match");
    assert_eq!(h.selected.get_untracked(), ["Apricot"]);
    no_errors(&screen);

    // Without labels, typing does nothing.
    let (screen, h) = mount_list(Vec::new(), |nav| nav.policy(ListPolicy::default()));
    focus_row(&screen, "Apple");
    screen.press_key(Key::Char('b'));
    assert!(row(&screen, "Apple").is_focused());
    assert_eq!(h.selected.get_untracked(), ["Apple"]);
    no_errors(&screen);
}

#[test]
fn the_active_row_moves_when_its_row_is_removed_or_filtered_out() {
    let (screen, h) = mount_list(Vec::new(), |nav| nav);
    focus_row(&screen, "Cherry");
    h.rows.update(|rows| rows.retain(|name| *name != "Cherry"));
    screen.run_until_idle();
    assert_eq!(
        h.nav.active().get_untracked(),
        Some("Date"),
        "the row that took its place"
    );
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(row(&screen, "Date").is_focused());

    // The last row goes: the new last row.
    screen.press_key(Key::End);
    assert!(row(&screen, "Mango").is_focused());
    h.rows.update(|rows| rows.retain(|name| *name != "Mango"));
    screen.run_until_idle();
    assert_eq!(h.nav.active().get_untracked(), Some("Lemon"));

    // A filter that hides the active row.
    h.rows.set(vec!["Kiwi", "Lemon"]);
    screen.run_until_idle();
    assert_eq!(h.nav.active().get_untracked(), Some("Lemon"));
    h.rows.set(vec!["Fig"]);
    screen.run_until_idle();
    assert_eq!(h.nav.active().get_untracked(), Some("Fig"));
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(row(&screen, "Fig").is_focused());
    h.rows.set(Vec::new());
    screen.run_until_idle();
    assert_eq!(h.nav.active().get_untracked(), None);
    no_errors(&screen);

    // Rows come back and one is the tab stop again.
    h.rows.set(vec!["Fig", "Grape"]);
    screen.run_until_idle();
    assert!(h.nav.active().get_untracked().is_some());
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(
        row(&screen, "Fig").is_focused() || row(&screen, "Grape").is_focused(),
        "a row takes the focus"
    );
    no_errors(&screen);
}

#[test]
fn a_press_selects_by_modifiers_makes_the_row_active_and_focuses_it() {
    let (screen, h) = mount_list(Vec::new(), |nav| nav);
    row(&screen, "Banana").click();
    assert!(row(&screen, "Banana").is_focused());
    assert_eq!(h.selected.get_untracked(), ["Banana"]);
    assert_eq!(h.nav.active().get_untracked(), Some("Banana"));

    hold(&screen, KeyCode::ControlLeft, CTRL);
    row(&screen, "Date").click();
    assert_eq!(h.selected.get_untracked(), ["Banana", "Date"]);
    row(&screen, "Banana").click();
    assert_eq!(h.selected.get_untracked(), ["Date"]);
    release(&screen, KeyCode::ControlLeft);

    row(&screen, "Apricot").click();
    hold(&screen, KeyCode::ShiftLeft, SHIFT);
    row(&screen, "Date").click();
    assert_eq!(
        h.selected.get_untracked(),
        ["Apricot", "Banana", "Cherry", "Date"]
    );
    assert!(row(&screen, "Date").is_focused());
    release(&screen, KeyCode::ShiftLeft);

    // Keys continue from the pressed row; Tab out and back returns to it.
    screen.press_key(Key::ArrowDown);
    assert!(row(&screen, "Elder").is_focused());
    assert_eq!(h.selected.get_untracked(), ["Elder"]);
    screen.get_by_role(Role::Button, "after").click();
    screen.press_tab(true);
    assert!(row(&screen, "Elder").is_focused());
    no_errors(&screen);
}

#[test]
fn moving_scrolls_the_focused_row_into_view() {
    let (screen, _h) = mount_list(Vec::new(), |nav| nav);
    focus_row(&screen, "Apple");
    let list = screen.get_by_test_id("list").rect().expect("laid out");
    let inside = |name: &str| {
        let r = row(&screen, name).rect().expect("laid out");
        r.y >= list.y - 0.5 && r.y + r.height <= list.y + list.height + 0.5
    };
    assert!(!inside("Mango"), "the last row starts below the viewport");
    screen.press_key(Key::End);
    assert!(inside("Mango"), "End scrolls to the last row");
    assert!(!inside("Apple"));
    screen.press_key(Key::Home);
    assert!(inside("Apple"), "Home scrolls back");
    for _ in 0..6 {
        screen.press_key(Key::ArrowDown);
    }
    assert!(inside("Fig"), "stepping down follows the focus");
    no_errors(&screen);
}

#[test]
fn a_control_inside_a_row_keeps_its_own_keys_and_focus() {
    let (screen, selected) = mount_with(real_window(), || {
        let rows = signal(vec!["a", "b", "c"]);
        let selected = signal(Vec::<&'static str>::new());
        let nav = use_list(
            move || {
                rows.get()
                    .into_iter()
                    .map(|key| ListItem::new(key, key))
                    .collect::<Vec<_>>()
            },
            ListSelection::Multiple(selected.into()),
        )
        .policy(ListPolicy {
            typeahead: true,
            ..ListPolicy::default()
        });
        let list = for_each(
            rows,
            |name| *name,
            move |name, _| {
                (node()
                    .role(Role::Option)
                    .name(name)
                    .child(
                        node()
                            .role(Role::Button)
                            .name(format!("act {name}"))
                            .focusable(true)
                            .child(text("act")),
                    )
                    .style(style().height(30)))
                .part(nav.item(name).role(Role::Option))
            },
        );
        (
            selected,
            (column(list).role(Role::ListBox)).part(nav.list()),
        )
    });
    screen.get_by_role(Role::Button, "act b").click();
    assert!(screen.get_by_role(Role::Button, "act b").is_focused());
    assert_eq!(
        selected.get_untracked(),
        ["b"],
        "the press still selects the row"
    );
    screen.press_key(Key::ArrowDown);
    assert!(
        screen.get_by_role(Role::Button, "act b").is_focused(),
        "the control keeps the key"
    );
    assert_eq!(selected.get_untracked(), ["b"]);
    no_errors(&screen);
}

#[test]
fn a_single_select_list_replaces_for_every_modifier_and_is_not_multiselectable() {
    let (screen, h) = mount_list_kind(Vec::new(), true, |nav| nav);
    assert!(
        !screen
            .get_by_role(Role::ListBox, "Fruit")
            .is_multiselectable()
    );
    focus_row(&screen, "Apricot");
    for modifiers in [SHIFT, CTRL, META, CTRL_SHIFT] {
        key_with(&screen, Key::ArrowDown, modifiers);
        let focused = h.nav.active().get_untracked().expect("an active row");
        assert!(row(&screen, focused).is_focused());
        assert_eq!(h.selected.get_untracked(), [focused], "{modifiers:?}");
    }
    key_with(&screen, Key::End, SHIFT);
    assert_eq!(h.selected.get_untracked(), ["Mango"]);
    key_with(&screen, Key::ArrowUp, CTRL);
    key_with(&screen, Key::Space, CTRL);
    assert_eq!(
        h.selected.get_untracked(),
        ["Lemon"],
        "Ctrl+Space selects, it does not toggle"
    );
    key_with(&screen, Key::Space, CTRL);
    assert_eq!(h.selected.get_untracked(), ["Lemon"]);
    key_with(&screen, Key::Char('a'), CTRL);
    assert_eq!(h.selected.get_untracked(), ["Lemon"], "no select all");
    hold(&screen, KeyCode::ShiftLeft, SHIFT);
    row(&screen, "Kiwi").click();
    assert_eq!(h.selected.get_untracked(), ["Kiwi"], "no range");
    release(&screen, KeyCode::ShiftLeft);
    hold(&screen, KeyCode::ControlLeft, CTRL);
    row(&screen, "Honeydew").click();
    assert_eq!(h.selected.get_untracked(), ["Honeydew"], "no add");
    release(&screen, KeyCode::ControlLeft);
    no_errors(&screen);
}
