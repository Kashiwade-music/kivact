//! Role: Define passive chat meanings and theme states.
use crate::Variant;
/// The participant role; the author name remains caller supplied text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatSender {
    User,
    #[default]
    Assistant,
    System,
}
/// Delivery state of a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatMessageStatus {
    Sending,
    Sent,
    Delivered,
    Read,
    Error,
}
/// A built-in chat appearance, interpreted by recipes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatMessageVariant {
    #[default]
    Bubble,
    Plain,
}
impl Variant for ChatMessageVariant {}
/// The edge at which chat content is aligned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatAlign {
    #[default]
    Start,
    End,
}
/// Joining position of a bubble in a group from the same participant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatBubbleGroup {
    #[default]
    Single,
    First,
    Middle,
    Last,
}
/// The edge a message's reaction slot overlaps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatReactionsSide {
    Top,
    #[default]
    Bottom,
}
/// A composer diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposerStatus {
    Error(String),
    Warning(String),
}
/// Placement of a composer diagnostic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComposerStatusPosition {
    Top,
    #[default]
    Bottom,
}
kivact_core::state::constant_value!(
    ChatSender,
    ChatBubbleGroup,
    ChatReactionsSide,
    ChatMessageStatus,
    ChatAlign,
    ComposerStatus,
    ComposerStatusPosition
);
// xtask-allow(state-fields): Read-only recipe facts projected from independent sender, layout, delivery and slot inputs.
/// Message facts passed to every message recipe.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessageState {
    pub sender: ChatSender,
    pub status: Option<ChatMessageStatus>,
    pub align: ChatAlign,
    pub group: ChatBubbleGroup,
    pub reactions_side: ChatReactionsSide,
    pub has_reactions: bool,
    pub actionable: bool,
}
/// Scroll-follow facts supplied by the shared follow behavior.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatThreadState {
    pub distance: f64,
}
// xtask-allow(state-fields): Read-only recipe facts projected from independent response and validation inputs.
/// Composer facts supplied to its recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatComposerState {
    pub stopped: bool,
    pub status: Option<ComposerStatus>,
}
/// The repeating typing phase supplied to recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct TypingIndicatorState {
    pub phase: f64,
}

// xtask-allow(state-fields): Read-only recipe projection of independent builder inputs, the shared disclosure owner and clipboard feedback owner.
/// Rendered source facts supplied to code-block recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeBlockState {
    pub numbered: bool,
    pub wrapped: bool,
    pub expanded: bool,
    pub copied: bool,
}
