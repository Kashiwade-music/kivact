//! Role: Conversation rows, bubbles and typing indicators with caller-owned state.

use crate::{Hue, Tokens, avatar, badge};
use kivact::Value;
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::i18n;
use kivact::motion::{Animated, Easing, Repeat, Transition};
use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::style::{Align, Color, ColorToken, TextOverflow, pct, style};
use kivact::view::{IntoText, IntoValue, IntoView, LiveRegion, Role, View, show};
/// A conversation row. Created by [`thread`].
pub struct Thread {
    root: Node,
    initials: Value<String>,
    portrait: Option<crate::slot::Slot>,
    title: Value<String>,
    preview: Value<String>,
    unread: Value<usize>,
}

/// A conversation row with selection and unread count supplied by the application.
///
/// [`Thread::portrait`] replaces the initials avatar with application-provided
/// avatar content.
#[track_caller]
pub fn thread(initials: impl IntoText, title: impl IntoText) -> Thread {
    let title = title.into_text_value();
    Thread {
        root: row(())
            .role(Role::Button)
            .focusable(true)
            .name(title)
            .selected(false)
            .style(
                style()
                    .font_size(14)
                    .line_height(1.5)
                    .padding(8)
                    .gap(10)
                    .align_items(Align::Center)
                    .radius(8)
                    .hover(style().background(ColorToken::SurfaceAlt))
                    .selected(
                        style()
                            .background(ColorToken::Surface)
                            .shadow(kivact::draw::Shadow::new(0, 1, 3, ColorToken::Shadow)),
                    ),
            ),
        initials: initials.into_text_value(),
        portrait: None,
        title,
        preview: String::new().into_value(),
        unread: 0usize.into_value(),
    }
}
impl Thread {
    /// Preview text; empty by default.
    pub fn preview(mut self, value: impl IntoText) -> Self {
        self.preview = value.into_text_value();
        self
    }
    /// Reactive unread count; zero by default.
    pub fn unread(mut self, value: impl IntoValue<usize>) -> Self {
        self.unread = value.into_value();
        self
    }
    /// Reactive selection; false by default.
    pub fn selected(mut self, value: impl IntoValue<bool>) -> Self {
        self.root = self.root.selected(value);
        self
    }

