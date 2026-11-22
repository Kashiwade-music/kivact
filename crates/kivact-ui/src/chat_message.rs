//! Role: Assemble a named chat message with caller supplied content and metadata.
pub(crate) mod look;
pub use look::{
    ChatAlign, ChatBubbleGroup, ChatMessageStatus, ChatMessageVariant, ChatReactionsSide,
    ChatSender,
};
mod view;
pub use view::{ChatMessage, chat_message};
