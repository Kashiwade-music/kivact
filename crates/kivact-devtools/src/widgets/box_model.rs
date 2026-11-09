//! Role: The nested boxes of a node's box model.

use kivact::geometry::{Point, Rect};
use kivact::prelude::*;
use kivact_devtools_protocol as p;

/// The margin, border, padding and content boxes of a node.
pub(crate) fn box_model(bounds: Value<Option<p::BoxModel>>) -> impl IntoView {
    canvas(move |draw, size| {
        let Some(bounds) = bounds.get() else {
            return;
        };
        let margin = &bounds.margin;
        let width = margin.width.max(1.0);
        let height = margin.height.max(1.0);
        let scale = ((size.width - 16.0).max(0.0) / width)
            .min((size.height - 16.0).max(0.0) / height)
            .max(0.0);
        let origin = Point::new(
            (size.width - width * scale) / 2.0,
            (size.height - height * scale) / 2.0,
        );
        let to_rect = |rect: &p::Rect| {
            Rect::new(
                origin.x + (rect.x - margin.x) * scale,
                origin.y + (rect.y - margin.y) * scale,
                (rect.width * scale).max(1.0),
                (rect.height * scale).max(1.0),
            )
        };
        draw.fill_rect(to_rect(margin), ColorToken::SurfaceAlt);
        draw.stroke_rect(to_rect(margin), 1.0, ColorToken::Border);
        draw.fill_rect(to_rect(&bounds.border), ColorToken::Surface);
        draw.stroke_rect(to_rect(&bounds.border), 1.0, ColorToken::Border);
        draw.fill_rect(
            to_rect(&bounds.padding),
            Color::token(ColorToken::Selection).with_alpha(0.35),
        );
        draw.fill_rect(
            to_rect(&bounds.content),
            Color::token(ColorToken::Accent).with_alpha(0.35),
        );
    })
    .style(style().width(pct(100)).height(180))
}
