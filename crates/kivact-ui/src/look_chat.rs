//! Role: Supply token-based default chat, typing and code recipes.
use crate::{
    ChatAlign, ChatBubbleGroup, ChatComposerState, ChatMessageState, ChatMessageVariant,
    ChatReactionsSide, CodeBlockState, ComposerStatus, Look, TypingIndicatorState,
};
use kivact_core::style::{
    Align, ColorToken, Direction, Justify, LineHeight, Style, Theme, pct, style,
};
pub(super) fn column<S>(t: &Theme, _: &Look<S>) -> Style {
    style()
        .direction(Direction::Column)
        .gap(t.space().sm)
        .min_width(0)
}
pub(super) fn row<S>(t: &Theme, _: &Look<S>) -> Style {
    style()
        .direction(Direction::Row)
        .gap(t.space().sm)
        .align_items(Align::Center)
        .min_width(0)
}
pub(super) fn slot<S>(_: &Theme, _: &Look<S>) -> Style {
    style().min_width(0)
}
pub(super) fn muted<S>(t: &Theme, _: &Look<S>) -> Style {
    style()
        .font_size(t.text().size - 2.0)
        .color(ColorToken::TextMuted)
}
pub(super) fn author(t: &Theme, _: &Look<ChatMessageState>) -> Style {
    style()
        .font_size(t.text().size)
        .weight(t.text().weight_strong)
        .color(ColorToken::Text)
}
pub(super) fn message_root(t: &Theme, s: &Look<ChatMessageState>) -> Style {
    row(t, s)
        .align_items(Align::Start)
        .justify_content(if s.align == ChatAlign::End {
            Justify::End
        } else {
            Justify::Start
        })
}
pub(super) fn message_body(t: &Theme, s: &Look<ChatMessageState>) -> Style {
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
pub(super) fn thread_root<S>(t: &Theme, s: &Look<S>) -> Style {
    column(t, s)
        .min_height(0)
        .position(kivact_core::style::Position::Relative)
}
pub(super) fn viewport<S>(_: &Theme, _: &Look<S>) -> Style {
    style().grow(1).min_height(0).min_width(0)
}
pub(super) fn composer_surface(t: &Theme, s: &Look<ChatComposerState>) -> Style {
    column(t, s)
        .padding(t.space().md)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .background(ColorToken::Surface)
}
pub(super) fn composer_status(t: &Theme, s: &Look<ChatComposerState>) -> Style {
    muted(t, s).color(match s.status {
        Some(ComposerStatus::Error(_)) => ColorToken::Danger,
        _ => ColorToken::Warning,
    })
}
pub(super) fn input<S>(_: &Theme, _: &Look<S>) -> Style {
    style().grow(1).min_width(0)
}
pub(super) fn typing_dots(_: &Theme, s: &Look<TypingIndicatorState>) -> Style {
    let phase = s.phase;
    style().size(48, 24).paint(move |d, _| {
        for i in 0..3 {
            let y = 12.0 - (phase - i as f64 * 0.95).sin().max(0.0) * 3.0;
            d.rounded_rect(
                kivact_core::geometry::Rect::new(7.0 + i as f64 * 12.0, y, 5.0, 5.0),
                2.5,
                ColorToken::TextMuted,
            );
        }
    })
}
pub(super) fn code_root(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    column(t, s)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
}
pub(super) fn code_header(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    row(t, s).padding(t.space().sm)
}
pub(super) fn code_body(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    row(t, s).align_items(Align::Start).min_width(pct(100))
}
pub(super) fn code_viewport(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    viewport(t, s).padding(t.space().md)
}
pub(super) fn code_text(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    let tokens = t.ext::<crate::UiTokens>();
    let metrics = &tokens.code_block;
    style()
        .font_family("monospace")
        .font_size(metrics.font_size)
        .line_height(LineHeight::px(metrics.line_height))
        .text_wrap(s.wrapped)
        .grow(1)
        .min_width(0)
}
pub(super) fn code_gutter(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    code_text(t, s)
        .grow(0)
        .shrink(0)
        .min_width(24)
        .text_wrap(false)
        .color(ColorToken::TextMuted)
        .text_align(kivact_core::style::TextAlign::End)
}

pub(super) fn reactions(_: &Theme, s: &Look<ChatMessageState>) -> Style {
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

pub(super) fn jump<S>(_: &Theme, _: &Look<S>) -> Style {
    style()
        .position(kivact_core::style::Position::Absolute)
        .right(12)
        .bottom(12)
}
