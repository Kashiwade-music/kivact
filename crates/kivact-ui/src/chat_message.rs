//! Role: Assemble a named chat message with caller supplied content and metadata.

pub(crate) mod look;

pub use look::{
    ChatAlign, ChatBubbleGroup, ChatMessageStatus, ChatMessageVariant, ChatReactionsSide,
    ChatSender,
};

use crate::chat_message::look::ChatMessageState;
use crate::chat_message::look::ChatMessageTheme;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::Caller;
use kivact_core::state::Value;
use kivact_core::style::{Length, style};
use kivact_core::view::{Node, Variants, View, node, text};
use kivact_macros::component;

/// A message attributed to `author`, with independent content, author and
/// supporting slots.
///
/// `content` is the body. `time`, `status`, `header`, `footer`, `metadata`,
/// `media`, `icon`, `actions`, `reactions` and `avatar` fill their slots.
/// `sender` sets the participant role, `align` places the message, `group`
/// joins the body with neighboring messages from the same participant,
/// `reactions_side` puts the reactions above or below the body, and
/// `bubble_width` sets the inner bubble's width while the message row keeps
/// its own.
#[component]
pub fn chat_message(
    author: Value<String>,
    #[prop(default = ChatSender::Assistant)] sender: Value<ChatSender>,
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
    #[prop(default = ChatBubbleGroup::Single)] group: ChatBubbleGroup,
    #[prop(default = ChatReactionsSide::Bottom)] reactions_side: Value<ChatReactionsSide>,
    bubble_width: Option<Value<Length>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let actionable = caller.has_press();
    let has_avatar = avatar.is_some();
    let slots = Slots {
        author,
        time,
        status,
        content,
        header,
        footer,
        metadata,
        media,
        icon,
        actions,
        reactions,
    };
    let state = message_state(
        sender,
        align,
        group,
        reactions_side,
        &slots,
        has_avatar,
        actionable,
    );
    let body = message_body(slots, &variants, state);
    let body = match bubble_width {
        Some(width) => body.style(move || style().width(width.get())),
        None => body,
    };
    let mut root = node();
    if actionable {
        root = root.part(kivact_interaction::use_button().control());
    }
    let avatar = message_slot(
        avatar,
        |r: &ChatMessageTheme| r.avatar_recipe(),
        &variants,
        state,
    );
    with_base_recipe_style(
        root,
        |r: &ChatMessageTheme| r.root_recipe(),
        &variants,
        state,
    )
    .child((avatar, body))
}

/// The slots of a message, as its body gives them over.
struct Slots {
    author: Value<String>,
    time: Option<Value<String>>,
    status: Option<Value<ChatMessageStatus>>,
    content: Option<View>,
    header: Option<View>,
    footer: Option<View>,
    metadata: Option<View>,
    media: Option<View>,
    icon: Option<View>,
    actions: Option<View>,
    reactions: Option<View>,
}

/// The facts the recipes read: the role, the place and the slots of the message.
fn message_state(
    sender: Value<ChatSender>,
    align: Option<Value<ChatAlign>>,
    group: ChatBubbleGroup,
    reactions_side: Value<ChatReactionsSide>,
    slots: &Slots,
    has_avatar: bool,
    actionable: bool,
) -> impl Fn() -> ChatMessageState + Copy + 'static {
    let status = slots.status;
    let has_header = slots.header.is_some();
    let has_footer = slots.footer.is_some();
    let has_reactions = slots.reactions.is_some();
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
        reactions_side: reactions_side.get(),
        has_reactions,
        actionable,
        has_header,
        has_footer,
        has_avatar,
    }
}

/// The text, the slots and the bubble of a message, as its body.
fn message_body(
    slots: Slots,
    variants: &Variants,
    state: impl Fn() -> ChatMessageState + Copy + 'static,
) -> Node {
    let author = with_recipe_style(
        text(slots.author).hidden(move || slots.author.with(String::is_empty)),
        |r: &ChatMessageTheme| r.author_recipe(),
        variants,
        state,
    );
    let time = slots.time.map(|t| {
        with_recipe_style(
            text(t),
            |r: &ChatMessageTheme| r.time_recipe(),
            variants,
            state,
        )
    });
    let status = render_status(slots.status, variants, state);
    let header = message_slot(
        slots.header,
        |r: &ChatMessageTheme| r.header_recipe(),
        variants,
        state,
    );
    let media = message_slot(
        slots.media,
        |r: &ChatMessageTheme| r.media_recipe(),
        variants,
        state,
    );
    let icon = message_slot(
        slots.icon,
        |r: &ChatMessageTheme| r.icon_recipe(),
        variants,
        state,
    );
    let content = message_slot(
        slots.content,
        |r: &ChatMessageTheme| r.content_recipe(),
        variants,
        state,
    );
    let metadata = message_slot(
        slots.metadata,
        |r: &ChatMessageTheme| r.metadata_recipe(),
        variants,
        state,
    );
    let actions = message_slot(
        slots.actions,
        |r: &ChatMessageTheme| r.actions_recipe(),
        variants,
        state,
    );
    let footer = message_slot(
        slots.footer,
        |r: &ChatMessageTheme| r.footer_recipe(),
        variants,
        state,
    );
    let reactions = message_slot(
        slots.reactions,
        |r: &ChatMessageTheme| r.reactions_recipe(),
        variants,
        state,
    );
    with_recipe_style(
        node().child((
            author, header, media, icon, content, time, status, metadata, actions, footer,
            reactions,
        )),
        |r: &ChatMessageTheme| r.body_recipe(),
        variants,
        state,
    )
}

fn render_status(
    status: Option<Value<ChatMessageStatus>>,
    variants: &Variants,
    state: impl Fn() -> ChatMessageState + Copy + 'static,
) -> Option<Node> {
    let status = status?;
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
        variants,
        state,
    ))
}

fn message_slot(
    view: Option<View>,
    recipe: fn(&ChatMessageTheme) -> &crate::recipe::Recipe<ChatMessageState>,
    variants: &Variants,
    state: impl Fn() -> ChatMessageState + Copy + 'static,
) -> Option<Node> {
    view.map(|view| with_recipe_style(node().child(view), recipe, variants, state))
}
