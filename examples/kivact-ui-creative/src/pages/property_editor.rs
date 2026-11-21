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
    Page::new("property-editor", "Property Editor", "Layout", include_str!("property_editor.rs"))
        .description("Collapsible sections, properties, keyframe state and modifiers.")
        .example(Example::new("property-editor", "Property Editor", "", demo))
}

// example: property-editor
fn demo() -> impl IntoView {
    let tab = signal(2_usize);
    let display_color = signal(rgb(0xffffff));
    let display_name = signal(false);
    let display_axes = signal(true);
    let display_wireframe = signal(false);
    let display_as = signal(Some("textured".to_owned()));
    row((
        inspector_tabs(tab),
        scroll(col((
            text("◇ Cube  ›  Object").style(style().padding_y(8)),
            ui::panel_group(
                signal(vec![
                    ui::InspectorPanel::new(1, "Transform"),
                    ui::InspectorPanel::new(2, "Delta Transform").collapsed(),
                    ui::InspectorPanel::new(3, "Relations"),
                    ui::InspectorPanel::new(4, "Visibility"),
                    ui::InspectorPanel::new(5, "Modifiers"),
                ]),
                move |panel| inspector_panel(panel.get_untracked().id),
            ),
        )))
        .style(style().width(278).height(556).padding_x(5)),
        spec(
            "Panel Behaviour",
            "パネル操作",
            col((
                note("ヘッダーをクリックで開閉。内部値を保持したまま折りたたみます。"),
                kivact::ui::disclosure("Viewport Display", signal(true)).content(move || {
                    col((
                        kivact::ui::checkbox("Name", display_name),
                        kivact::ui::checkbox("Axes", display_axes),
                        kivact::ui::checkbox("Wireframe", display_wireframe),
                        ui::property_row(
                            "Display As",
                            kivact::ui::select(
                                "Selection",
                                keyed_options([
                                    ("textured", "Textured"),
                                    ("solid", "Solid"),
                                    ("wire", "Wire"),
                                ]),
                                display_as,
                            )
                            .placeholder(""),
                        ),
                        ui::property_row("Color", ui::color_popup("Viewport color", display_color))
                            .test_id("creative-property-display-color"),
                    ))
                    .role(kivact::view::Role::List)
                }),
                kivact::ui::alert("Property Editing")
                    .tone(kivact::interaction::Tone::Info)
                    .with_node(|node| {
                        node.role(kivact::view::Role::Status)
                            .live_region(kivact::view::LiveRegion::Polite)
                    })
                    .description("ドラッグ・ホイール・直接入力・キー操作で編集。")
                    .variant(ui::NoticeVariant)
                    .dismiss_content(kivact::view::icon("x"))
                    .dismiss_label(kivact::i18n::message(
                        "creative.notice.dismiss",
                        "Dismiss notification",
                    ))
                    .icon(kivact::view::icon("info")),
            )),
        )
        .style(style().width(316)),
    ))
    .style(style().align_items(Align::Start))
}
// end example
