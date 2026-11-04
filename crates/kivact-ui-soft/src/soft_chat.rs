//! Role: Neutral chat messages, lists, and scroll layout.
use kivact::state::Callback;
use kivact::view::IntoViewFn;
use kivact::view::ViewFn;

use kivact::state::ItemKey;
use kivact::state::PropValue;

use crate::tokens;
use kivact::i18n::message;
use kivact::interaction::use_follow_end;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::memo;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Length;
use kivact::style::Position;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::LiveRegion;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::NodeRef;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::for_each;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::scroll;
use kivact::view::show;
use kivact::view::text;

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
    density: Value<ChatDensity>,
}

pub fn chat_message(sender: ChatSender, content: impl IntoView) -> ChatMessage {
    ChatMessage {
        root: node(),
        sender,
        content: content.into_view(),
        avatar: None,
        metadata: None,
        name: None,
        density: (ChatDensity::Balanced).into_value(),
    }
}

impl NodeBuilder for ChatMessage {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatMessage {
    pub fn avatar(mut self, render: impl IntoView) -> Self {
        self.avatar = Some(render.into_view());
        self
    }

    pub fn metadata(mut self, render: impl IntoView) -> Self {
        self.metadata = Some(render.into_view());
        self
    }

    pub fn name(mut self, name: impl IntoText) -> Self {
        self.name = Some(name.into_text_value());
        self
    }

