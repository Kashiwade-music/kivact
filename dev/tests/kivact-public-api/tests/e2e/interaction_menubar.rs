//! Role: Tests the menu bar behavior on real windows.
//!
//! A row of menu triggers: the roles, the arrows that move the focus along
//! the bar and, with a menu open, move the open menu, wrapping at the ends,
//! mirrored in a right-to-left subtree, and the pointer that moves an open
//! menu to the trigger it rests on.

use crate::support::mount;
use kivact::interaction::{MenuApi, MenuEntry, MenuRow, Menubar, use_menu, use_menubar};
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::style::Align;
use kivact::view::{Placement, ReadingDirection, overlay, provide_direction};

type Entry = MenuEntry<&'static str>;

/// The rows of one level.
fn level(menu: MenuApi<&'static str>) -> Node {
    let rows = for_each(
        menu.entries(),
        |(row, _)| row.clone(),
        move |row, entry| match row {
            MenuRow::Item { key, .. } => {
                let label = memo(move || entry.with(|(_, entry)| entry.label()));
                node()
                    .part(menu.item(key))
                    .style(style().height(28).padding_x(8).align_items(Align::Center))
                    .child(text(label))
                    .into_view()
            }
            _ => ().into_view(),
        },
    );
    column(rows).style(style().min_width(120).padding(4))
}

/// The trigger of the menu `at`, named `name`, and the menu's surface.
fn trigger(
    bar: Menubar<&'static str>,
    at: usize,
    name: &'static str,
    menu: MenuApi<&'static str>,
) -> View {
    let trigger = menu.trigger();
    let anchor = trigger.node_ref();
    (
        node()
            .part(trigger)
            .part(bar.item(at))
            .name(name)
            .style(style().size(80, 28).align_items(Align::Center))
            .child(text(name)),
        menu.show(move || {
            overlay(anchor, Placement::Below, level(menu))
                .part(menu.content())
                .name(name)
        }),
    )
        .into_view()
}

fn bar(chosen: Signal<Vec<&'static str>>) -> impl IntoView {
    let pick = move |key: &'static str| chosen.update(|all| all.push(key));
    let file = use_menu(
        vec![Entry::item("new", "New"), Entry::item("open", "Open")],
        pick,
    );
    let edit = use_menu(vec![Entry::item("undo", "Undo")], pick);
    let view = use_menu(vec![Entry::item("zoom", "Zoom")], pick);
    let titles = vec!["File".to_string(), "Edit".to_string(), "View".to_string()];
    let menubar = use_menubar(titles, vec![file, edit, view]);
    row((
        trigger(menubar, 0, "File", file),
        trigger(menubar, 1, "Edit", edit),
        trigger(menubar, 2, "View", view),
    ))
    .part(menubar.bar())
}

#[test]
fn a_bar_is_a_menu_bar_of_menu_items_and_the_arrows_move_the_focus() {
    let (screen, ()) = mount(|| ((), column((button("Before"), bar(signal(Vec::new()))))));
    screen.get_by_role(Role::MenuBar, "");
    for name in ["File", "Edit", "View"] {
        screen.get_by_role(Role::MenuItem, name);
    }
    let file = screen.get_by_role(Role::MenuItem, "File");
    file.focus();
    file.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::MenuItem, "Edit").is_focused());
    screen.press_key(Key::End);
    assert!(screen.get_by_role(Role::MenuItem, "View").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(
        screen.get_by_role(Role::MenuItem, "File").is_focused(),
        "the focus wraps"
    );
    screen.press_key(Key::ArrowLeft);
    assert!(screen.get_by_role(Role::MenuItem, "View").is_focused());
    screen.press_key(Key::Home);
    assert!(screen.get_by_role(Role::MenuItem, "File").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn with_a_menu_open_the_arrows_move_the_open_menu_and_it_wraps() {
    let (screen, chosen) = mount(|| {
        let chosen = signal(Vec::new());
        (chosen, bar(chosen))
    });
    let file = screen.get_by_role(Role::MenuItem, "File");
    file.focus();
    file.click();
    screen.get_by_role(Role::Menu, "File");
    screen.press_key(Key::ArrowRight);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Menu, "File").is_none());
    screen.get_by_role(Role::Menu, "Edit");
    assert!(
        screen.get_by_role(Role::MenuItem, "Edit").is_expanded() == Some(true),
        "the neighbour's trigger is the open one"
    );
    screen.press_key(Key::ArrowRight);
    screen.run_until_idle();
    screen.get_by_role(Role::Menu, "View");
    screen.press_key(Key::ArrowRight);
    screen.run_until_idle();
    screen.get_by_role(Role::Menu, "File");
    screen.press_key(Key::ArrowLeft);
    screen.run_until_idle();
    screen.get_by_role(Role::Menu, "View");
    assert!(chosen.get_untracked().is_empty());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn resting_the_pointer_on_another_trigger_moves_an_open_menu() {
    let (screen, ()) = mount(|| ((), bar(signal(Vec::new()))));
    screen.get_by_role(Role::MenuItem, "File").hover();
    screen.run_until_idle();
    assert!(
        screen.query_by_role(Role::Menu, "File").is_none(),
        "with no menu open, resting opens nothing"
    );
    screen.get_by_role(Role::MenuItem, "File").click();
    screen.get_by_role(Role::Menu, "File");
    screen.get_by_role(Role::MenuItem, "Edit").hover();
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Menu, "File").is_none());
    screen.get_by_role(Role::Menu, "Edit");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn in_a_right_to_left_subtree_the_arrows_are_mirrored() {
    let (screen, ()) = mount(|| {
        (
            (),
            provide_direction(ReadingDirection::Rtl, || bar(signal(Vec::new()))),
        )
    });
    let file = screen.get_by_role(Role::MenuItem, "File");
    file.focus();
    file.press_key(Key::ArrowLeft);
    assert!(
        screen.get_by_role(Role::MenuItem, "Edit").is_focused(),
        "Left goes to the next trigger"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
