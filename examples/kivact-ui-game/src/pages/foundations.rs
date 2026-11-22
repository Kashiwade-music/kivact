//! Role: The Foundations page of the game catalog: Color, type and shape foundations of the console.

use crate::support::common::{cards, display_family, line, list, meta, specimen};
use crate::support::specimens::{ShapeKind, shape_tile, swatch};
use catalog_shell::{Example, Page};
use kivact::interaction::{Tone, use_toaster};
use kivact::platform;
use kivact::prelude::*;
use kivact::style::Transform;
use kivact::view::Node;
use kivact_ui_game as g;
use std::time::Duration;

/// The page: its one demo, with its explanation.
pub(crate) fn page() -> Page {
    Page::new(
        "foundations",
        "Foundations",
        "Overview",
        include_str!("foundations.rs"),
    )
    .description("Color, type and shape foundations of the console")
    .example(Example::new(
        "foundations",
        "Foundations",
        "The color tokens, type scale, shape motifs and density of the console theme.",
        demo,
    ))
}

// example: foundations
fn demo() -> impl IntoView {
    let density = signal(g::Density::Comfortable);
    let mode = memo(move || use_theme().get().mode());
    provide_theme(
        move || {
            g::theme_with(
                mode.get(),
                g::ThemeOptions {
                    density: density.get(),
                    ..crate::options()
                },
            )
        },
        move || column((density_controls(density), hero(mode), foundations(mode))),
    )
}
// end example

pub(crate) fn foundations(mode: Value<kivact::style::ThemeMode>) -> Node {
    let swatches = [
        ("Primary", "--c-primary", 0x1283f0, 0x2b8ceb),
        ("Primary Strong", "--c-primary-strong", 0x0a62c2, 0x1a6cc4),
        ("Signal Cyan", "--c-cyan", 0x2ec4f0, 0x2ec4f0),
        ("Accent Yellow", "--c-accent", 0xffc928, 0xffc928),
        ("Command Navy", "--c-navy", 0x0c1f36, 0x1a3150),
        ("Success", "--c-success", 0x12a06e, 0x2cc48a),
        ("Warning", "--c-warning", 0xd98300, 0xf5a524),
        ("Danger", "--c-danger", 0xe0364d, 0xff5a6e),
        ("Surface", "--surface", 0xffffff, 0x0f1c2c),
        ("Canvas", "--bg", 0xedf2f7, 0x08111c),
        ("Line Strong", "--line-strong", 0xafc0d2, 0x33506f),
        ("Ink", "--text", 0x13233a, 0xe4edf7),
    ];
    list((specimen("TKN-01", "Color tokens", cards(swatches.into_iter().map(|(name, token, light, dark)| {
        let color = move || if mode.get() == kivact::style::ThemeMode::Light { light } else { dark };
        swatch(name, move || format!("#{:06X}", color()), move || rgb(color()))
            .test_id(format!("game-swatch-{token}"))
            .on_press(move || {
                let value = format!("var({token})");
                let label = format!("{value} = #{:06X}", color());
                spawn(async move {
                    if platform::copy_text(value.clone()).await.is_ok() {
                        use_toaster().push(Toast::new("Copied").body(label).tone(Tone::Success).duration(Some(Duration::from_millis(2400))));
                    } else {
                        use_toaster().push(Toast::new("Copy failed").body(value).tone(Tone::Warning).duration(Some(Duration::from_millis(3000))));
                    }
                });
            })
    }).collect::<Vec<_>>(), 140.).style(style().gap(10))),
        cards((specimen("TKN-02", "Typography", list((
            type_row("DISPLAY", text("MISSION 07").style(style().font_family(display_family()).font_size(36).font_style(kivact::style::FontStyle::Italic).font_weight(800))),
            type_row("H1 / 22", text("Approve operation logs").style(style().font_size(22).font_weight(900))),
            type_row("H2 / 16", text("Request workflow settings").style(style().font_size(16).font_weight(700))),
            type_row("BODY / 13", text("Body text on work screens is 13 px; supporting information drops to 11.5 px.")),
            type_row("MONO / 12", g::mono("OPS-2041 • 2026-10-03T09:40:00+09:00").style(style().font_size(12))),
        ))), specimen("TKN-03", "Shape language", list((line((shape_tile(ShapeKind::Cut, "CUT 14"), shape_tile(ShapeKind::Skew, "SKEW -12°"),
            shape_tile(ShapeKind::Bracket, "BRACKET"), shape_tile(ShapeKind::Stripe, "STRIPE"))),
            text("No rounded corners. Primary actions are -12° parallelograms, surfaces have cut corners, focus areas use HUD brackets and danger areas use warning stripes.")
                .style(style().color(color::TEXT_MUTED)),
        )))), 460.)))
}

