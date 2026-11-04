//! Role: The Lyra scroll area: a scroll container with a slim border-colored bar.

use kivact::style::{Color, Direction, pct, style};
use kivact::view::{IntoView, View};
use kivact::view::{Node, NodeBuilder, Scroll, ScrollAxes, node, scroll};

/// A scrolling region (scroll-area.tsx). Created by [`scroll_area`].
///
/// Give the area a height (or a maximum height) with `.style(..)`; content
/// taller than that scrolls. The modifiers apply to the scroll container
/// itself.
pub struct ScrollArea {
    /// The scroll container; the caller's modifiers apply to it.
    viewport: Scroll,
}

/// A scroll area around `child`.
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
#[track_caller]
pub fn scroll_area(child: impl IntoView) -> ScrollArea {
    ScrollArea {
        viewport: scroll(child).style(
            style()
                .grow(1)
                .shrink(1)
                .min_height(0)
                .min_width(0)
                .width(pct(100))
                .scrollbar_width(10)
                .scrollbar_color(crate::Color::Border, Color::TRANSPARENT),
        ),
    }
}

impl ScrollArea {
    /// The axes that scroll (default [`ScrollAxes::Vertical`]), as the core
    /// [`Scroll::axes`](kivact::view::Scroll::axes).
    pub fn axes(mut self, axes: impl kivact::state::IntoValue<ScrollAxes>) -> Self {
        self.viewport = self.viewport.axes(axes);
        self
    }
}

impl NodeBuilder for ScrollArea {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        self.viewport.node_mut()
    }
}

impl IntoView for ScrollArea {
    fn into_view(self) -> View {
        node()
            .style(style().direction(Direction::Column))
            .child(self.viewport)
            .into_view()
    }
}
