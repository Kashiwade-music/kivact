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
/// Message facts passed to every message recipe.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessageState {
    pub sender: ChatSender,
    // xtask-allow(state-fields): passive record - status
    pub status: Option<ChatMessageStatus>,
    pub align: ChatAlign,
    pub group: ChatBubbleGroup,
    pub reactions_side: ChatReactionsSide,
    // xtask-allow(state-fields): passive record - has reactions
    pub has_reactions: bool,
    // xtask-allow(state-fields): passive record - actionable
    pub actionable: bool,
    // xtask-allow(state-fields): passive record - has header
    pub has_header: bool,
    // xtask-allow(state-fields): passive record - has footer
    pub has_footer: bool,
    // xtask-allow(state-fields): passive record - has avatar
    pub has_avatar: bool,
}
/// Scroll-follow facts supplied by the shared follow behavior.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatThreadState {
    pub distance: f64,
}
/// Composer facts supplied to its recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatComposerState {
    // xtask-allow(state-fields): passive record - stopped
    pub stopped: bool,
    // xtask-allow(state-fields): passive record - status
    pub status: Option<ComposerStatus>,
}
/// The repeating typing phase supplied to recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct TypingIndicatorState {
    pub phase: f64,
}
