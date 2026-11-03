//! Role: Keep content clipped to a specified aspect ratio.
use kivact::control::{Node, NodeBuilder, node};
use kivact::primitive::NodeExt;
use kivact::style::{Position, pct, style};
use kivact::view::IntoView;

/// A full-width box whose height follows the supplied width-to-height ratio.
pub fn aspect_ratio(ratio: f32, children: impl IntoView) -> Node {
    node()
        .base_style(style().width(pct(100)).aspect_ratio(ratio).clip(true))
        .child(
            node()
                .style(
                    style()
                        .position(Position::Absolute)
                        .top(0)
                        .right(0)
                        .bottom(0)
                        .left(0),
                )
                .child(children),
        )
}
