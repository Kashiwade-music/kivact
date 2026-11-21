//! Role: The page of the Creative catalog.

use super::common::*;
use super::editors::*;
use catalog_shell::{Example, Page};
use kivact::geometry::Point;
use kivact::prelude::{IntoView, signal};
use kivact::style::style;
use kivact_ui_creative as ui;

pub(crate) fn page() -> Page {
    Page::new(
        "dope-sheet-graph",
        "Dope Sheet & Graph",
        "Animation",
        include_str!("dope_sheet_graph.rs"),
    )
    .description("Key selection, moving, box select and F-curves.")
    .example(Example::new(
        "dope-sheet-graph",
        "Dope Sheet & Graph",
        "",
        demo,
    ))
}

// example: dope-sheet-graph
fn demo() -> impl IntoView {
    let state = transport();
    let graph = signal(animation_curves());
    let graph_selected = signal(vec![18_u64]);
    let (keys, selected) = super::animation_model::bindings(graph, graph_selected);
    let initial_keys = super::txn::Txn::signal();
    let channels = [signal(true), signal(true), signal(true)];
    let transform = signal(kivact::interaction::ViewTransform {
        scale: 6.8,
        offset: Point::new(12.0, 0.0),
    });
    let graph_view = signal(ui::GraphView {
        time: (-20.0, 200.0),
        value: (-2.0, 3.0),
    });
    col((
        animation_dope_toolbar(state, keys),
        animation_dope_sheet(state, keys, selected, initial_keys, transform),
        animation_graph_toolbar(graph, channels, graph_view),
        animation_graph_editor(state, graph, graph_selected, channels, graph_view),
    ))
    .style(style().gap(0))
}
// end example
