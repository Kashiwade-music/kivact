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
    Page::new(
        "dialogs-popovers",
        "Dialogs & Popovers",
        "Feedback",
        include_str!("dialogs_popovers.rs"),
    )
    .description("Windows, popovers, tooltips and banners.")
    .example(Example::new(
        "dialogs-popovers",
        "Dialogs & Popovers",
        "",
        demo,
    ))
}

// example: dialogs-popovers
fn demo() -> impl IntoView {
    let status = signal(String::new());
    specs((
        dialogs_metrics_and_descriptions(),
        dialogs_dialog(status),
        dialogs_popover(),
        dialogs_tooltip(),
        dialogs_banner().style(style().grid_column(GridPlacement::span(2)).min_height(250)),
    ))
}
// end example
