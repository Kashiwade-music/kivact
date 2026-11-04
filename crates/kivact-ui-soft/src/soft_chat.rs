//! Role: Neutral chat messages, lists, and scroll layout.

use std::hash::Hash;

use crate::tokens;
use kivact::control::{Node, NodeBuilder, column, node, row, scroll, text};
use kivact::i18n::message;
use kivact::interaction::use_follow_end;
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Direction, Justify, Length, Position, pct, style};
use kivact::view::{
    HandlerResult, IntoText, IntoValue, IntoView, LiveRegion, NodeRef, Role, View, for_each,
    node_ref, show,
};
use kivact::{Value, memo};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatDensity {
    Compact,
    #[default]
    Balanced,
    Spacious,
}

impl ChatDensity {
    fn gap(self) -> i32 {
        match self {
            Self::Compact => 8,
            Self::Balanced => 16,
            Self::Spacious => 24,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatSender {
    User,
    Assistant,
    System,
}

pub struct ChatMessage {
    root: Node,
    sender: ChatSender,
    content: View,
    avatar: Option<View>,
    metadata: Option<View>,
    name: Option<Value<String>>,
    density: ChatDensity,
}

pub fn chat_message(sender: ChatSender, content: impl IntoView) -> ChatMessage {
    ChatMessage {
        root: node(),
        sender,
        content: content.into_view(),
        avatar: None,
        metadata: None,
        name: None,
        density: ChatDensity::Balanced,
    }
}

impl NodeBuilder for ChatMessage {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatMessage {
    pub fn avatar<V: IntoView>(mut self, render: impl FnOnce() -> V) -> Self {
        self.avatar = Some(render().into_view());
        self
    }

    pub fn metadata<V: IntoView>(mut self, render: impl FnOnce() -> V) -> Self {
        self.metadata = Some(render().into_view());
        self
    }

    pub fn name(mut self, name: impl IntoText) -> Self {
        self.name = Some(name.into_text_value());
        self
    }

    pub fn density(mut self, density: ChatDensity) -> Self {
        self.density = density;
        self
    }
}

impl IntoView for ChatMessage {
    fn into_view(self) -> View {
        let Self {
            root,
            sender,
            content,
            avatar,
            metadata,
            name,
            density,
        } = self;
        let user = sender == ChatSender::User;
        let system = sender == ChatSender::System;
        let name = name.map(|name| {
            text(name).style(move || {
                style()
                    .font_size(12)
                    .line_height(20.0 / 12.0)
                    .weight(600)
                    .color(tokens().colors.text_secondary)
            })
        });
        let name_offset = if name.is_some() { 20 } else { 0 };
        let avatar = avatar.filter(|_| !system).map(|avatar| {
            node()
                .style(style().margin_top(name_offset).shrink(0))
                .child(avatar)
        });
        let content = column((name, content, metadata)).style(
            style()
                .min_width(0)
                .max_width(pct(90))
                .gap(match density {
                    ChatDensity::Compact => 2,
                    ChatDensity::Balanced => 4,
                    ChatDensity::Spacious => 6,
                })
                .align_items(if user {
                    Align::End
                } else if system {
                    Align::Center
                } else {
                    Align::Start
                }),
        );
        root.role(Role::Article)
            .base_style(
                style()
                    .direction(if user {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    })
                    .justify_content(if system {
                        Justify::Center
                    } else {
                        Justify::Start
                    })
                    .align_items(Align::Start)
                    .gap(match density {
                        ChatDensity::Compact => 6,
                        ChatDensity::Balanced => 8,
                        ChatDensity::Spacious => 12,
                    }),
            )
            .child((avatar, content))
            .into_view()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BubbleVariant {
    #[default]
    Filled,
    Ghost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BubbleGroup {
    First,
    Middle,
    Last,
}

pub struct ChatBubble {
    root: Node,
    content: View,
    variant: BubbleVariant,
    sender: ChatSender,
    density: ChatDensity,
    group: Option<BubbleGroup>,
    width: Option<Length>,
    name: Option<Value<String>>,
    metadata: Option<View>,
}

pub fn chat_bubble(content: impl IntoView) -> ChatBubble {
    ChatBubble {
        root: node(),
        content: content.into_view(),
        variant: BubbleVariant::Filled,
        sender: ChatSender::Assistant,
        density: ChatDensity::Balanced,
        group: None,
        width: None,
        name: None,
        metadata: None,
    }
}

impl NodeBuilder for ChatBubble {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatBubble {
    pub fn variant(mut self, variant: BubbleVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn sender(mut self, sender: ChatSender) -> Self {
        self.sender = sender;
        self
    }
    pub fn density(mut self, density: ChatDensity) -> Self {
        self.density = density;
        self
    }
    pub fn group(mut self, group: BubbleGroup) -> Self {
        self.group = Some(group);
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }
    pub fn name(mut self, name: impl IntoText) -> Self {
        self.name = Some(name.into_text_value());
        self
    }
    pub fn metadata<V: IntoView>(mut self, metadata: impl FnOnce() -> V) -> Self {
        self.metadata = Some(metadata().into_view());
        self
    }
}

impl IntoView for ChatBubble {
    fn into_view(self) -> View {
        let Self {
            root,
            content,
            variant,
            sender,
            density,
            group,
            width,
            name,
            metadata,
        } = self;
        let rtl = kivact::view::current_direction() == kivact::view::ReadingDirection::Rtl;
        let right = (sender == ChatSender::User) != rtl;
        let inline = if density == ChatDensity::Spacious {
            20
        } else {
            16
        };
        let bubble = node()
            .style(move || {
                let p = tokens();
                let mut bubble = style()
                    .direction(Direction::Column)
                    .min_width(0)
                    .padding_x(inline)
                    .radius(if density == ChatDensity::Compact {
                        p.radius.container
                    } else {
                        28.0
                    })
                    .font_size(14)
                    .line_height(20.0 / 14.0)
                    .color(p.colors.text_primary);
                if let Some(width) = width {
                    bubble = bubble.width(width);
                } else {
                    bubble = bubble.max_width(pct(100));
                }
                if variant == BubbleVariant::Filled {
                    bubble = bubble.background(p.colors.neutral).padding_y(
                        if density == ChatDensity::Spacious {
                            16
                        } else {
                            12
                        },
                    );
                }
                if matches!(group, Some(BubbleGroup::Middle | BubbleGroup::Last)) {
                    bubble = if right {
                        bubble.radius_top_right(6)
                    } else {
                        bubble.radius_top_left(6)
                    };
                }
                if matches!(group, Some(BubbleGroup::Middle | BubbleGroup::First)) {
                    bubble = if right {
                        bubble.radius_bottom_right(6)
                    } else {
                        bubble.radius_bottom_left(6)
                    };
                }
                bubble
            })
            .child(content);
        let name = name.map(|name| {
            text(name).style(move || {
                style()
                    .padding_x(inline)
                    .font_size(12)
                    .line_height(20.0 / 12.0)
                    .weight(600)
                    .color(tokens().colors.text_secondary)
            })
        });
        let metadata =
            metadata.map(|metadata| node().style(style().padding_x(inline)).child(metadata));
        root.base_style(
            style()
                .direction(Direction::Column)
                .min_width(0)
                .align_items(if sender == ChatSender::User {
                    Align::End
                } else {
                    Align::Start
                }),
        )
        .child((name, bubble, metadata))
        .into_view()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageStatus {
    Sending,
    Sent,
    Delivered,
    Read,
    Error,
}

pub struct ChatMetadata {
    root: Node,
    timestamp: Option<Value<String>>,
    footer: Option<View>,
    status: Option<Value<MessageStatus>>,
    sender: ChatSender,
}

pub fn chat_metadata() -> ChatMetadata {
    ChatMetadata {
        root: node(),
        timestamp: None,
        footer: None,
        status: None,
        sender: ChatSender::Assistant,
    }
}

impl NodeBuilder for ChatMetadata {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatMetadata {
    pub fn timestamp(mut self, value: impl IntoText) -> Self {
        self.timestamp = Some(value.into_text_value());
        self
    }
    pub fn footer<V: IntoView>(mut self, footer: impl FnOnce() -> V) -> Self {
        self.footer = Some(footer().into_view());
        self
    }
    pub fn status(mut self, status: impl IntoValue<MessageStatus>) -> Self {
        self.status = Some(status.into_value());
        self
    }
    pub fn sender(mut self, sender: ChatSender) -> Self {
        self.sender = sender;
        self
    }
}

impl IntoView for ChatMetadata {
    fn into_view(self) -> View {
        let Self {
            root,
            timestamp,
            footer,
            status,
            sender,
        } = self;
        let mut segments = Vec::new();
        if let Some(timestamp) = timestamp {
            segments.push(text(timestamp).into_view());
        }
        if let Some(footer) = footer {
            if !segments.is_empty() {
                segments.push(text("·").into_view());
            }
            segments.push(footer);
        }
        if let Some(status) = status {
            if !segments.is_empty() {
                segments.push(text("·").into_view());
            }
            segments.push(
                text(move || match status.get() {
                    MessageStatus::Sending => "◷ Sending",
                    MessageStatus::Sent => "✓ Sent",
                    MessageStatus::Delivered => "✓✓ Delivered",
                    MessageStatus::Read => "✓✓ Read",
                    MessageStatus::Error => "! Error",
                })
                .style(move || {
                    let p = tokens();
                    style().color(if status.get() == MessageStatus::Error {
                        p.colors.text_error
                    } else {
                        p.colors.text_secondary
                    })
                })
                .into_view(),
            );
        }
        root.base_style(move || {
            style()
                .direction(if sender == ChatSender::User {
                    Direction::RowReverse
                } else {
                    Direction::Row
                })
                .align_items(Align::Center)
                .gap(4)
                .margin_top(4)
                .font_size(12)
                .line_height(20.0 / 12.0)
                .color(tokens().colors.text_secondary)
        })
        .child(segments)
        .into_view()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SystemMessageVariant {
    #[default]
    Default,
    Divider,
}

pub struct ChatSystemMessage {
    root: Node,
    text: Value<String>,
    icon: Option<View>,
    variant: SystemMessageVariant,
}

pub fn chat_system_message(message: impl IntoText) -> ChatSystemMessage {
    ChatSystemMessage {
        root: node(),
        text: message.into_text_value(),
        icon: None,
        variant: SystemMessageVariant::Default,
    }
}

impl NodeBuilder for ChatSystemMessage {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatSystemMessage {
    pub fn variant(mut self, variant: SystemMessageVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
}

impl IntoView for ChatSystemMessage {
    fn into_view(self) -> View {
        let divider = self.variant == SystemMessageVariant::Divider;
        let line = || {
            divider.then(|| {
                node().style(move || style().grow(1).height(1).background(tokens().colors.border))
            })
        };
        self.root
            .base_style(move || {
                style()
                    .direction(Direction::Row)
                    .padding_y(4)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .gap(if divider { 12 } else { 6 })
                    .font_size(12)
                    .line_height(20.0 / 12.0)
                    .color(tokens().colors.text_secondary)
            })
            .child((line(), self.icon, text(self.text), line()))
            .into_view()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatListAlign {
    Top,
    #[default]
    Bottom,
}

pub struct ChatMessageList {
    root: Node,
    messages: View,
    empty: Option<Box<dyn Fn() -> View>>,
    is_empty: Value<bool>,
    loading_top: Value<bool>,
    streaming: Value<bool>,
    density: ChatDensity,
    gap: Option<f64>,
    align: ChatListAlign,
}

pub fn chat_message_list<T, K, V>(
    items: impl IntoValue<Vec<T>>,
    key: impl Fn(&T) -> K + 'static,
    render: impl Fn(Value<T>) -> V + 'static,
) -> ChatMessageList
where
    T: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + 'static,
    V: IntoView,
{
    let items = items.into_value();
    ChatMessageList {
        root: node(),
        messages: for_each(items, key, move |_, item| render(item)).into_view(),
        empty: None,
        is_empty: memo(move || items.with(Vec::is_empty)),
        loading_top: false.into_value(),
        streaming: false.into_value(),
        density: ChatDensity::Balanced,
        gap: None,
        align: ChatListAlign::Bottom,
    }
}

impl NodeBuilder for ChatMessageList {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatMessageList {
    pub fn empty<V: IntoView>(mut self, empty: impl Fn() -> V + 'static) -> Self {
        self.empty = Some(Box::new(move || empty().into_view()));
        self
    }
    pub fn loading_top(mut self, loading: impl IntoValue<bool>) -> Self {
        self.loading_top = loading.into_value();
        self
    }
    pub fn streaming(mut self, streaming: impl IntoValue<bool>) -> Self {
        self.streaming = streaming.into_value();
        self
    }
    pub fn density(mut self, density: ChatDensity) -> Self {
        self.density = density;
        self
    }
    pub fn gap(mut self, gap: f64) -> Self {
        self.gap = Some(gap.max(0.0));
        self
    }
    pub fn align(mut self, align: ChatListAlign) -> Self {
        self.align = align;
        self
    }
}

impl IntoView for ChatMessageList {
    fn into_view(self) -> View {
        let Self {
            root,
            messages,
            empty,
            is_empty,
            loading_top,
            streaming,
            density,
            gap,
            align,
        } = self;
        root.role(Role::Log)
            .live_region(LiveRegion::Polite)
            .busy(move || streaming.get() || loading_top.get())
            .base_style(
                style()
                    .direction(Direction::Column)
                    .grow(1)
                    .justify_content(if align == ChatListAlign::Bottom {
                        Justify::End
                    } else {
                        Justify::Start
                    })
                    .padding_y(density.gap())
                    .padding_x(density.gap().max(12))
                    .gap(gap.unwrap_or(density.gap() as f64)),
            )
            .child((
                show(loading_top, || {
                    text("Loading earlier messages…").role(Role::Status)
                }),
                messages,
                empty.map(|empty| show(is_empty, empty)),
                show(streaming, || text("Responding…").role(Role::Status)),
            ))
            .into_view()
    }
}

pub struct ChatLayout {
    root: Node,
    messages: View,
    composer: Option<View>,
    density: ChatDensity,
    scroll_button: bool,
    viewport: Option<NodeRef>,
    history_revision: Option<Value<u64>>,
}

pub fn chat_layout(messages: impl IntoView) -> ChatLayout {
    ChatLayout {
        root: node(),
        messages: messages.into_view(),
        composer: None,
        density: ChatDensity::Balanced,
        scroll_button: true,
        viewport: None,
        history_revision: None,
    }
}

impl NodeBuilder for ChatLayout {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatLayout {
    pub fn composer<V: IntoView>(mut self, composer: impl FnOnce() -> V) -> Self {
        self.composer = Some(composer().into_view());
        self
    }
    pub fn density(mut self, density: ChatDensity) -> Self {
        self.density = density;
        self
    }
    /// Share the viewport reference with application scrolling or observation.
    pub fn viewport_ref(mut self, viewport: NodeRef) -> Self {
        self.viewport = Some(viewport);
        self
    }

    /// Increment this revision in the same update that prepends earlier messages.
    /// The layout compensates for the inserted height to keep the current messages in view.
    pub fn history_revision(mut self, revision: impl IntoValue<u64>) -> Self {
        self.history_revision = Some(revision.into_value());
        self
    }

    pub fn scroll_button(mut self, show: bool) -> Self {
        self.scroll_button = show;
        self
    }
}

pub fn chat_scroll_button<R: HandlerResult>(
    visible: impl IntoValue<bool>,
    press: impl Fn() -> R + 'static,
) -> Node {
    let visible = visible.into_value();
    node()
        .part(
            part(|state| state)
                .role(Role::Button)
                .focusable(true)
                .name(message(
                    "soft.chat_layout_scroll_button.scroll_to_bottom",
                    "Scroll to bottom",
                ))
                .on_press(press),
        )
        .hidden(move || !visible.get())
        .base_style(move || {
            let p = tokens();
            style()
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .size(36, 36)
                .radius(18)
                .background(p.colors.bg_surface)
                .border(1, p.colors.border)
                .color(p.colors.text_primary)
                .hover(style().background(p.colors.accent_muted))
                .focus_visible(style().ring(p.sizes.focus_ring_width, p.colors.focus_ring))
        })
        .child(text("↓"))
}

impl IntoView for ChatLayout {
    fn into_view(self) -> View {
        let Self {
            root,
            messages,
            composer,
            density,
            scroll_button,
            viewport,
            history_revision,
        } = self;
        let scroller = viewport.unwrap_or_else(node_ref);
        let content = node_ref();
        const REATTACH: f64 = 10.0;
        const SHOW_BUTTON: f64 = 100.0;
        let follow = use_follow_end(scroller, content, REATTACH, history_revision);
        let distance = follow.distance();
        let can_jump = memo(move || distance.get() > SHOW_BUTTON);
        let scroll_view = scroll(
            column(messages)
                .node_ref(content)
                .style(style().grow(1).min_width(0)),
        )
        .node_ref(scroller)
        .style(style().grow(1).min_height(0).min_width(0));
        let button = scroll_button.then(|| {
            chat_scroll_button(can_jump, move || follow.jump_to_end())
                .style(style().position(Position::Absolute).right(12).bottom(12))
        });
        let area = column((scroll_view, button)).style(
            style()
                .position(Position::Relative)
                .grow(1)
                .min_height(0)
                .min_width(0),
        );
        let composer = composer.map(|composer| {
            row(composer).style(
                style()
                    .padding_x(density.gap())
                    .padding_bottom(density.gap())
                    .justify_content(Justify::Center),
            )
        });
        root.base_style(
            style()
                .direction(Direction::Column)
                .min_height(0)
                .min_width(0),
        )
        .child((area, composer))
        .into_view()
    }
}
