//! Role: The Pop looks of the conversation: bubbles, the thread, the composer and the typing indicator.

use crate::color;
use crate::variants::ChatBubbleVariant;
use kivact::geometry::Rect;
use kivact::style::{
    Align, Color, Display, FlexDirection, Length, Style, Theme, Track, pct, style,
};
use kivact::ui::theme::{
    ChatComposerState, ChatComposerTheme, ChatMessageState, ChatMessageTheme, ChatThreadState,
    ChatThreadTheme, Look, TypingIndicatorState, TypingIndicatorTheme,
};
use kivact::ui::{ChatAlign, ChatBubbleGroup, ChatMessageVariant};

/// Install the kit's recipes of the conversation.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(
            ChatMessageTheme::default()
                .root(message_root)
                .body(message_body)
                .author(message_author)
                .avatar(message_avatar)
                .content(message_content)
                .time(message_time),
        )
        .with(
            ChatThreadTheme::default()
                .root(thread_root)
                .scroll_button(thread_scroll_button),
        )
        .with(
            ChatComposerTheme::default()
                .root(composer_root)
                .surface(composer_surface)
                .header(composer_header)
                .input(composer_input)
                .footer(composer_footer)
                .send(composer_send),
        );
    theme.with(
        TypingIndicatorTheme::default()
            .root(typing_root)
            .dots(typing_dots),
    )
}

/// Whether the message is a bubble of the kit's conversation look.
fn is_bubble(s: &Look<ChatMessageState>) -> bool {
    s.variant::<ChatBubbleVariant>().is_some()
}

fn message_root(_theme: &Theme, s: &Look<ChatMessageState>, base: Style) -> Style {
    let base = base.gap(10).align_items(Align::Start).min_width(0);
    if is_bubble(s) {
        base.max_width(pct(78))
            .align_self(if s.align == ChatAlign::End {
                Align::End
            } else {
                Align::Start
            })
            .padding_left(
                if s.align == ChatAlign::Start
                    && matches!(s.group, ChatBubbleGroup::Middle | ChatBubbleGroup::Last)
                {
                    34
                } else {
                    0
                },
            )
    } else {
        base
    }
}

fn message_body(_theme: &Theme, s: &Look<ChatMessageState>, base: Style) -> Style {
    let base = base.border_radius(14).gap(4);
    if is_bubble(s) {
        base.padding(0)
            .background(Color::TRANSPARENT)
            .max_width(pct(100))
    } else if s.variant::<ChatMessageVariant>() == Some(ChatMessageVariant::Plain) {
        base
    } else {
        base.background(if s.align == ChatAlign::End {
            color::BLUE
        } else {
            color::SURFACE_ACTIVE
        })
        .color(if s.align == ChatAlign::End {
            Color::WHITE
        } else {
            color::TEXT
        })
    }
}

fn message_author(_theme: &Theme, s: &Look<ChatMessageState>, base: Style) -> Style {
    if is_bubble(s) {
        base.position(kivact::style::Position::Absolute)
            .visibility(kivact::style::Visibility::Hidden)
    } else {
        base.color(color::TEXT)
    }
}

fn message_avatar(_theme: &Theme, s: &Look<ChatMessageState>, base: Style) -> Style {
    if is_bubble(s) && s.align == ChatAlign::End {
        base.position(kivact::style::Position::Absolute)
            .visibility(kivact::style::Visibility::Hidden)
    } else {
        base
    }
}

/// The bubble's text: a 14 px body with the tail corner on the side of the sender.
fn message_content(_theme: &Theme, s: &Look<ChatMessageState>, base: Style) -> Style {
    if !is_bubble(s) {
        return base;
    }
    let mine = s.align == ChatAlign::End;
    let first = matches!(s.group, ChatBubbleGroup::Single | ChatBubbleGroup::First);
    base.font_size(14)
        .padding_x(12)
        .padding_y(8)
        .border_radius_corners(
            if !mine && first { 4 } else { 14 },
            if mine && first { 4 } else { 14 },
            14,
            14,
        )
        .background(if mine {
            color::BLUE
        } else {
            color::SURFACE_ACTIVE
        })
        .color(if mine { Color::WHITE } else { color::TEXT })
}

fn message_time(_theme: &Theme, _: &Look<ChatMessageState>, base: Style) -> Style {
    base.font_size(11)
        .padding_x(4)
        .align_self(Align::End)
        .color(color::TEXT_MUTED)
}

fn thread_root(_theme: &Theme, _: &Look<ChatThreadState>, base: Style) -> Style {
    base.background(color::SURFACE)
}

fn thread_scroll_button(_theme: &Theme, _: &Look<ChatThreadState>, base: Style) -> Style {
    base.border_radius(8)
        .background(color::SURFACE)
        .border(1, color::LINE)
}

fn composer_root(_theme: &Theme, _: &Look<ChatComposerState>, base: Style) -> Style {
    base.border_top(1, color::LINE)
}

fn composer_surface(_theme: &Theme, _: &Look<ChatComposerState>, _base: Style) -> Style {
    style()
        .display(Display::Grid)
        .grid_template_columns((Length::Auto, Track::fr(1), Length::Auto))
        .gap(8)
        .padding(12)
        .background(color::SURFACE)
}

fn composer_header(_theme: &Theme, _: &Look<ChatComposerState>, _base: Style) -> Style {
    style()
        .grid_column(1)
        .grid_row(1)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
}

fn composer_input(_theme: &Theme, _: &Look<ChatComposerState>, base: Style) -> Style {
    base.grid_column(2)
        .grid_row(1)
        .height(36)
        .min_height(0)
        .flex_grow(1)
}

fn composer_footer(_theme: &Theme, _: &Look<ChatComposerState>, _base: Style) -> Style {
    style()
        .grid_column(3)
        .grid_row(1)
        .flex_direction(FlexDirection::Row)
        .gap(8)
        .align_items(Align::Center)
}

fn composer_send(_theme: &Theme, _: &Look<ChatComposerState>, base: Style) -> Style {
    base.background(color::PINK)
        .color(Color::WHITE)
        .border_radius(8)
}

fn typing_root(_theme: &Theme, _: &Look<TypingIndicatorState>, base: Style) -> Style {
    base.gap(10).align_items(Align::Start).max_width(pct(78))
}

/// The three dots of the typing indicator, bouncing in turn.
fn typing_dots(_theme: &Theme, s: &Look<TypingIndicatorState>, _base: Style) -> Style {
    let (phase, color) = (s.phase, color::TEXT_FAINT);
    style()
        .size(54, 30)
        .border_radius_corners(4, 14, 14, 14)
        .background(color::SURFACE_ACTIVE)
        .paint(move |d, _| {
            for index in 0..3 {
                let bounce = (phase - index as f64 * 0.95).sin().max(0.0) * 3.0;
                d.rounded_rect(
                    Rect::new(14.0 + index as f64 * 10.0, 12.0 - bounce, 6.0, 6.0),
                    3.0,
                    color,
                );
            }
        })
}
