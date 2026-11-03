//! Role: Display a round accent icon for navigation headings.
use crate::tokens;
use kivact::control::{Node, node};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Justify, pct, style};
use kivact::view::IntoView;

pub fn nav_icon(icon: impl IntoView) -> Node {
    node()
        .base_style(|| {
            let t = tokens();
            let size = t.sizes.control_md;
            style()
                .size(size, size)
                .shrink(0)
                .radius(pct(50))
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .background(t.colors.accent)
                .color(t.colors.on_accent)
        })
        .child(icon)
}
