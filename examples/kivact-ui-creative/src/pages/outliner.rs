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
    Page::new("outliner", "Outliner", "Layout", include_str!("outliner.rs"))
        .description("Hierarchy, selection, visibility and lock.")
        .example(Example::new("outliner", "Outliner", "", demo))
}

// example: outliner
fn demo() -> impl IntoView {
    let selected = signal(vec![4u64]);
    let filter = signal(String::new());
    let renaming = signal(None);
    let nodes = signal(vec![
        ui::TreeItem::new(1, None, "Scene Collection", "box"),
        ui::TreeItem::new(2, Some(1), "Hero", "folder"),
        ui::TreeItem::new(3, Some(2), "Camera", "camera"),
        ui::TreeItem::new(23, Some(3), "Camera.001", "camera"),
        ui::TreeItem::new(4, Some(2), "Cube", "box"),
        ui::TreeItem::new(5, Some(4), "Modifiers", "settings"),
        ui::TreeItem::new(6, Some(5), "Bevel", "settings"),
        ui::TreeItem::new(7, Some(5), "Subdivision", "settings"),
        ui::TreeItem::new(8, Some(4), "Cube", "box"),
        ui::TreeItem::new(9, Some(8), "Metal.002", "palette"),
        ui::TreeItem::new(10, Some(2), "Key Light", "bulb"),
        ui::TreeItem::new(11, Some(10), "Area", "bulb"),
        ui::TreeItem::new(12, Some(2), "Fill Light", "bulb"),
        ui::TreeItem::new(13, Some(12), "Point", "bulb"),
        ui::TreeItem::new(14, Some(1), "Environment", "folder"),
        ui::TreeItem::new(15, Some(14), "Ground", "box"),
        ui::TreeItem::new(16, Some(15), "Plane", "box"),
        ui::TreeItem::new(17, Some(14), "Sky Dome", "box"),
        ui::TreeItem::new(18, Some(17), "Sphere", "box"),
        ui::TreeItem::new(19, Some(14), "Trees [Instances]", "stack"),
        ui::TreeItem::new(20, Some(1), "Rig", "folder"),
        ui::TreeItem::new(21, Some(20), "Armature", "hierarchy"),
        ui::TreeItem::new(22, Some(21), "Bones", "hierarchy"),
    ]);
    let mode = signal(Some("scene".to_owned()));
    let blocks = signal(vec![
        ui::TreeItem::new(101, None, "Kivact File", "folder"),
        ui::TreeItem::new(102, Some(101), "Scenes (1)", "stack"),
        ui::TreeItem::new(103, Some(102), "Scene", "box"),
        ui::TreeItem::new(104, Some(101), "Objects (8)", "box"),
        ui::TreeItem::new(105, Some(101), "Materials (5)", "palette"),
        ui::TreeItem::new(106, Some(105), "Metal.002", "palette"),
        ui::TreeItem::new(107, Some(101), "Images (3)", "camera"),
    ]);
    specs((
        col((
            row((kivact::ui::select("Outliner mode", keyed_options([("scene", "▧"), ("kivact-file", "Kivact File")]), mode).placeholder("").name("Outliner mode").style(style().width(32).padding(0)),kivact::ui::text_input("Search Outliner", filter).placeholder("Search").style(style().flex_grow(1)),kivact::ui::button("Outliner filter").variant(kivact::ui::ButtonVariant::Secondary).hide_label(true).icon(kivact::view::icon("filter")),kivact::ui::button("Rename selected").variant(kivact::ui::ButtonVariant::Secondary).hide_label(true).icon(kivact::view::icon("folder")).on_press(move||renaming.set(selected.with(|v|v.first().copied())))))
                .style(style().height(40).padding_x(8).gap(6)),
            scroll(dynamic(mode,move|mode|{
                super::layout_reducers::outliner(if mode.as_deref()==Some("scene") { nodes } else { blocks }, ui::OutlinerState { expanded: (signal(if mode.as_deref()==Some("scene") { vec![1,2,4,14,19] } else { vec![101,102] })), selection: selected, renaming}).filter(filter).test_id("creative-scene-outliner")
            })).style(style().height(338).min_height(0)),
        )).style(style().gap(0).grid_column(GridPlacement::span(2)).height(380)),
        spec("Interaction","操作",col((
            note("クリックで選択、Ctrl 追加、Shift 範囲。ダブルクリック / F2 で改名。矢印で階層を開閉、ドラッグで移動。"),
            note(move||format!("ACTIVE       {}\nSELECTED     {}\nROWS         13",nodes.with(|v|v.iter().find(|n|selected.with(|s|s.contains(&n.id))).map_or("—",|n|n.label.as_str()).to_owned()),selected.with(Vec::len))),
        ))).style(style().height(380)),
    ))
}
// end example
