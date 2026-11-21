//! Role: The design tokens page: the colors, the type scale and the radii and shadows of the Pop theme.

use kivact::prelude::*;
use kivact::style::{Align, Color, GridPlacement, Justify, Track, color, pct, rgb, rgba, style};
use kivact::view::Node;
use kivact_ui_pop as ui;
use catalog_shell::{Example, Page};
use super::common::*;
/// The page of the design tokens.
pub fn page() -> Page {
    Page::new(
        "tokens",
        "Design Tokens",
        "Components",
        include_str!("tokens.rs"),
    )
    .description("The colors, type scale, radii and shadows that every Pop component draws with.")
    .example(Example::new(
        "tokens",
        "Tokens",
        "Click a swatch to copy its hex value.",
        tokens_demo,
    ))
}

fn tokens_demo() -> impl IntoView {
    // example: tokens
    let swatches = color_swatches(notify());
    let type_scale = type_samples();
    let mut shapes = shape_samples();
    shapes.push(cut_corner_sample());
    section(
        "01",
        "Design Tokens",
        "Design tokens — click a swatch to copy it",
        cards_grid((
            card(
                "Color palette",
                grid(swatches).style(
                    style()
                        .grid_template_columns(Track::repeat_auto_fill(Track::minmax(
                            116,
                            Track::fr(1),
                        )))
                        .gap(12),
                ),
            )
            .style(style().grid_column(GridPlacement::span(8))),
            card("Typography", column(type_scale).style(style().gap(12)))
                .style(style().grid_column(GridPlacement::span(4))),
            card(
                "Radius, shadow and elevation",
                grid(shapes).style(
                    style()
                        .grid_template_columns(Track::repeat_auto_fill(Track::minmax(
                            112,
                            Track::fr(1),
                        )))
                        .gap(16),
                ),
            )
            .style(style().grid_column(GridPlacement::span(12))),
        )),
    )
    // end example
}

/// The named swatches of the palette; a press copies the hex value.
fn color_swatches(notify: Callback<(ui::Hue, String, String)>) -> Vec<Node> {
    [
        ("Pop Blue", 0x2b8bd8),
        ("Deep Blue", 0x1a66a8),
        ("Sky Cyan", 0x5cbde8),
        ("Signal Yellow", 0xffdc3c),
        ("Talk Pink", 0xff7cb0),
        ("Mint", 0x2fcfa6),
        ("Alert Red", 0xef4458),
        ("Orange", 0xffa53c),
        ("Lavender", 0x9d86ff),
        ("Navy Ink", 0x1d2b4a),
        ("Slate 50", 0xf8fafc),
        ("Night", 0x0b1120),
    ]
    .into_iter()
    .map(|(name, hex)| {
        column((
            node().style(style().height(52).background(rgb(hex))),
            column((
                text(name).style(style().font_size(12.5).font_weight(500)),
                text(format!("#{hex:06X}")).style(
                    style()
                        .font_family("monospace")
                        .font_size(11)
                        .color(color::TEXT_MUTED),
                ),
            ))
            .style(style().padding_x(10).padding_top(8).padding_bottom(10)),
        ))
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .on_press(move || {
            spawn(kivact::platform::copy_text(format!("#{hex:06X}")));
            notify.call((
                ui::Hue::Blue,
                "Copied".into(),
                format!("#{hex:06X}"),
            ));
        })
        .style(
            style()
                .border(1, color::BORDER)
                .border_radius(10.0)
                .hover(style().shadow(Shadow::new(0, 4, 6, color::SHADOW)))
                .focus(style().outline(3, color::FOCUS)),
        )
    })
    .collect::<Vec<_>>()
}
