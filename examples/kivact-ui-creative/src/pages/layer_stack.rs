//! Role: The page of the Creative catalog.

use catalog_shell::{Example, Page};
use super::audio_values::TransportState;
use kivact::state::Signal;
use super::common::{bind, col, keyed_options, line, note, spec, specs};
use super::controls::chip;
use super::controls::num;
use super::controls::slide;
use kivact::geometry::Point;
use kivact::prelude::{
    Element, IntoView, Key, column, dynamic, grid, node, rgb, row, scroll, signal, spacer, text,
};
use kivact::style::{Align, Color, pct, style};
use kivact::style::{GridPlacement, Track};
use kivact::view::Handled;
use kivact::view::{Node, Orientation};
use kivact_ui_creative as ui;
use super::layout::*;
use super::common::*;

pub(crate) fn page() -> Page {
    Page::new("layer-stack", "Layer Stack", "Animation", include_str!("layer_stack.rs"))
        .description("A/V switches, layers, blend modes and parenting.")
        .example(Example::new("layer-stack", "Layer Stack", "", demo))
}

// example: layer-stack
fn demo() -> impl IntoView {
    let state = transport();
    let query = signal(String::new());
    let flags = signal(
        (0..8)
            .map(|i| {
                let mut f = [false; 12];
                f[0] = i != 7;
                f[1] = i == 7;
                f[3] = i == 6;
                for x in [6, 7] {
                    f[x] = i < 6;
                }
                f
            })
            .collect::<Vec<_>>(),
    );
    let opened = signal(vec![4_u64]);
    let ranges = signal(vec![
        (0.0, 240.0),
        (24.0, 168.0),
        (0.0, 200.0),
        (0.0, 240.0),
        (40.0, 220.0),
        (0.0, 240.0),
        (0.0, 240.0),
        (0.0, 240.0),
    ]);
    let rows = layer_rows();
    col((
        line((
            text("Comp 1").style(style().font_weight(600)),
            kivact::ui::text_input("Layer search", query)
                .placeholder("Search layers")
                .style(style().width(180)),
            kivact::ui::toggle_button("Shy", signal(false)),
            kivact::ui::toggle_button("Collapse", signal(false)),
            kivact::ui::toggle_button("Motion Blur", signal(false)),
        )),
        super::common::text_table(
            &[
                ("A/V", 104.0),
                ("Layer Name", 0.0),
                ("Switches", 160.0),
                ("Mode", 110.0),
                ("Parent & Link", 120.0),
                ("Range", 260.0),
            ],
            rows,
            signal(vec![4]),
            signal(None),
        )
        .filter(query)
        .row_label(|row| row.cells.get(1).cloned().unwrap_or_default())
        .resizable(signal(Vec::new()))
        .cell_controls(move |item, column| layer_cell(item, column, flags, opened, ranges, rows))
        .row_details(move |item| {
            let id = item.get_untracked().id;
            col((
                text("⌄  Transform").style(style().height(20).font_weight(600)),
                layer_property(id, "Anchor Point", vec![960.0, 540.0], "", state, false),
                layer_property(id, "Position", vec![960.0, 540.0], "", state, id == 2),
                layer_property(id, "Scale", vec![100.0, 100.0], "%", state, false),
                layer_property(id, "Rotation", vec![0.0], "°", state, false),
                layer_property(id, "Opacity", vec![100.0], "%", state, id == 2),
            ))
            .hidden(move || !opened.with(|v| v.contains(&id)))
            .style(style().padding_left(168).gap(4).padding_bottom(8))
            .into_view()
        })
        .on_reorder(move |change| super::layout_reducers::reorder(rows, change.key, change.to))
        .name("Layer Stack table"),
    ))
    .style(style().padding(8))
}
// end example
