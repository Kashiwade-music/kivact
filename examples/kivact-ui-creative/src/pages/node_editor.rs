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
    Page::new("node-editor", "Node Editor", "Compositing", include_str!("node_editor.rs"))
        .description("Nodes, typed sockets, link connection, pan and zoom.")
        .example(Example::new("node-editor", "Node Editor", "", demo))
}

// example: node-editor
fn demo() -> impl IntoView {
    let data = signal(vec![
        super::node_model::editor_node(1, "Render Layers", 100.0, 28.0, 0xff5c6c),
        super::node_model::editor_node(2, "Blur", 310.0, 20.0, 0x8b7cff),
        super::node_model::editor_node(3, "Image", 100.0, 230.0, 0xff5c6c),
        super::node_model::editor_node(4, "Color Balance", 310.0, 200.0, 0xffd25a),
        super::node_model::editor_node(5, "Keying", 310.0, 390.0, 0x34d399),
        super::node_model::editor_node(6, "Mix", 535.0, 150.0, 0xffd25a),
        super::node_model::editor_node(7, "Viewer", 755.0, 20.0, 0xff5c6c),
        super::node_model::editor_node(8, "Composite", 755.0, 260.0, 0xff5c6c),
    ]);
    let connections = signal(vec![
        ui::NodeLink::new(1, 0, 2, 0),
        ui::NodeLink::new(1, 2, 2, 1),
        ui::NodeLink::new(1, 1, 6, 0),
        ui::NodeLink::new(2, 0, 6, 1),
        ui::NodeLink::new(3, 0, 4, 1),
        ui::NodeLink::new(3, 0, 5, 0),
        ui::NodeLink::new(4, 0, 6, 2),
        ui::NodeLink::new(6, 0, 7, 0),
        ui::NodeLink::new(6, 0, 8, 0),
    ]);
    let selected = signal(Vec::<u64>::new());
    let transform = signal(kivact::interaction::ViewTransform {
        offset: Point::new(300.0, 0.0),
        scale: 0.85,
    });
    col((node().style(style().position(Position::Relative).height(480)).child((
        ui::node_editor(data,connections, ui::NodeState { selection: selected, transform, ..ui::NodeState::new() }).can_connect(move |link| super::node_model::can_connect(&data.get_untracked(), link)).on_edit(super::node_model::edits(data, connections)).style(style().height(480)).test_id("creative-node-editor"),
        line((kivact::ui::select("Selection", keyed_options([("compositor", "Compositor"), ("geometry-nodes", "Geometry Nodes"), ("shader-editor", "Shader Editor")]), signal(Some("compositor".to_owned()))).placeholder("").style(style().width(136)),kivact::ui::toggle_button("Use Nodes", signal(true)))).style(style().position(Position::Absolute).bottom(8).left(8)),
        line((kivact::ui::menu("Add", ["Image", "Blur", "Mix", "Color Balance", "Keying"].into_iter().map(|label| MenuEntry::item(label, label)).collect::<Vec<_>>()).on_activate(move |label: &'static str| super::node_model::add(data, label)).offset(0.0).attach(kivact::ui::button("Add").variant(kivact::ui::ButtonVariant::Secondary).trailing(kivact::view::icon("chevron-down"))),kivact::ui::button("Frame All").variant(kivact::ui::ButtonVariant::Secondary).on_press(move || transform.set(kivact::interaction::ViewTransform{offset:Point::new(300.0,0.0),scale:0.85})))).style(style().position(Position::Absolute).top(8).right(8)),
    )), row((note(move || format!("{} nodes · {} links · {:.0}%", data.with(Vec::len), connections.with(Vec::len), transform.get().scale * 100.0)), spacer(), note("ヘッダーをドラッグ / 出力 → 入力で接続 / Del: 選択削除 / Ctrl-wheel: zoom / Middle: pan"))).style(style().padding(12))))
}
// end example
