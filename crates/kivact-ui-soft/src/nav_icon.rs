//! Role: Display a round accent icon for navigation headings.
use crate::color as c;
use crate::metrics as m;
use crate::radius as r;

use kivact::component;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::style;
use kivact::view::Element;
use kivact::view::View;
use kivact::view::node;

/// Mount an icon once inside the round accent badge used by navigation headings.
#[component]
pub fn nav_icon(icon: View) -> impl Element {
    node()
        .with_node(move |node| {
            node.base_style(|| {
                style()
                    .size(m::CONTROL_MD, m::CONTROL_MD)
                    .flex_shrink(0)
                    .border_radius(r::RADIUS_FULL)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .background(c::ACCENT)
                    .color(c::ON_ACCENT)
            })
        })
        .child(icon)
}
