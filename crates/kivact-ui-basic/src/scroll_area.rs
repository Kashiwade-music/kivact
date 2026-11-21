//! Role: The Lyra scroll area: a scroll container with a slim border-colored bar.

use crate::BORDER;
use kivact::component;
use kivact::style::{Color, pct, style};
use kivact::view::{Element, ScrollAxes, View, scroll};

/// A scrolling region (scroll-area.tsx). Created by [`scroll_area`].
///
/// Give the area a height (or a maximum height) with `.style(..)`; content
/// taller than that scrolls. The modifiers apply to the scroll container
/// itself. The axes that scroll default to [`ScrollAxes::Vertical`].
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn tags() -> impl IntoView {
///     basic::scroll_area(column((0..50).map(|n| text(format!("v1.2.0-beta.{n}"))).collect::<Vec<_>>()))
///         .style(style().height(200).width(192))
/// }
/// ```
#[component]
pub fn scroll_area(
    child: View,
    #[prop(default = ScrollAxes::Vertical)] axes: kivact::state::Value<ScrollAxes>,
) -> impl Element {
    scroll(child).axes(axes).style(
        style()
            .flex_shrink(1.0)
            .min_height(0)
            .min_width(0)
            .width(pct(100))
            .scrollbar_width(10)
            .scrollbar_color(BORDER, Color::TRANSPARENT),
    )
}
