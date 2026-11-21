//! Role: The page of the Creative catalog.

use super::common::{col, keyed_options, note, spec};
use super::layout::*;
use catalog_shell::{Example, Page};
use kivact::prelude::{Element, IntoView, rgb, row, scroll, signal, text};
use kivact::style::{Align, style};
use kivact_ui_creative as ui;

pub(crate) fn page() -> Page {
    Page::new(
        "property-editor",
        "Property Editor",
        "Layout",
        include_str!("property_editor.rs"),
    )
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
            "Panel Actions",
            col((
                note("Click the header to open or close. Collapsing keeps the internal values."),
                kivact::ui::disclosure("Viewport Display", signal(true)).content(move || {
                    col((
                        kivact::ui::checkbox("Name", display_name),
                        kivact::ui::checkbox("Axes", display_axes),
                        kivact::ui::checkbox("Wireframe", display_wireframe),
                        ui::property_row(
                            "Display As",
                            kivact::ui::select(
                                keyed_options([
                                    ("textured", "Textured"),
                                    ("solid", "Solid"),
                                    ("wire", "Wire"),
                                ]),
                                display_as,
                            )
                            .label("Selection")
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
                    .description("Edit by drag, wheel, typing or keys.")
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
