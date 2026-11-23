//! Role: The feature card of the game catalog.
use kivact::draw::Path;
use kivact::geometry::Point;
use kivact::state::IntoText;
use kivact::style::Shape;
use kivact::style::{Align, Justify, color, pct, style};
use kivact::view::{Node, column, node, row, text};
use kivact_ui_game as g;
/// Feature card with a clipped 18 px corner and a cyan command icon.
#[track_caller]
pub(crate) fn feature_card(
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
                        .background(g::color::NAVY)
                        .color(g::color::CYAN)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center),
                )
                .child(kivact::view::icon("radar")),
            text(title).style(style().font_size(14).font_weight(700)),
        ))
        .style(style().gap(10).align_items(Align::Center)),
        text(body).style(style().font_size(12).color(color::TEXT_MUTED)),
        action,
    ))
    .style(
        style()
            .padding(16)
            .gap(8)
            .background(color::SURFACE_ALT)
            .border(1, color::BORDER)
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
            color::ACCENT,
        )
    })
}