fn type_row(label: &str, content: impl IntoView) -> Node {
    row((
        g::mono(label.to_owned()).style(style().width(88).color(color::TEXT_MUTED)),
        content,
    ))
    .style(
        style()
            .gap(14)
            .padding_y(8)
            .align_items(Align::Baseline)
            .border_bottom(1, color::BORDER),
    )
}

fn hero(mode: Value<kivact::style::ThemeMode>) -> Node {
    row((column((row((node().style(style().size(22, 8).background(rgb(0xffc928)).transform(Transform::new().skew_x(-12))), g::mono("DESIGN SYSTEM // IDEA 20261003-0940").style(style().color(color::ACCENT).letter_spacing(2.2)))).style(style().gap(10).align_items(Align::Center)),
        row((text("COMPONENT").style(style().font_family(display_family()).font_size(54).font_weight(800).font_style(kivact::style::FontStyle::Italic).line_height(0.95)),
            node().style(style().background(color::ACCENT).padding_x(8).transform(Transform::new().skew_x(-12)))
                .child(text("ARCHIVE").style(style().font_family(display_family()).font_size(54).font_weight(800).font_style(kivact::style::FontStyle::Italic).line_height(0.95)
                    .color(color::ACCENT_TEXT).transform(Transform::new().skew_x(12)))))).style(style().gap(8).align_items(Align::Center)),
        text("A catalog of UI components for native business SaaS, built in blue and white with skewed shapes, cut corners and HUD brackets. Decoration is minimal; information density and reliable operation come first.")
            .style(style().max_width(470).font_size(13.5).color(color::TEXT_MUTED)),
        line((meta("COMPONENTS 45"), meta("STACK KIVACT / RUST"), meta(move || format!("THEME {}", if mode.get() == kivact::style::ThemeMode::Light { "LIGHT" } else { "DARK" })), meta("ICONS Tabler (MIT)"))),
    )).style(style().gap(12).flex_grow(1)), g::halo()))
        .style(style().min_height(250).padding_y(28).padding_x(32).gap(24).align_items(Align::Center).background(color::SURFACE).border(1, color::BORDER).border_left(6, color::ACCENT))
}

fn density_controls(density: Signal<g::Density>) -> impl IntoView {
    kivact::ui::button_group(
        "Console actions",
        (
            kivact::ui::button("Standard")
                .variant(kivact::ui::Slant)
                .test_id("game-density-comfortable")
                .size(kivact::ui::ControlSize::Small)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .with_node(|node| node.selected(move || density.get() == g::Density::Comfortable))
                .style(
                    style().selected(style().background(color::ACCENT).color(color::ACCENT_TEXT)),
                )
                .on_press(move || density.set(g::Density::Comfortable)),
            kivact::ui::button("Dense")
                .variant(kivact::ui::Slant)
                .test_id("game-density-compact")
                .size(kivact::ui::ControlSize::Small)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .with_node(|node| node.selected(move || density.get() == g::Density::Compact))
                .style(
                    style().selected(style().background(color::ACCENT).color(color::ACCENT_TEXT)),
                )
                .on_press(move || density.set(g::Density::Compact)),
        ),
    )
}
