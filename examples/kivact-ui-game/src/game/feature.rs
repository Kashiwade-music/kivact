//! Role: Catalog feature presentation card.
use kivact::draw::{Path, Shape};
use kivact::geometry::Point;
use kivact::state::IntoText;
use kivact::style::{Align, ColorToken, Justify, pct, style};
use kivact::view::{Node, column, node, row, text};
/// Feature card with a clipped 18 px corner and a cyan command icon.
#[track_caller]
pub(super) fn feature_card(
    title: impl IntoText,
    body: impl IntoText,
    action: impl kivact::view::IntoView,
) -> Node {
    column((
        row((
            node()
                .style(
                    style()
                        .size(34, 34)
                        .background(kivact_ui_game::Color::Navy)
                        .color(kivact_ui_game::Color::Cyan)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center),
                )
                .child(kivact::view::icon("radar")),
            text(title).style(style().font_size(14).weight(700)),
        ))
        .style(style().gap(10).align_items(Align::Center)),
        text(body).style(style().font_size(12).color(ColorToken::TextMuted)),
        action,
    ))
    .style(
        style()
            .padding(16)
            .gap(8)
            .background(ColorToken::SurfaceAlt)
            .border(1, ColorToken::Border)
            .clip_path(
                Shape::polygon()
                    .point(0, 0)
                    .point(pct(100) - 18, 0)
                    .point(pct(100), 18)
                    .point(pct(100), pct(100))
                    .point(0, pct(100)),
            ),
    )
    .draw_over(|d, s| {
        d.fill_path(
            &Path::new()
                .move_to(Point::new(s.width - 18., 0.))
                .line_to(Point::new(s.width, 18.))
                .line_to(Point::new(s.width - 18., 18.))
                .close(),
            ColorToken::Accent,
        )
    })
}
