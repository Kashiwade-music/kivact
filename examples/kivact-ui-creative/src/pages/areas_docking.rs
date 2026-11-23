//! Role: The page of the Creative catalog.

use super::common::{col, spec, specs};
use super::layout::*;
use catalog_shell::{Example, Page};
use kivact::prelude::{Element, IntoView, signal};
use kivact::style::style;
use kivact_ui_creative as ui;

pub(crate) fn page() -> Page {
    Page::new(
        "areas-docking",
        "Areas & Docking",
        "Layout",
        include_str!("areas_docking.rs"),
    )
    .description("Editor types, splits, maximize and document tabs.")
    .example(Example::new("areas-docking", "Areas & Docking", "", demo))
}

// example: areas-docking
fn demo() -> impl IntoView {
    let document=signal("import kivact as kv\n\nscene = kv.context.scene\nfor obj in scene.objects:\n    if obj.type == \"MESH\":\n        obj.modifiers.new(\"Bevel\", \"BEVEL\")".to_string());
    let command = signal(String::new());
    let editor_sizes = signal(vec![0.55f64, 0.45]);
    let output = signal("KIVACT Console 0.1 — type help\n".to_string());
    let layout = signal(ui::DockLayout::new(ui::DockNode::split(
        ui::DockAxis::Horizontal,
        [
            (
                0.34,
                ui::DockNode::tabs(vec![
                    ui::DockTab::new("info", "Info").icon("info"),
                    ui::DockTab::new("console", "Console"),
                ]),
            ),
            (
                0.66,
                ui::DockNode::tabs(vec![
                    ui::DockTab::new("text", "Text Editor"),
                    ui::DockTab::new("image", "Image Viewer").icon("eye"),
                    ui::DockTab::new("properties", "Properties"),
                ]),
            ),
        ],
    )));
    let dock = ui::dock(layout, move |panel: &'static str| {
        dock_panel(panel, document, command, output, editor_sizes)
    })
    .test_id("creative-dock-workspace")
    .style(style().height(390));
    col((
        dock.style(style().margin(6).flex_shrink(0)),
        specs((
            document_tabs_spec(),
            spec("Panel Tab Group", "panel tabs", effects_panel()),
        )),
    ))
}
// end example
