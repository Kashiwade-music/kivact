//! Role: Assemble a named chat message with caller supplied content and metadata.
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::variants::{Variant, Variants};
use crate::{
    ChatAlign, ChatBubbleGroup, ChatMessageState, ChatMessageStatus, ChatReactionsSide, ChatSender,
};
use kivact_core::state::{Callback, IntoText, IntoValue, Value};
use kivact_core::view::{IntoView, Node, NodeBuilder, View, node, text};
// xtask-allow(state-fields): Independent builder inputs and view slots; this builder owns no persistent interaction state.
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
    on_press: Option<Callback<()>>,
    variants: Variants,
}
/// A message attributed to `author`; put its body in `.content(..)`.
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
        on_press: None,
        variants: Variants::default(),
    }
}
impl ChatMessage {
    /// Align the message independently of participant role.
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
    pub fn reactions_side(mut self, side: impl IntoValue<ChatReactionsSide>) -> Self {
        self.reactions_side = side.into_value();
        self
    }
    /// Set the width of the inner bubble while preserving the message row's width.
    pub fn bubble_width(mut self, width: impl IntoValue<kivact_core::style::Length>) -> Self {
        self.bubble_width = Some(width.into_value());
        self
    }
    /// Make the body actionable using shared button keyboard and accessibility semantics.
    pub fn on_press(mut self, press: impl Into<Callback<()>>) -> Self {
        self.on_press = Some(press.into());
        self
    }
    /// Set the participant role independently of their displayed name.
    pub fn sender(mut self, sender: impl IntoValue<ChatSender>) -> Self {
        self.sender = sender.into_value();
        self
    }
    /// Set the displayed timestamp.
    pub fn time(mut self, time: impl IntoText) -> Self {
        self.time = Some(time.into_text_value());
        self
    }
    /// Set delivery information.
    pub fn status(mut self, status: impl IntoValue<ChatMessageStatus>) -> Self {
        self.status = Some(status.into_value());
        self
    }
    /// Add an appearance read by the theme recipes.
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
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
impl NodeBuilder for ChatMessage {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ChatMessage {
    fn into_view(self) -> View {
        let sender = self.sender;
        let status = self.status;
        let align = self.align;
        let group = self.group;
        let side = self.reactions_side;
        let has_reactions = self.reactions.is_some();
        let actionable = self.on_press.is_some();
        let state = move || ChatMessageState {
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
        };
        let variants = &self.variants;
        let author = with_recipe_style(
            text(self.author),
            |r| r.chat_message_recipes().author_recipe(),
            variants,
            state,
        );
        let time = self.time.map(|t| {
            with_recipe_style(
                text(t),
                |r| r.chat_message_recipes().time_recipe(),
                variants,
                state,
            )
        });
        let sending = kivact_core::i18n::message("chat_message.sending", "Sending");
        let sent = kivact_core::i18n::message("chat_message.sent", "Sent");
        let delivered = kivact_core::i18n::message("chat_message.delivered", "Delivered");
        let read = kivact_core::i18n::message("chat_message.read", "Read");
        let failed = kivact_core::i18n::message("chat_message.error", "Failed");
        let status = self.status.map(|s| {
            with_recipe_style(
                text(move || match s.get() {
                    ChatMessageStatus::Sending => sending.get(),
                    ChatMessageStatus::Sent => sent.get(),
                    ChatMessageStatus::Delivered => delivered.get(),
                    ChatMessageStatus::Read => read.get(),
                    ChatMessageStatus::Error => failed.get(),
                }),
                |r| r.chat_message_recipes().status_recipe(),
                variants,
                state,
            )
        });
        let content = self.content.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.chat_message_recipes().content_recipe(),
                variants,
                state,
            )
        });
        let avatar = self.avatar.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.chat_message_recipes().avatar_recipe(),
                variants,
                state,
            )
        });
        let header = self.header.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.chat_message_recipes().header_recipe(),
                variants,
                state,
            )
        });
        let footer = self.footer.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.chat_message_recipes().footer_recipe(),
                variants,
                state,
            )
        });
        let metadata = self.metadata.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.chat_message_recipes().metadata_recipe(),
                variants,
                state,
            )
        });
        let media = self.media.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.chat_message_recipes().media_recipe(),
                variants,
                state,
            )
        });
        let icon = self.icon.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.chat_message_recipes().icon_recipe(),
                variants,
                state,
            )
        });
        let actions = self.actions.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.chat_message_recipes().actions_recipe(),
                variants,
                state,
            )
        });
        let reactions = self.reactions.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.chat_message_recipes().reactions_recipe(),
                variants,
                state,
            )
        });

        let body = with_recipe_style(
            node().child((
                author, header, media, icon, content, time, status, metadata, actions, footer,
                reactions,
            )),
            |r| r.chat_message_recipes().body_recipe(),
            variants,
            state,
        );
        let mut body = body;
        if let Some(press) = self.on_press {
            body = body
                .part(kivact_interaction::use_button().control())
                .on_press(move || press.call(()));
        }
        if let Some(width) = self.bubble_width {
            body = body.style(move || kivact_core::style::style().width(width.get()));
        }
        with_base_recipe_style(
            self.root,
            |r| r.chat_message_recipes().root_recipe(),
            variants,
            state,
        )
        .child((avatar, body))
        .into_view()
    }
}
