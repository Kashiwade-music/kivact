//! Role: Basic message and standalone bubble recipes for the canonical chat message.
use crate::{BubbleVariant, Tokens};
use kivact::style::{Align, Direction, Justify, Overflow, Position, Style, Theme, pct, style};
use kivact::ui::{ChatAlign, ChatMessageRecipes, ChatMessageState, ChatReactionsSide, Look};
/// The Basic visual arrangement of a canonical chat message.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatLayout {
    /// A full-width message row with optional author, avatar and metadata.
    #[default]
    Message,
    /// A standalone bubble sized to its contents.
    Bubble,
}
impl kivact::ui::Variant for ChatLayout {}
fn standalone(s: &Look<ChatMessageState>) -> bool {
    s.variant::<ChatLayout>() == Some(&ChatLayout::Bubble)
}
fn edge(s: &Look<ChatMessageState>) -> Align {
    if s.align == ChatAlign::End {
        Align::End
    } else {
        Align::Start
    }
}
pub(crate) fn recipes(r: ChatMessageRecipes) -> ChatMessageRecipes {
    r.root(|_| root)
        .body(|_| body)
        .content(|_| content)
        .author(|_| band)
        .header(|_| band)
        .footer(|_| band)
        .avatar(|_| avatar)
        .reactions(|_| reactions)
        .time(|_| band)
        .status(|_| band)
}
fn root(_: &Theme, s: &Look<ChatMessageState>) -> Style {
    let base = style()
        .min_width(0)
        .font_size(12)
        .gap(6)
        .direction(if s.align == ChatAlign::End {
            Direction::RowReverse
        } else {
            Direction::Row
        });
    if standalone(s) {
        base.align_self(edge(s)).max_width(
            if s.variant::<BubbleVariant>() == Some(&BubbleVariant::Ghost) {
                pct(100)
            } else {
                pct(80)
            },
        )
    } else {
        base.width(pct(100))
    }
}
fn body(t: &Theme, s: &Look<ChatMessageState>) -> Style {
    let base = style()
        .direction(Direction::Column)
        .min_width(0)
        .gap(8)
        .align_items(edge(s))
        .position(Position::Relative);
    let base = if standalone(s) { base } else { base.grow(1) };
    let base = if s.has_reactions {
        match s.reactions_side {
            ChatReactionsSide::Top => base.margin_top(18),
            ChatReactionsSide::Bottom => base.margin_bottom(18),
        }
    } else {
        base
    };
    if s.actionable {
        base.focus_visible(crate::theme_access::focus_ring(&t.ext::<Tokens>()))
    } else {
        base
    }
}
fn content(t: &Theme, s: &Look<ChatMessageState>) -> Style {
    let p = t.ext::<Tokens>();
    let variant = s.variant::<BubbleVariant>().copied().unwrap_or_default();
    let (fill, fg, line) = crate::chat_colors::colors(&p, variant, false);
    let base = style()
        .direction(Direction::Column)
        .align_self(edge(s))
        .max_width(pct(100))
        .min_width(0)
        .border(1, line)
        .radius(0)
        .background(fill)
        .color(fg)
        .font_size(12)
        .line_height(1.625)
        .overflow(Overflow::Hidden);
    let base = if variant == BubbleVariant::Ghost {
        base
    } else {
        base.padding_x(10).padding_y(8)
    };
    if s.actionable {
        let (fill, fg, line) = crate::chat_colors::colors(&p, variant, true);
        base.hover(style().background(fill).color(fg).border_color(line))
    } else {
        base
    }
}
fn band(t: &Theme, s: &Look<ChatMessageState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_self(Align::Stretch)
        .align_items(Align::Center)
        .justify_content(if s.align == ChatAlign::End {
            Justify::End
        } else {
            Justify::Start
        })
        .max_width(pct(100))
        .padding_x(10)
        .font_size(12)
        .weight(500)
        .color(t.ext::<Tokens>().muted_foreground)
}
fn avatar(t: &Theme, s: &Look<ChatMessageState>) -> Style {
    let base = style()
        .min_width(32)
        .shrink(0)
        .align_self(Align::End)
        .align_items(Align::Center)
        .radius(999)
        .background(t.ext::<Tokens>().muted);
    if s.has_footer {
        base.translate(0, -32)
    } else {
        base
    }
}
fn reactions(t: &Theme, s: &Look<ChatMessageState>) -> Style {
    let p = t.ext::<Tokens>();
    let base = style()
        .direction(Direction::Row)
        .position(Position::Absolute)
        .align_items(Align::Center)
        .gap(4)
        .padding_x(6)
        .padding_y(2)
        .background(p.muted)
        .color(p.foreground)
        .font_size(12)
        .ring(2, p.card)
        .z_index(10);
    let base = match s.reactions_side {
        ChatReactionsSide::Top => base.top(0).translate(0, pct(-75)),
        ChatReactionsSide::Bottom => base.bottom(0).translate(0, pct(75)),
    };
    if s.align == ChatAlign::End {
        base.right(12)
    } else {
        base.left(12)
    }
}