    /// An application-provided avatar view in place of the initials avatar,
    /// built once by `portrait` when the row mounts.
    pub fn portrait<V: IntoView>(mut self, portrait: impl FnOnce() -> V + 'static) -> Self {
        self.portrait = Some(crate::slot::slot(portrait));
        self
    }
}
impl NodeBuilder for Thread {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Thread {
    fn into_view(self) -> View {
        let Self {
            root,
            initials,
            portrait,
            title,
            preview,
            unread,
        } = self;
        let portrait = match portrait {
            Some(portrait) => portrait(),
            None => avatar(initials).size(32).into_view(),
        };
        root.child((
            portrait,
            column((
                text(title).style(style().font_size(14).weight(500).line_height(1.5)),
                text(preview).style(
                    style()
                        .font_size(12)
                        .line_height(1.5)
                        .color(ColorToken::TextMuted)
                        .text_wrap(false)
                        .text_overflow(TextOverflow::Ellipsis),
                ),
            ))
            .style(style().grow(1).min_width(0)),
            show(
                move || unread.get() > 0,
                move || {
                    badge(move || unread.get().to_string())
                        .hue(Hue::Pink)
                        .variant(crate::BadgeVariant::Count)
                        .into_view()
                },
            ),
        ))
        .into_view()
    }
}

/// An incoming typing bubble, with a polite status for assistive technology.
/// Its presence is controlled by the caller just like an ordinary message.
#[track_caller]
pub fn typing_message(initials: impl IntoText) -> Node {
    let phase = Animated::from(
        0.0f32,
        std::f32::consts::TAU,
        Transition::tween(1000, Easing::Linear)
            .repeat(Repeat::Forever)
            .decorative(false),
    );
    row((
        avatar(initials).size(24),
        node()
            .style(style().size(54, 30).radius_corners(4, 14, 14, 14))
            .style(style().background(crate::Color::SurfaceActive))
            .draw(move |draw, _| {
                let color = draw.theme().get::<Tokens>().text_faint;
                for index in 0..3 {
                    let bounce = (phase.get() - index as f32 * 0.95).sin().max(0.0) * 3.0;
                    draw.rounded_rect(
                        kivact::draw::Rect::new(
                            14.0 + index as f32 * 10.0,
                            12.0 - bounce,
                            6.0,
                            6.0,
                        ),
                        3.0,
                        color,
                    );
                }
            }),
    ))
    .role(Role::Status)
    .name(i18n::message("pop.typing.label", "Typing"))
    .live_region(LiveRegion::Polite)
    .style(style().gap(10).align_items(Align::Start).max_width(pct(78)))
}

/// A chat bubble, with direction, author initials and timestamp supplied explicitly.
/// `mine` is a constant: it decides the alignment and whether the author's
/// avatar is part of the tree, so it is fixed when the bubble is built.
#[track_caller]
pub fn message(content: impl IntoText) -> Message {
    let content = content.into_text_value();
    Message {
        group: message_group(move || vec![content.get()]),
    }
}
/// A single message with optional author, time and direction.
pub struct Message {
    group: MessageGroup,
}
impl Message {
    /// Timestamp; empty by default.
    pub fn timestamp(mut self, value: impl IntoText) -> Self {
        self.group = self.group.timestamp(value);
        self
    }
    /// Outgoing direction; false by default.
    pub fn mine(mut self, value: bool) -> Self {
        self.group = self.group.mine(value);
        self
    }
    /// Author initials; empty by default.
    pub fn initials(mut self, value: impl IntoText) -> Self {
        self.group = self.group.initials(value);
        self
    }
}
impl NodeBuilder for Message {
    fn node_mut(&mut self) -> &mut Node {
        self.group.node_mut()
    }
}
impl IntoView for Message {
    fn into_view(self) -> View {
        self.group.into_view()
    }
}

/// Several bubbles sent together, sharing one avatar and timestamp. `mine` is
/// a constant, as for [`message`].
#[track_caller]
pub fn message_group(content: impl IntoValue<Vec<String>>) -> MessageGroup {
    MessageGroup {
        root: row(()),
        content: content.into_value(),
        timestamp: String::new().into_value(),
        mine: false,
        initials: String::new().into_value(),
    }
}
/// Presentation builder returned by [`message_group`].
pub struct MessageGroup {
    root: Node,
    content: Value<Vec<String>>,
    timestamp: Value<String>,
    mine: bool,
    initials: Value<String>,
}
impl MessageGroup {
    /// Timestamp; empty by default.
    pub fn timestamp(mut self, value: impl IntoText) -> Self {
        self.timestamp = value.into_text_value();
        self
    }
    /// Outgoing direction; false by default.
    pub fn mine(mut self, value: bool) -> Self {
        self.mine = value;
        self
    }
    /// Author initials; empty by default.
    pub fn initials(mut self, value: impl IntoText) -> Self {
        self.initials = value.into_text_value();
        self
    }
}
impl NodeBuilder for MessageGroup {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for MessageGroup {
    fn into_view(self) -> View {
        let content = self.content;
        let timestamp = self.timestamp;
        let mine = self.mine;
        let initials = self.initials;

        let bubble = column((
            kivact::view::dynamic(content, move |paragraphs| {
                column(
                    paragraphs
                        .into_iter()
                        .enumerate()
                        .map(|(index, paragraph)| {
                            text(paragraph)
                                .style(
                                    style()
                                        .font_size(14)
                                        .padding_x(12)
                                        .padding_y(8)
                                        .radius_corners(
                                            if mine || index > 0 { 14 } else { 4 },
                                            if mine && index == 0 { 4 } else { 14 },
                                            14,
                                            14,
                                        ),
                                )
                                .style(move || {
                                    let t = &current_theme();
                                    let p = t.get::<Tokens>();
                                    style()
                                        .background(if mine { p.blue } else { p.surface_active })
                                        .color(if mine { Color::WHITE } else { p.text })
                                })
                                .into_view()
                        })
                        .collect::<Vec<_>>(),
                )
                .style(style().gap(4).align_items(if mine {
                    Align::End
                } else {
                    Align::Start
                }))
            }),
            text(timestamp).style(
                style()
                    .padding_x(4)
                    .font_size(11)
                    .color(ColorToken::TextMuted),
            ),
        ))
        .style(
            style()
                .gap(4)
                .align_items(if mine { Align::End } else { Align::Start }),
        );
        (if mine {
            self.root
                .child(bubble)
                .base_style(style().align_self(Align::End).max_width(pct(78)))
        } else {
            self.root
                .child((avatar(initials).size(24), bubble))
                .base_style(style().gap(10).align_items(Align::Start).max_width(pct(78)))
        })
        .into_view()
    }
}
