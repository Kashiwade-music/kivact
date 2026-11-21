//! Role: Lyra recipes of the chat: the message, its bubble colors, and the
//! thread and its scroll-to-end button.
//!
//! The looks of the chat parts of the kit: a bubble of each variant (the
//! standalone chat bubbles and the message's body), the message's bands and
//! reactions, and the thread's viewport and jump button.

use crate::color::{
    BACKGROUND, BORDER, BUBBLE_TINTED, BUBBLE_TINTED_HOVER, CARD, DESTRUCTIVE, FOREGROUND, INPUT,
    MUTED, MUTED_FOREGROUND, PRIMARY, PRIMARY_FOREGROUND, SECONDARY, SECONDARY_FOREGROUND,
};
use crate::recipe_parts::focus_ring;
use crate::variants::{ChatBubbleVariant, ChatLayout};
use kivact::style::{
    Align, Color, FlexDirection, Justify, Overflow, Position, Style, Theme, ThemeMode, pct, style,
};
use kivact::ui::theme::{
    ChatMessageState, ChatMessageTheme, ChatThreadState, ChatThreadTheme, Look,
};
use kivact::ui::{ChatAlign, ChatReactionsSide};

/// Whether the message is a standalone bubble rather than a row of a thread.
fn standalone(s: &Look<ChatMessageState>) -> bool {
    s.variant::<ChatLayout>() == Some(ChatLayout::Bubble)
}

/// The side the message's content sits on.
fn edge(s: &Look<ChatMessageState>) -> Align {
    if s.align == ChatAlign::End {
        Align::End
    } else {
        Align::Start
    }
}

/// The fill, text and border colors of a bubble body, at rest or while a
/// pressable bubble is hovered. `hovered` only applies to pressable bubbles.
fn bubble_colors(t: &Theme, variant: ChatBubbleVariant, hovered: bool) -> (Color, Color, Color) {
    let clear = Color::TRANSPARENT;
    let dark = t.mode() == ThemeMode::Dark;
    match variant {
        ChatBubbleVariant::Default => (
            PRIMARY.with_alpha(if hovered { 0.8 } else { 1.0 }),
            PRIMARY_FOREGROUND,
            clear,
        ),
        ChatBubbleVariant::Secondary => (
            if hovered {
                SECONDARY.mix(FOREGROUND, 0.05, t)
            } else {
                SECONDARY
            },
            SECONDARY_FOREGROUND,
            clear,
        ),
        ChatBubbleVariant::Muted => (
            if hovered {
                MUTED.mix(FOREGROUND, 0.05, t)
            } else {
                MUTED
            },
            FOREGROUND,
            clear,
        ),
        ChatBubbleVariant::Tinted => (
            if hovered {
                BUBBLE_TINTED_HOVER
            } else {
                BUBBLE_TINTED
            },
            FOREGROUND,
            clear,
        ),
        ChatBubbleVariant::Outline => (
            match (hovered, dark) {
                (false, _) => BACKGROUND,
                (true, false) => MUTED,
                (true, true) => INPUT.with_alpha(0.3),
            },
            FOREGROUND,
            BORDER,
        ),
        ChatBubbleVariant::Ghost => (
            if hovered {
                MUTED.with_alpha(if dark { 0.5 } else { 1.0 })
            } else {
                clear
            },
            FOREGROUND,
            clear,
        ),
        ChatBubbleVariant::Destructive => {
            let (base, over) = if dark { (0.2, 0.3) } else { (0.1, 0.2) };
            (
                DESTRUCTIVE.with_alpha(if hovered { over } else { base }),
                DESTRUCTIVE,
                clear,
            )
        }
    }
}

/// The message: a row reversed for the end side; a standalone bubble is at most
/// 80 % wide (the ghost bubble, the full width), a thread's row the full width.
fn chat_message_root(_: &Theme, s: &Look<ChatMessageState>, _base: Style) -> Style {
    let base = style()
        .min_width(0.0)
        .font_size(12.0)
        .gap(6.0)
        .flex_direction(if s.align == ChatAlign::End {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        });
    if standalone(s) {
        base.align_self(edge(s)).max_width(
            if s.variant::<ChatBubbleVariant>() == Some(ChatBubbleVariant::Ghost) {
                pct(100.0)
            } else {
                pct(80.0)
            },
        )
    } else {
        base.width(pct(100.0))
    }
}

/// The column of the message's content, with room for its reactions above or
/// below; focusable when the message acts.
fn chat_message_body(t: &Theme, s: &Look<ChatMessageState>, _base: Style) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Column)
        .min_width(0.0)
        .gap(8.0)
        .align_items(edge(s))
        .position(Position::Relative);
    let base = if standalone(s) {
        base
    } else {
        base.flex_grow(1.0)
    };
    let base = if s.has_reactions {
        match s.reactions_side {
            ChatReactionsSide::Top => base.margin_top(18.0),
            ChatReactionsSide::Bottom => base.margin_bottom(18.0),
        }
    } else {
        base
    };
    if s.actionable {
        base.focus_visible(focus_ring(t))
    } else {
        base
    }
}

