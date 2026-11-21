//! Role: Build the chat message, its status and bubble slots and its layout.

use crate::chat_message::look::ChatMessageState;
use crate::chat_message::look::ChatMessageTheme;
use crate::chat_message::look::{
    ChatAlign, ChatBubbleGroup, ChatMessageStatus, ChatReactionsSide, ChatSender,
};
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::view::Variants;
use kivact_core::view::{Element, IntoView, Node, View, node, text};

// xtask-allow(state): legacy: status, time, content, avatar, header, footer, metadata, media, icon, actions, reactions, align, bubble_width
/// A message with independent content, author and supporting slots.
pub struct ChatMessage {
    root: Node,
    author: Value<String>,
    sender: Value<ChatSender>,
    status: Option<Value<ChatMessageStatus>>,
    time: Option<Value<String>>,
    content: Option<View>,
    avatar: Option<View>,
    header: Option<View>,
    footer: Option<View>,
    metadata: Option<View>,
    media: Option<View>,
    icon: Option<View>,
    actions: Option<View>,
    reactions: Option<View>,
    align: Option<Value<ChatAlign>>,
    group: ChatBubbleGroup,
    reactions_side: Value<ChatReactionsSide>,
    bubble_width: Option<Value<kivact_core::style::Length>>,
}
/// A message attributed to `author`; put its body in `.content(..)`.
#[track_caller]
pub fn chat_message(author: impl IntoText) -> ChatMessage {
    ChatMessage {
        root: node(),
        author: author.into_text_value(),
        sender: ChatSender::Assistant.into_value(),
        status: None,
        time: None,
        content: None,
        avatar: None,
        header: None,
        footer: None,
        metadata: None,
        media: None,
        icon: None,
        actions: None,
        reactions: None,
        align: None,
        group: ChatBubbleGroup::Single,
        reactions_side: ChatReactionsSide::Bottom.into_value(),
        bubble_width: None,
    }
}
impl ChatMessage {
    /// Align the message independently of participant role.
    #[track_caller]
    pub fn align(mut self, align: impl IntoValue<ChatAlign>) -> Self {
        self.align = Some(align.into_value());
        self
    }
    /// Join the body with neighboring messages from the same participant.
    pub fn group(mut self, group: ChatBubbleGroup) -> Self {
        self.group = group;
        self
    }
    /// Place the reaction slot above or below the body.
    #[track_caller]
    pub fn reactions_side(mut self, side: impl IntoValue<ChatReactionsSide>) -> Self {
        self.reactions_side = side.into_value();
        self
    }
    /// Set the width of the inner bubble while preserving the message row's width.
    #[track_caller]
    pub fn bubble_width(mut self, width: impl IntoValue<kivact_core::style::Length>) -> Self {
        self.bubble_width = Some(width.into_value());
        self
    }
    /// Set the participant role independently of their displayed name.
    #[track_caller]
    pub fn sender(mut self, sender: impl IntoValue<ChatSender>) -> Self {
        self.sender = sender.into_value();
        self
    }
    /// Set the displayed timestamp.
    #[track_caller]
    pub fn time(mut self, time: impl IntoText) -> Self {
        self.time = Some(time.into_text_value());
        self
    }
    /// Set delivery information.
    #[track_caller]
    pub fn status(mut self, status: impl IntoValue<ChatMessageStatus>) -> Self {
        self.status = Some(status.into_value());
        self
    }
    /// Fill the content slot.
    pub fn content(mut self, content: impl IntoView) -> Self {
        self.content = Some(content.into_view());
        self
    }
    /// Fill the avatar slot.
    pub fn avatar(mut self, content: impl IntoView) -> Self {
        self.avatar = Some(content.into_view());
        self
    }
    /// Fill the header slot.
    pub fn header(mut self, content: impl IntoView) -> Self {
        self.header = Some(content.into_view());
        self
    }
    /// Fill the footer slot.
    pub fn footer(mut self, content: impl IntoView) -> Self {
        self.footer = Some(content.into_view());
        self
    }
    /// Fill the metadata slot.
    pub fn metadata(mut self, content: impl IntoView) -> Self {
        self.metadata = Some(content.into_view());
        self
    }
    /// Fill the media slot.
    pub fn media(mut self, content: impl IntoView) -> Self {
        self.media = Some(content.into_view());
        self
    }
    /// Fill the icon slot.
    pub fn icon(mut self, content: impl IntoView) -> Self {
        self.icon = Some(content.into_view());
        self
    }
    /// Fill the actions slot.
    pub fn actions(mut self, content: impl IntoView) -> Self {
        self.actions = Some(content.into_view());
        self
    }
    /// Fill the reactions slot.
    pub fn reactions(mut self, content: impl IntoView) -> Self {
        self.reactions = Some(content.into_view());
        self
    }
}
impl Element for ChatMessage {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl ChatMessage {
    fn recipe_state(&self) -> impl Fn() -> ChatMessageState + Copy + 'static {
        let sender = self.sender;
        let status = self.status;
        let align = self.align;
        let group = self.group;
        let side = self.reactions_side;
        let has_reactions = self.reactions.is_some();
        let actionable = self.root.authored().has_press();
        let has_header = self.header.is_some();
        let has_footer = self.footer.is_some();
        let has_avatar = self.avatar.is_some();
        move || ChatMessageState {
            sender: sender.get(),
            status: status.map(|s| s.get()),
            align: align.map(|v| v.get()).unwrap_or_else(|| {
                if sender.get() == ChatSender::User {
                    ChatAlign::End
                } else {
                    ChatAlign::Start
                }
            }),
            group,
            reactions_side: side.get(),
            has_reactions,
            actionable,
            has_header,
            has_footer,
            has_avatar,
        }
    }
    fn render_status(&self, state: impl Fn() -> ChatMessageState + Copy + 'static) -> Option<Node> {
        let variants = self.root.authored().variants();
        let status = self.status?;
        let sending = kivact_core::i18n::message("chat_message.sending", "Sending");
        let sent = kivact_core::i18n::message("chat_message.sent", "Sent");
        let delivered = kivact_core::i18n::message("chat_message.delivered", "Delivered");
        let read = kivact_core::i18n::message("chat_message.read", "Read");
        let failed = kivact_core::i18n::message("chat_message.error", "Failed");
        Some(with_recipe_style(
            text(move || match status.get() {
                ChatMessageStatus::Sending => sending.get(),
                ChatMessageStatus::Sent => sent.get(),
                ChatMessageStatus::Delivered => delivered.get(),
                ChatMessageStatus::Read => read.get(),
                ChatMessageStatus::Error => failed.get(),
            }),
            |r: &ChatMessageTheme| r.status_recipe(),
            &variants,
            state,
        ))
    }
    fn render_body(&mut self, state: impl Fn() -> ChatMessageState + Copy + 'static) -> Node {
        let variants = self.root.authored().variants();
        let variants = &variants;
        let author = with_recipe_style(
            text(self.author).hidden({
                let author = self.author;
                move || author.with(String::is_empty)
            }),
            |r: &ChatMessageTheme| r.author_recipe(),
            variants,
            state,
        );
        let time = self.time.map(|t| {
            with_recipe_style(
                text(t),
                |r: &ChatMessageTheme| r.time_recipe(),
                variants,
                state,
            )
        });
        let status = self.render_status(state);
        let content = message_slot(
            self.content.take(),
            |r: &ChatMessageTheme| r.content_recipe(),
            variants,
            state,
        );
        let header = message_slot(
            self.header.take(),
            |r: &ChatMessageTheme| r.header_recipe(),
            variants,
            state,
        );
        let footer = message_slot(
            self.footer.take(),
            |r: &ChatMessageTheme| r.footer_recipe(),
            variants,
            state,
        );
        let metadata = message_slot(
            self.metadata.take(),
            |r: &ChatMessageTheme| r.metadata_recipe(),
            variants,
            state,
        );
        let media = message_slot(
            self.media.take(),
            |r: &ChatMessageTheme| r.media_recipe(),
            variants,
            state,
        );
        let icon = message_slot(
            self.icon.take(),
            |r: &ChatMessageTheme| r.icon_recipe(),
            variants,
            state,
        );
        let actions = message_slot(
            self.actions.take(),
            |r: &ChatMessageTheme| r.actions_recipe(),
            variants,
            state,
        );
        let reactions = message_slot(
            self.reactions.take(),
            |r: &ChatMessageTheme| r.reactions_recipe(),
            variants,
            state,
        );

        let body = with_recipe_style(
            node().child((
                author, header, media, icon, content, time, status, metadata, actions, footer,
                reactions,
            )),
            |r: &ChatMessageTheme| r.body_recipe(),
            variants,
            state,
        );
        self.configure_body(body)
    }
    fn configure_body(&self, mut body: Node) -> Node {
        if let Some(width) = self.bubble_width {
            body = body.style(move || kivact_core::style::style().width(width.get()));
        }
        body
    }
}
fn message_slot(
    view: Option<View>,
    recipe: fn(&ChatMessageTheme) -> &crate::recipe::Recipe<ChatMessageState>,
    variants: &Variants,
    state: impl Fn() -> ChatMessageState + Copy + 'static,
) -> Option<Node> {
    view.map(|view| with_recipe_style(node().child(view), recipe, variants, state))
}
impl IntoView for ChatMessage {
    fn into_view(mut self) -> View {
        let variants = self.root.authored().variants();
        let state = self.recipe_state();
        let body = self.render_body(state);
        let avatar = message_slot(
            self.avatar.take(),
            |r: &ChatMessageTheme| r.avatar_recipe(),
            &variants,
            state,
        );
        let mut root = self.root;
        if root.authored().has_press() {
            root = root.part(kivact_interaction::use_button().control());
        }
        with_base_recipe_style(
            root,
            |r: &ChatMessageTheme| r.root_recipe(),
            &variants,
            state,
        )
        .child((avatar, body))
        .into_view()
    }
}
