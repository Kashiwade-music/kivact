//! Role: The Lyra message scroller that follows new content and offers a jump to the end.

use kivact::i18n::message;
use kivact::interaction::use_follow_end;
use kivact::style::{Color, Direction, Position, pct, style};
use kivact::ui::{ButtonVariant, ControlSize, button};
use kivact::view::{IntoView, View, node_ref};
use kivact::view::{Node, NodeBuilder, column, icon, node, scroll};

/// A scrolling column of messages that sticks to the bottom (message-scroller.tsx).
///
/// Created by [`message_scroller`].
pub struct MessageScroller {
    root: Node,
    content: View,
}

/// A message scroller around `content` (usually a column of messages).
///
/// The scroller fills its parent, so give the parent a height. It starts
/// following: it keeps the end of the content in view as the content grows.
/// Scrolling up away from the end stops following; reaching the end again, or
/// pressing the "Scroll to end" button that appears while the end is out of
/// view, follows again. The messages are spaced 24 px apart.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn chat() -> impl IntoView {
///     basic::message_scroller(column((0..40).map(|n| text(format!("Message {n}"))).collect::<Vec<_>>()))
///         .style(style().height(240).width(320))
/// }
/// ```
#[track_caller]
pub fn message_scroller(content: impl IntoView) -> MessageScroller {
    MessageScroller {
        root: node().style(
            style()
                .position(Position::Relative)
                .direction(Direction::Column)
                .min_height(0),
        ),
        content: content.into_view(),
    }
}

impl NodeBuilder for MessageScroller {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// How close to the end, in pixels, counts as at the end.
const SLACK: f64 = 1.5;

impl IntoView for MessageScroller {
    fn into_view(self) -> View {
        let viewport = node_ref();
        let body = node_ref();
        let follow = use_follow_end(viewport, body, SLACK, None);
        let distance = follow.distance();
        let scroller = scroll(
            column(self.content)
                .node_ref(body)
                .style(style().gap(24).shrink(0).min_height(pct(100)).min_width(0)),
        )
        .node_ref(viewport)
        .style(
            style()
                .grow(1)
                .shrink(1)
                .min_height(0)
                .min_width(0)
                .width(pct(100))
                .scrollbar_width(10)
                .scrollbar_color(crate::Color::Border, Color::TRANSPARENT),
        );
        let button = button(message("basic.message_scroller.end", "Scroll to end"))
            .variant(ButtonVariant::Outline)
            .hide_label(true)
            .icon(icon("arrow-down"))
            .size(ControlSize::Small)
            .hidden(move || distance.get() <= SLACK)
            .on_press(move || follow.jump_to_end())
            .style(
                style()
                    .position(Position::Absolute)
                    .left(pct(50) - 14)
                    .bottom(16),
            );
        self.root.child(scroller).child(button).into_view()
    }
}