/// The bubble of the message's content: its variant's colors, and the hover
/// colors while the message acts.
fn chat_message_content(t: &Theme, s: &Look<ChatMessageState>, _base: Style) -> Style {
    let variant = s.variant::<ChatBubbleVariant>().unwrap_or_default();
    let (fill, fg, line) = bubble_colors(t, variant, false);
    let base = style()
        .flex_direction(FlexDirection::Column)
        .align_self(edge(s))
        .max_width(pct(100.0))
        .min_width(0.0)
        .border(1.0, line)
        .border_radius(0.0)
        .background(fill)
        .color(fg)
        .font_size(12.0)
        .line_height(1.625)
        .overflow(Overflow::Hidden);
    let base = if variant == ChatBubbleVariant::Ghost {
        base
    } else {
        base.padding_x(10.0).padding_y(8.0)
    };
    if s.actionable {
        let (fill, fg, line) = bubble_colors(t, variant, true);
        base.hover(style().background(fill).color(fg).border_color(line))
    } else {
        base
    }
}

/// A band of the message's author, header, footer, time or status: a
/// muted row of medium text on the message's side.
fn chat_message_band(_: &Theme, s: &Look<ChatMessageState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_self(Align::Stretch)
        .align_items(Align::Center)
        .justify_content(if s.align == ChatAlign::End {
            Justify::End
        } else {
            Justify::Start
        })
        .max_width(pct(100.0))
        .padding_x(10.0)
        .font_size(12.0)
        .font_weight(500)
        .color(MUTED_FOREGROUND)
}

/// The avatar: a round muted box, lifted over the footer when there is one.
fn chat_message_avatar(_: &Theme, s: &Look<ChatMessageState>, _base: Style) -> Style {
    let base = style()
        .min_width(32.0)
        .flex_shrink(0.0)
        .align_self(Align::End)
        .align_items(Align::Center)
        .border_radius(999.0)
        .background(MUTED);
    if s.has_footer {
        base.translate(0.0, -32.0)
    } else {
        base
    }
}

/// The reactions: a small muted row over the bubble's edge, on the message's side.
fn chat_message_reactions(_: &Theme, s: &Look<ChatMessageState>, _base: Style) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Row)
        .position(Position::Absolute)
        .align_items(Align::Center)
        .gap(4.0)
        .padding_x(6.0)
        .padding_y(2.0)
        .background(MUTED)
        .color(FOREGROUND)
        .font_size(12.0)
        .outline(2.0, CARD)
        .z_index(10);
    let base = match s.reactions_side {
        ChatReactionsSide::Top => base.top(0.0).translate(0.0, pct(-75.0)),
        ChatReactionsSide::Bottom => base.bottom(0.0).translate(0.0, pct(75.0)),
    };
    if s.align == ChatAlign::End {
        base.right(12.0)
    } else {
        base.left(12.0)
    }
}

/// The thread: a column filling its place, clipped at its edges.
fn chat_thread_root(_: &Theme, _: &Look<ChatThreadState>, _base: Style) -> Style {
    style()
        .position(Position::Relative)
        .flex_direction(FlexDirection::Column)
        .min_height(0.0)
        .min_width(0.0)
}

/// The scrolling viewport: a thin scrollbar in the border color.
fn chat_thread_viewport(_: &Theme, _: &Look<ChatThreadState>, _base: Style) -> Style {
    style()
        .flex_grow(1.0)
        .flex_shrink(1.0)
        .min_height(0.0)
        .min_width(0.0)
        .width(pct(100.0))
        .scrollbar_width(10.0)
        .scrollbar_color(BORDER, Color::TRANSPARENT)
}

/// The messages of the thread: a column with 24 px between them.
fn chat_thread_content(_: &Theme, _: &Look<ChatThreadState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .gap(24.0)
        .flex_shrink(0.0)
        .min_height(pct(100.0))
        .min_width(0.0)
}

/// The jump to the end: a 28 px round-cornered box, centred at the bottom.
fn chat_thread_jump(_: &Theme, _: &Look<ChatThreadState>, _base: Style) -> Style {
    style()
        .size(28.0, 28.0)
        .padding(0.0)
        .position(Position::Absolute)
        .left(pct(50.0) - 14.0)
        .bottom(16.0)
}

/// Installs the chat parts of the kit.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            ChatMessageTheme::default()
                .root(chat_message_root)
                .body(chat_message_body)
                .content(chat_message_content)
                .author(chat_message_band)
                .header(chat_message_band)
                .footer(chat_message_band)
                .avatar(chat_message_avatar)
                .reactions(chat_message_reactions)
                .time(chat_message_band)
                .status(chat_message_band),
        )
        .with(
            ChatThreadTheme::default()
                .root(chat_thread_root)
                .viewport(chat_thread_viewport)
                .content(chat_thread_content)
                .scroll_button(chat_thread_jump),
        )
}
