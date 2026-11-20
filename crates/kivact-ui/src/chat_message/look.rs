//! Role: The chat message recipe state, tokens, recipe table and default looks.
use crate::look_common::{column, row};

use crate::recipe::recipe_table;
use crate::variants::Look;
use crate::variants::Variant;
use kivact_core::style::{Align, ColorToken, Justify, Style, Theme, pct, style};
use kivact_macros::Prop;
pub(crate) fn author(t: &Theme, _: &Look<ChatMessageState>) -> Style {
    style()
        .font_size(t.text().size)
        .weight(t.text().weight_strong)
        .color(ColorToken::Text)
}

pub(crate) fn message_root(t: &Theme, s: &Look<ChatMessageState>) -> Style {
    row(t, s)
        .align_items(Align::Start)
        .justify_content(if s.align == ChatAlign::End {
            Justify::End
        } else {
            Justify::Start
        })
}

pub(crate) fn message_body(t: &Theme, s: &Look<ChatMessageState>) -> Style {
    let base = column(t, s)
        .position(kivact_core::style::Position::Relative)
        .max_width(pct(85))
        .color(ColorToken::Text);
    let base = if s.has_reactions {
        match s.reactions_side {
            ChatReactionsSide::Top => base.margin_top(18),
            ChatReactionsSide::Bottom => base.margin_bottom(18),
        }
    } else {
        base
    };
    let base = if s.actionable {
        base.cursor(kivact_core::style::Cursor::Pointer)
            .focus_visible(style().ring(t.focus_ring(), ColorToken::Accent))
    } else {
        base
    };
    if s.variant::<ChatMessageVariant>() == Some(&ChatMessageVariant::Plain) {
        base
    } else {
        base.padding(t.space().md)
            .radius_corners(
                if matches!(s.group, ChatBubbleGroup::Middle | ChatBubbleGroup::Last) {
                    0.0
                } else {
                    t.radius().md
                },
                if matches!(s.group, ChatBubbleGroup::Middle | ChatBubbleGroup::Last) {
                    0.0
                } else {
                    t.radius().md
                },
                if matches!(s.group, ChatBubbleGroup::Middle | ChatBubbleGroup::First) {
                    0.0
                } else {
                    t.radius().md
                },
                if matches!(s.group, ChatBubbleGroup::Middle | ChatBubbleGroup::First) {
                    0.0
                } else {
                    t.radius().md
                },
            )
            .background(ColorToken::Surface)
    }
}

pub(crate) fn reactions(_: &Theme, s: &Look<ChatMessageState>) -> Style {
    let base = style()
        .position(kivact_core::style::Position::Absolute)
        .right(12)
        .background(ColorToken::SurfaceAlt)
        .radius(999)
        .padding_x(6)
        .padding_y(2);
    match s.reactions_side {
        ChatReactionsSide::Top => base.top(0).translate(0, pct(-75)),
        ChatReactionsSide::Bottom => base.bottom(0).translate(0, pct(75)),
    }
}

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
    pub has_header: bool,
    pub has_footer: bool,
    pub has_avatar: bool,
}

recipe_table! {
    /// Styles for the persistent parts of ChatMessage.
    ChatMessageRecipes {
        root(root_recipe): ChatMessageState = message_root,
        body(body_recipe): ChatMessageState = message_body,
        author(author_recipe): ChatMessageState = author,
        time(time_recipe): ChatMessageState = crate::look_common::muted,
        status(status_recipe): ChatMessageState = crate::look_common::muted,
        content(content_recipe): ChatMessageState = crate::look_common::slot,
        avatar(avatar_recipe): ChatMessageState = crate::look_common::slot,
        header(header_recipe): ChatMessageState = crate::look_common::slot,
        footer(footer_recipe): ChatMessageState = crate::look_common::slot,
        metadata(metadata_recipe): ChatMessageState = crate::look_common::slot,
        media(media_recipe): ChatMessageState = crate::look_common::slot,
        icon(icon_recipe): ChatMessageState = crate::look_common::slot,
        actions(actions_recipe): ChatMessageState = crate::look_common::slot,
        reactions(reactions_recipe): ChatMessageState = reactions,
    }
}

/// The participant role; the author name remains caller supplied text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ChatSender {
    User,
    #[default]
    Assistant,
    System,
}

/// Delivery state of a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ChatAlign {
    #[default]
    Start,
    End,
}

/// Joining position of a bubble in a group from the same participant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ChatBubbleGroup {
    #[default]
    Single,
    First,
    Middle,
    Last,
}

/// The edge a message's reaction slot overlaps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ChatReactionsSide {
    Top,
    #[default]
    Bottom,
}
