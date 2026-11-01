//! Role: Tests for list navigation and roving tab stops.
//!
//! The unstyled `List`: a roving tab stop over the rows of a
//! multi-select list, the keys of an ARIA list box with multiple selection,
//! typeahead, the press, and the active row following the rows.

use kivact::primitive::NodeExt;

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

struct Handles {
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
            Handles { selected, nav },
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
