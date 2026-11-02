//! Role: The Lyra message scroller that follows new content and offers a jump to the end.

use kivact::control::{Node, NodeBuilder, column, icon, node, scroll};
use kivact::draw::Point;
use kivact::i18n::message;
use kivact::style::{Color, Direction, Position, pct, style};
use kivact::ui::{ButtonVariant, ControlSize, button};
use kivact::view::{IntoView, ScrollBehavior, View, node_ref};
use kivact::{effect, effect_on, memo, signal};

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
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// How close to the end, in pixels, counts as at the end.
const SLACK: f32 = 1.5;

impl IntoView for MessageScroller {
    fn into_view(self) -> View {
        // Whether the end stays in view: the scroller's own state, like its
        // scroll offset.
        let follow = signal(true);
        let viewport = node_ref();
        let body = node_ref();
        let (view_rect, body_rect) = (viewport.rect(), body.rect());
        let offset = viewport.scroll_offset();
        let view_height = memo(move || view_rect.get().map_or(0.0, |r| r.height));
        let body_height = memo(move || body_rect.get().map_or(0.0, |r| r.height));
        let is_at_end = move |offset: Point, view: f32, body: f32| offset.y + view >= body - SLACK;
        let at_end = memo(move || is_at_end(offset.get(), view_height.get(), body_height.get()));
        let to_end = move || {
            let max = (body_height.get_untracked() - view_height.get_untracked()).max(0.0);
            viewport.scroll_to(Point::new(0.0, max), ScrollBehavior::Instant);
        };
        // A scroll by the user decides whether to follow. The offset the
        // scroller starts with is not a scroll.
        effect_on(offset, move |&offset| {
            let now = is_at_end(
                offset,
                view_height.get_untracked(),
                body_height.get_untracked(),
            );
            let measured = view_height.get_untracked() > 0.0 && body_height.get_untracked() > 0.0;
            if measured && now != follow.get_untracked() {
                follow.set(now);
            }
        });
        // New content, or following turned on, moves to the end.
        effect(move || {
            body_height.get();
            if follow.get() {
                to_end();
            }
        });
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
            .hidden(move || at_end.get())
            .on_press(move || {
                follow.set(true);
                to_end();
            })
            .style(
                style()
                    .position(Position::Absolute)
                    .left(pct(50) - 14)
                    .bottom(16),
            );
        self.root.child(scroller).child(button).into_view()
    }
}
