//! Role: Paint canonical chat surfaces with Soft conversation treatments.
use crate::{
    BubbleVariant, ChatAppearance, ChatDensity, ComposerElevation, SystemMessageVariant, Tokens,
};
use kivact::style::{Align, Direction, Justify, Style, Theme, pct, style};
use kivact::ui::{
    ChatBubbleGroup, ChatComposerRecipes, ChatComposerState, ChatMessageRecipes, ChatMessageState,
    ChatMessageVariant, ChatSender, ChatThreadRecipes, ChatThreadState, Look,
};
fn density<S>(s: &Look<S>) -> ChatDensity {
    s.variant::<ChatAppearance>()
        .map(|a| a.density.get())
        .or_else(|| s.variant::<ChatDensity>().copied())
        .unwrap_or_default()
}
fn gap(d: ChatDensity) -> i32 {
    match d {
        ChatDensity::Compact => 8,
        ChatDensity::Balanced => 16,
        ChatDensity::Spacious => 24,
    }
}
pub(crate) fn message(r: ChatMessageRecipes) -> ChatMessageRecipes {
    r.root(|_| message_root)
        .body(|_| message_body)
        .author(|_| author)
        .metadata(|_| muted)
        .time(|_| muted)
        .status(|_| muted)
}
fn message_root(_: &Theme, s: &Look<ChatMessageState>) -> Style {
    style()
        .direction(if s.sender == ChatSender::User {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(Align::Start)
        .gap(gap(density(s)) / 2)
        .justify_content(if s.sender == ChatSender::System {
            Justify::Center
        } else {
            Justify::Start
        })
}
fn message_body(t: &Theme, s: &Look<ChatMessageState>) -> Style {
    let p = t.ext::<Tokens>();
    let d = density(s);
    let bubble = s
        .variant::<ChatAppearance>()
        .map(|a| a.bubble.get())
        .or_else(|| s.variant::<BubbleVariant>().copied())
        .unwrap_or_else(|| {
            if s.variant::<ChatMessageVariant>() == Some(&ChatMessageVariant::Plain) {
                BubbleVariant::Ghost
            } else {
                BubbleVariant::Filled
            }
        });
    let mut look = style()
        .direction(Direction::Column)
        .min_width(0)
        .max_width(pct(90))
        .gap(gap(d) / 4)
        .font_size(14)
        .line_height(20.0 / 14.0)
        .color(p.colors.text_primary);
    if s.sender != ChatSender::System && bubble == BubbleVariant::Filled {
        look = look
            .padding_x(if d == ChatDensity::Spacious { 20 } else { 16 })
            .padding_y(if d == ChatDensity::Spacious { 16 } else { 12 })
            .radius(if d == ChatDensity::Compact {
                p.radius.container
            } else {
                28.0
            })
            .background(p.colors.neutral);
    }
    if s.sender == ChatSender::System {
        let divider = s
            .variant::<ChatAppearance>()
            .map(|a| a.system.get())
            .or_else(|| s.variant::<SystemMessageVariant>().copied())
            .unwrap_or_default();
        if divider == SystemMessageVariant::Divider {
            look = look
                .border_top(1, p.colors.border)
                .border_bottom(1, p.colors.border)
                .padding_y(4);
        }
    }
    let user = (s.sender == ChatSender::User)
        != (kivact::view::use_direction().get() == kivact::view::ReadingDirection::Rtl);
    if matches!(s.group, ChatBubbleGroup::Middle | ChatBubbleGroup::Last) {
        look = if user {
            look.radius_top_right(6)
        } else {
            look.radius_top_left(6)
        };
    }
    if matches!(s.group, ChatBubbleGroup::Middle | ChatBubbleGroup::First) {
        look = if user {
            look.radius_bottom_right(6)
        } else {
            look.radius_bottom_left(6)
        };
    }
    look
}
fn author(t: &Theme, _: &Look<ChatMessageState>) -> Style {
    style()
        .font_size(12)
        .line_height(20.0 / 12.0)
        .weight(600)
        .color(t.ext::<Tokens>().colors.text_secondary)
}
fn muted<S>(t: &Theme, _: &Look<S>) -> Style {
    style()
        .font_size(12)
        .line_height(20.0 / 12.0)
        .color(t.ext::<Tokens>().colors.text_secondary)
}
pub(crate) fn thread(r: ChatThreadRecipes) -> ChatThreadRecipes {
    r.content(|_| thread_content).composer(|_| thread_composer)
}
fn thread_content(_: &Theme, s: &Look<ChatThreadState>) -> Style {
    let g = gap(density(s));
    style()
        .direction(Direction::Column)
        .grow(1)
        .padding_y(g)
        .padding_x(g.max(12))
        .gap(g)
}
fn thread_composer(_: &Theme, s: &Look<ChatThreadState>) -> Style {
    style()
        .padding_x(gap(density(s)))
        .padding_bottom(gap(density(s)))
}
pub(crate) fn composer(r: ChatComposerRecipes) -> ChatComposerRecipes {
    r.surface(|_| composer_surface)
        .input(|_| input)
        .status(|_| muted)
}
fn composer_surface(t: &Theme, s: &Look<ChatComposerState>) -> Style {
    let p = t.ext::<Tokens>();
    let compact = density(s) == ChatDensity::Compact;
    let mut look = style()
        .direction(Direction::Column)
        .min_width(0)
        .padding(if compact { 8 } else { 12 })
        .gap(4)
        .radius(if compact { 16 } else { 28 })
        .background(p.colors.bg_surface)
        .border(1, p.colors.border);
    let elevation = s
        .variant::<ChatAppearance>()
        .map(|a| a.elevation.get())
        .or_else(|| s.variant::<ComposerElevation>().copied())
        .unwrap_or_default();
    if elevation == ComposerElevation::Low {
        for shadow in &p.shadows.low {
            look = look.shadow(*shadow);
        }
    }
    look
}
fn input(t: &Theme, _: &Look<ChatComposerState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .padding(4)
        .border(0, p.colors.border)
        .background(kivact::style::Color::TRANSPARENT)
        .focus_visible(style().border(0, p.colors.border))
}
