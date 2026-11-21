//! Role: The page of the Creative catalog.

use super::common::specs;
use super::feedback::*;
use catalog_shell::{Example, Page};
use kivact::geometry::Point;
use kivact::interaction::MenuEntry;
use kivact::prelude::{IntoView, signal};
use kivact::view::node_ref;
use kivact_ui_creative as ui;

pub(crate) fn page() -> Page {
    Page::new("menus", "Menus", "Feedback", include_str!("menus.rs"))
        .description("Menus, search and context menus.")
        .example(Example::new("menus", "Menus", "", demo))
}

// example: menus
fn demo() -> impl IntoView {
    let status = signal(String::new());
    let pie = signal(ui::PieState::default());
    let pointer = signal(Point::new(0.0, 0.0));
    let pie_zone = node_ref();
    let search = signal(false);
    let entries = vec![
        MenuEntry::item("move", "Move").with_shortcut("G"),
        MenuEntry::item("rotate", "Rotate").with_shortcut("R"),
        MenuEntry::item("scale", "Scale").with_shortcut("S"),
        MenuEntry::separator(),
        MenuEntry::item("duplicate", "Duplicate").with_shortcut("Shift D"),
        MenuEntry::item("delete", "Delete").with_shortcut("X"),
        MenuEntry::item("apply", "Apply Transform").disabled(true),
    ];
    specs((
        menus_context_menu(status, entries),
        menus_pie_menu(pie, pointer, pie_zone),
        menus_dropdown_search(status, search),
    ))
}
// end example
