//! Role: Assemble a named chat message with caller supplied content and metadata.
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::variants::{Variant, Variants};
use crate::{ChatMessageState, ChatMessageStatus, ChatSender};
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::view::{IntoView, Node, NodeBuilder, View, node, text};
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
        variants: Variants::default(),
    }
}
impl ChatMessage {
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
        let state = move || ChatMessageState {
            sender: sender.get(),
            status: status.map(|s| s.get()),
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
        let status = self.status.map(|s| {
            with_recipe_style(
                text(move || match s.get() {
                    ChatMessageStatus::Sending => {
                        kivact_core::i18n::message("chat_message.sending", "Sending").get()
                    }
                    ChatMessageStatus::Sent => {
                        kivact_core::i18n::message("chat_message.sent", "Sent").get()
                    }
                    ChatMessageStatus::Delivered => {
                        kivact_core::i18n::message("chat_message.delivered", "Delivered").get()
                    }
                    ChatMessageStatus::Read => {
                        kivact_core::i18n::message("chat_message.read", "Read").get()
                    }
                    ChatMessageStatus::Error => {
                        kivact_core::i18n::message("chat_message.error", "Failed").get()
                    }
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
