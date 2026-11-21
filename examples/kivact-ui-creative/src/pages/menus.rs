//! Role: The page of the Creative catalog.

use super::common::*;
use super::common::{col, keyed_options, line, note, spec, specs};
use super::controls::num;
use super::controls::slide;
use super::feedback::*;
use catalog_shell::{Example, Page};
use kivact::geometry::Point;
use kivact::interaction::MenuEntry;
use kivact::prelude::{
    Element, IntoView, Key, NodeRef, Role, Signal, Theme, column, for_each, memo, node, on_mount,
    row, scroll, show, signal, text,
};
use kivact::state::callback;
use kivact::style::GridPlacement;
use kivact::style::use_theme;
use kivact::style::{Align, Justify, Style, pct, rgba, style};
use kivact::view::Node;
use kivact::view::{Handled, Placement, node_ref, overlay};
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
