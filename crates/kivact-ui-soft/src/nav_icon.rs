//! Role: Display a round accent icon for navigation headings.

use kivact::view::NodeBuilder;
use kivact::view::node;

use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::IntoView;

/// Mount an icon once inside the round accent badge used by navigation headings.
pub fn nav_icon(icon: impl IntoView) -> impl kivact::view::Component {
    node()
        .with_node(move |node| {
            node.base_style(|| {
                let t = kivact::style::use_tokens::<crate::Tokens>();
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
        })
        .child(icon)
}
