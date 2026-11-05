//! Role: Basic visual badge variants and their recipe.
use crate::theme_access::wide_focus_ring;
use crate::tokens::Tokens;
use kivact::custom::PartState;
use kivact::style::{Align, Color, Cursor, Justify, TextDecorationLine, style};
/// The look of a standard `kivact::ui::badge` (badge.tsx `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BadgeVariant {
    /// Solid primary.
    #[default]
    Default,
    /// Secondary fill.
    Secondary,
    /// Tinted destructive.
    Destructive,
    /// Bordered, no fill.
    Outline,
    /// No fill; muted on hover.
    Ghost,
    /// Primary text, underlined on hover.
    Link,
}

impl kivact::ui::Variant for BadgeVariant {}
/// Background, text and border colors of `variant`; hover colors apply to pressable badges.
/// The fill, text and border colors of `variant`, at rest or while a
/// pressable badge is hovered.
fn colors(p: &Tokens, variant: BadgeVariant, hovered: bool) -> (Color, Color, Color) {
    let dark = p.is_dark();
    let clear = Color::TRANSPARENT;
    match variant {
        BadgeVariant::Default => (
            p.primary.with_alpha(if hovered { 0.8 } else { 1.0 }),
            p.primary_foreground,
            clear,
        ),
        BadgeVariant::Secondary => (
            p.secondary.with_alpha(if hovered { 0.8 } else { 1.0 }),
            p.secondary_foreground,
            clear,
        ),
        BadgeVariant::Destructive => {
            let (base, over) = if dark { (0.2, 0.3) } else { (0.1, 0.2) };
            let alpha = if hovered { over } else { base };
            (p.destructive.with_alpha(alpha), p.destructive, clear)
        }
        BadgeVariant::Outline => {
            if hovered {
                (p.muted, p.muted_foreground, p.border)
            } else {
                (clear, p.foreground, p.border)
            }
        }
        BadgeVariant::Ghost => {
            if hovered {
                (
                    p.muted.with_alpha(if dark { 0.5 } else { 1.0 }),
                    p.muted_foreground,
                    clear,
                )
            } else {
                (clear, p.foreground, clear)
            }
        }
        BadgeVariant::Link => (clear, p.primary, clear),
    }
}

/// badge.tsx: the variant's colors, and while pressable the hover colors,
/// the underline of a hovered link (a text decoration, which a state style
/// cannot change, so it follows `input`) and the wide focus ring.
fn badge_style(
    p: &Tokens,
    variant: BadgeVariant,
    pressable: bool,
    input: &PartState,
    padding: (f64, f64),
) -> kivact::style::Style {
    let (fill, fg, line) = colors(p, variant, false);
    let mut s = style()
        .direction(kivact::style::Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .shrink(0)
        .gap(4)
        .height(20)
        .padding_left(padding.0)
        .padding_right(padding.1)
        .radius(0)
        .border(1, line)
        .background(fill)
        .color(fg)
        .font_size(12)
        .weight(500)
        .text_wrap(false)
        .clip(true);
    if variant == BadgeVariant::Link && input.hovered {
        s = s
            .text_decoration(TextDecorationLine::UNDERLINE)
            .text_underline_offset(4);
    }
    if !pressable {
        return s;
    }
    let (fill, fg, line) = colors(p, variant, true);
    let focus = if variant == BadgeVariant::Destructive {
        style().border_color(p.destructive.with_alpha(0.4)).ring(
            3,
            p.destructive
                .with_alpha(if p.is_dark() { 0.4 } else { 0.2 }),
        )
    } else {
        wide_focus_ring(p)
    };
    s.cursor(Cursor::Pointer)
        .hover(style().background(fill).color(fg).border_color(line))
        .focus_visible(focus)
}

pub(crate) fn recipe(
    t: &kivact::style::Theme,
    s: &kivact::ui::Look<kivact::ui::BadgeState>,
) -> kivact::style::Style {
    badge_style(
        &t.ext::<Tokens>(),
        s.variant::<BadgeVariant>().copied().unwrap_or_default(),
        s.actionable,
        &s.input,
        (
            if s.leading_icon { 6.0 } else { 8.0 },
            if s.trailing_icon { 6.0 } else { 8.0 },
        ),
    )
}
