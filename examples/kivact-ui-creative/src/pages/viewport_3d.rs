//! Role: The page of the Creative catalog.

use catalog_shell::{Example, Page};
use super::audio_values::TransportState;
use kivact::state::Signal;
use super::common::{bind, col, keyed_options, line, note, spec, specs};
use super::controls::num;
use super::controls::tool_palette;
use super::editor_images as images;
use kivact::draw::Path;
use kivact::geometry::Rect;
use kivact::geometry::{Point, Size};
use kivact::interaction::MenuEntry;
use kivact::prelude::{
    Element, IntoView, NodeRef, Role, dynamic, memo, node, rgb, row, signal, spacer, text,
};
use kivact::style::Position;
use kivact::style::{Align, Color, Justify, pct, style};
use kivact::view::Node;
use kivact_ui_creative as ui;
use super::editors::*;
use super::common::*;

pub(crate) fn page() -> Page {
    Page::new("3d-viewport", "3D Viewport", "Compositing", include_str!("viewport_3d.rs"))
        .description("View, grid, selection, pan and zoom.")
        .example(Example::new("3d-viewport", "3D Viewport", "", demo))
}

// example: 3d-viewport
fn demo() -> impl IntoView {
    let shading = signal(1_usize);
    let grid = signal(true);
    let reference = node_ref();
    let pointer = signal(Point::new(0.0, 0.0));
    let transform = signal(kivact::interaction::ViewTransform::default());
    let zoom = kivact::interaction::use_zoom_pan(transform);
    let frame = memo(move || images::gradient(640, 360, shading.get(), grid.get()));
    col((
        viewport_toolbar(shading, grid, reference, transform)
            .style(style().height(38).padding_x(10)),
        row((
            tool_palette(true).style(style().width(42).padding(6)),
            kivact::view::viewport(frame)
                .transform(zoom)
                .with_node(move |node| {
                    node.on_pointer(move |event| {
                        pointer.set(zoom.to_content(event.pos()));
                        kivact::view::Handled::No
                    })
                    .region(kivact::draw::RegionId(1), {
                        let input = kivact::custom::part(|input| input);
                        kivact::custom::Part::new(input.state(), move |node| {
                            node.part(input).role(Role::Image).name("Sample area")
                        })
                    })
                    .draw_over(move |draw, _| {
                        let top = zoom.to_view(Point::new(220.0, 110.0));
                        let bottom = zoom.to_view(Point::new(420.0, 250.0));
                        let bounds = Rect::new(top.x, top.y, bottom.x - top.x, bottom.y - top.y);
                        draw.stroke_rect(bounds, 1.0, rgb(0xffd25a));
                        draw.region(kivact::draw::RegionId(1), bounds);
                    })
                })
                .style(style().height(392).flex_grow(1))
                .with_node(|node| node.node_ref(reference))
                .test_id("creative-viewport"),
        ))
        .style(style().gap(4)),
        row((
            note("CPU gradient · image coordinates"),
            spacer(),
            note("Middle drag: pan · Wheel: zoom · 0: fit"),
        ))
        .style(style().padding(12)),
    ))
}
// end example