    pub fn density(mut self, density: impl IntoValue<ChatDensity>) -> Self {
        self.density = density.into_value();
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
                .gap(match density.get() {
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
        root.with_node(move |node| node.role(Role::Article))
            .with_node(move |node| {
                node.base_style(
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
                        .gap(match density.get() {
                            ChatDensity::Compact => 6,
                            ChatDensity::Balanced => 8,
                            ChatDensity::Spacious => 12,
                        }),
                )
            })
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
    variant: Value<BubbleVariant>,
    sender: Value<ChatSender>,
    density: Value<ChatDensity>,
    group: Option<BubbleGroup>,
    width: Option<Value<Length>>,
    name: Option<Value<String>>,
    metadata: Option<View>,
}

pub fn chat_bubble(content: impl IntoView) -> ChatBubble {
    ChatBubble {
        root: node(),
        content: content.into_view(),
        variant: (BubbleVariant::Filled).into_value(),
        sender: (ChatSender::Assistant).into_value(),
        density: (ChatDensity::Balanced).into_value(),
        group: None,
        width: None,
        name: None,
        metadata: None,
    }
}

impl NodeBuilder for ChatBubble {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatBubble {
    pub fn variant(mut self, variant: impl IntoValue<BubbleVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }
    pub fn sender(mut self, sender: impl IntoValue<ChatSender>) -> Self {
        self.sender = sender.into_value();
        self
    }
    pub fn density(mut self, density: impl IntoValue<ChatDensity>) -> Self {
        self.density = density.into_value();
        self
    }
    pub fn group(mut self, group: BubbleGroup) -> Self {
        self.group = Some(group);
        self
    }
    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        self.width = Some(width.into_value());
        self
    }
    pub fn name(mut self, name: impl IntoText) -> Self {
        self.name = Some(name.into_text_value());
        self
    }
    pub fn metadata(mut self, metadata: impl IntoView) -> Self {
        self.metadata = Some(metadata.into_view());
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
        let rtl = kivact::view::use_direction().get() == kivact::view::ReadingDirection::Rtl;
        let right = (sender.get() == ChatSender::User) != rtl;
        let inline = if density.get() == ChatDensity::Spacious {
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
                    .radius(if density.get() == ChatDensity::Compact {
                        p.radius.container
                    } else {
                        28.0
                    })
                    .font_size(14)
                    .line_height(20.0 / 14.0)
                    .color(p.colors.text_primary);
                if let Some(width) = width {
                    bubble = bubble.width(width.get());
                } else {
                    bubble = bubble.max_width(pct(100));
                }
                if variant.get() == BubbleVariant::Filled {
                    bubble = bubble.background(p.colors.neutral).padding_y(
                        if density.get() == ChatDensity::Spacious {
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
        root.with_node(move |node| {
            node.base_style(
                style()
                    .direction(Direction::Column)
                    .min_width(0)
                    .align_items(if sender.get() == ChatSender::User {
                        Align::End
                    } else {
                        Align::Start
                    }),
            )
        })
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
    sender: Value<ChatSender>,
}

pub fn chat_metadata() -> ChatMetadata {
    ChatMetadata {
        root: node(),
        timestamp: None,
        footer: None,
        status: None,
        sender: (ChatSender::Assistant).into_value(),
    }
}

impl NodeBuilder for ChatMetadata {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatMetadata {
    pub fn timestamp(mut self, value: impl IntoText) -> Self {
        self.timestamp = Some(value.into_text_value());
        self
    }
    pub fn footer(mut self, footer: impl IntoView) -> Self {
        self.footer = Some(footer.into_view());
        self
    }
    pub fn status(mut self, status: impl IntoValue<MessageStatus>) -> Self {
        self.status = Some(status.into_value());
        self
    }
    pub fn sender(mut self, sender: impl IntoValue<ChatSender>) -> Self {
        self.sender = sender.into_value();
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
        root.with_node(move |node| {
            node.base_style(move || {
                style()
                    .direction(if sender.get() == ChatSender::User {
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
    variant: Value<SystemMessageVariant>,
}

pub fn chat_system_message(message: impl IntoText) -> ChatSystemMessage {
    ChatSystemMessage {
        root: node(),
        text: message.into_text_value(),
        icon: None,
        variant: (SystemMessageVariant::Default).into_value(),
    }
}

impl NodeBuilder for ChatSystemMessage {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatSystemMessage {
    pub fn variant(mut self, variant: impl IntoValue<SystemMessageVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
}

impl IntoView for ChatSystemMessage {
    fn into_view(self) -> View {
        let divider = self.variant.get() == SystemMessageVariant::Divider;
        let line = || {
            divider.then(|| {
                node().style(move || style().grow(1).height(1).background(tokens().colors.border))
            })
        };
        self.root
            .with_node(move |node| {
                node.base_style(move || {
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
    empty: Option<ViewFn>,
    is_empty: Value<bool>,
    loading_top: Value<bool>,
    streaming: Value<bool>,
    density: Value<ChatDensity>,
    gap: Option<Value<f64>>,
    align: Value<ChatListAlign>,
}

pub fn chat_message_list<T, K, V>(
    items: impl IntoValue<Vec<T>>,
    key: impl Fn(&T) -> K + 'static,
    render: impl Fn(Value<T>) -> V + 'static,
) -> ChatMessageList
where
    T: PropValue,
    K: ItemKey,
    V: IntoView,
{
    let items = items.into_value();
    ChatMessageList {
        root: node(),
        messages: for_each(items, key, move |item| render(item)).into_view(),
        empty: None,
        is_empty: memo(move || items.with(Vec::is_empty)),
        loading_top: false.into_value(),
        streaming: false.into_value(),
        density: (ChatDensity::Balanced).into_value(),
        gap: None,
        align: (ChatListAlign::Bottom).into_value(),
    }
}

impl NodeBuilder for ChatMessageList {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatMessageList {
    pub fn empty<V: IntoView>(mut self, empty: impl IntoViewFn<V>) -> Self {
        self.empty = Some(empty.into_view_fn());
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
    pub fn density(mut self, density: impl IntoValue<ChatDensity>) -> Self {
        self.density = density.into_value();
        self
    }
    pub fn gap(mut self, gap: impl IntoValue<f64>) -> Self {
        self.gap = Some(gap.into_value());
        self
    }
    pub fn align(mut self, align: impl IntoValue<ChatListAlign>) -> Self {
        self.align = align.into_value();
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
        root.with_node(move |node| node.role(Role::Log))
            .live_region(LiveRegion::Polite)
            .busy(move || streaming.get() || loading_top.get())
            .with_node(move |node| {
                node.base_style(move || {
                    style()
                        .direction(Direction::Column)
                        .grow(1)
                        .justify_content(if align.get() == ChatListAlign::Bottom {
                            Justify::End
                        } else {
                            Justify::Start
                        })
                        .padding_y(density.get().gap())
                        .padding_x(density.get().gap().max(12))
                        .gap(
                            gap.map_or_else(
                                || density.get().gap() as f64,
                                |gap| gap.get().max(0.0),
                            ),
                        )
                })
            })
            .child((
                show(loading_top, || {
                    text("Loading earlier messages…").with_node(move |node| node.role(Role::Status))
                }),
                messages,
                empty.map(|empty| show(is_empty, move || empty.call(()))),
                show(streaming, || {
                    text("Responding…").with_node(move |node| node.role(Role::Status))
                }),
            ))
            .into_view()
    }
}

pub struct ChatLayout {
    root: Node,
    messages: View,
    composer: Option<View>,
    density: Value<ChatDensity>,
    scroll_button: Value<bool>,
    viewport: Option<NodeRef>,
    history_revision: Option<Value<u64>>,
}

pub fn chat_layout(messages: impl IntoView) -> ChatLayout {
    ChatLayout {
        root: node(),
        messages: messages.into_view(),
        composer: None,
        density: (ChatDensity::Balanced).into_value(),
        scroll_button: (true).into_value(),
        viewport: None,
        history_revision: None,
    }
}

impl NodeBuilder for ChatLayout {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatLayout {
    pub fn composer(mut self, composer: impl IntoView) -> Self {
        self.composer = Some(composer.into_view());
        self
    }
    pub fn density(mut self, density: impl IntoValue<ChatDensity>) -> Self {
        self.density = density.into_value();
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

    pub fn scroll_button(mut self, show: impl IntoValue<bool>) -> Self {
        self.scroll_button = show.into_value();
        self
    }
}

pub fn chat_scroll_button(
    visible: impl IntoValue<bool>,
    press: impl Into<Callback<()>>,
) -> impl kivact::view::Component {
    let press: Callback<()> = press.into();
    let visible = visible.into_value();
    node()
        .with_node(move |node| node.role(Role::Button))
        .focusable(true)
        .name(message(
            "soft.chat_layout_scroll_button.scroll_to_bottom",
            "Scroll to bottom",
        ))
        .on_press(press)
        .hidden(move || !visible.get())
        .with_node(move |node| {
            node.base_style(move || {
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
        let button = scroll_button.get().then(|| {
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
                    .padding_x(density.get().gap())
                    .padding_bottom(density.get().gap())
                    .justify_content(Justify::Center),
            )
        });
        root.with_node(move |node| {
            node.base_style(
                style()
                    .direction(Direction::Column)
                    .min_height(0)
                    .min_width(0),
            )
        })
        .child((area, composer))
        .into_view()
    }
}

kivact::state::constant_value!(
    BubbleVariant,
    ChatDensity,
    ChatListAlign,
    ChatSender,
    MessageStatus,
    SystemMessageVariant
);
