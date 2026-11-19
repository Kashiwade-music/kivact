//! Role: Assemble a named chat message with caller supplied content and metadata.

pub(crate) mod look;
mod view;

pub use look::{
    ChatAlign, ChatBubbleGroup, ChatMessageStatus, ChatMessageVariant, ChatReactionsSide,
    ChatSender,
};
pub use view::{ChatMessage, chat_message};
