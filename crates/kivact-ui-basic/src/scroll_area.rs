//! Role: The Lyra scroll area: a scroll container with a slim border-colored bar.

use kivact::control::{Node, NodeBuilder, node, scroll};
use kivact::style::{Color, Direction, pct, style};
use kivact::view::{IntoView, Orientation, View};

/// A scrolling region (scroll-area.tsx). Created by [`scroll_area`].
///
/// Give the area a height (or a maximum height) with `.style(..)`; content
/// taller than that scrolls.
pub struct ScrollArea {
    root: Node,
    child: View,
    orientation: Orientation,
    both_axes: bool,
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
        root: node().style(style().direction(Direction::Column)),
        child: child.into_view(),
        orientation: Orientation::Vertical,
        both_axes: false,
    }
}

impl ScrollArea {
    /// The direction that scrolls (default [`Orientation::Vertical`]), as
    /// the core [`Scroll::orientation`](kivact::control::Scroll::orientation);
    /// [`Orientation::Unspecified`] means the default. It shadows the
    /// accessibility setter of the same name on purpose. A constant: fixed
    /// when the scroll container is built.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Scroll in both directions, as the core
    /// [`Scroll::both_axes`](kivact::control::Scroll::both_axes).
    pub fn both_axes(mut self) -> Self {
        self.both_axes = true;
        self
    }
}

impl NodeBuilder for ScrollArea {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for ScrollArea {
    fn into_view(self) -> View {
        let mut viewport = scroll(self.child).orientation(self.orientation);
        if self.both_axes {
            viewport = viewport.both_axes();
        }
        let viewport = viewport.style(
            style()
                .grow(1)
                .shrink(1)
                .min_height(0)
                .min_width(0)
                .width(pct(100))
                .scrollbar_width(10)
                .scrollbar_color(crate::Color::Border, Color::TRANSPARENT),
        );
        self.root.child(viewport).into_view()
    }
}
