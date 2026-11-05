//! Role: Basic chat color variants, independent of message interaction.
use crate::Tokens;
use kivact::style::Color;
/// The Basic color treatment of a standard chat message content slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BubbleVariant {
    /// Solid primary: the sender's own text.
    #[default]
    Default,
    /// Secondary fill.
    Secondary,
    /// Muted fill.
    Muted,
    /// A faint gray derived from the primary color.
    Tinted,
    /// Bordered on the background.
    Outline,
    /// No fill, border or padding.
    Ghost,
    /// Tinted destructive.
    Destructive,
}

impl kivact::ui::Variant for BubbleVariant {}
/// `oklch(from primary 0.93 ..)` and friends for the neutral primary: the grays of lightness
/// 0.93, 0.88 (light) and 0.30, 0.35 (dark).
fn tinted(p: &Tokens, hovered: bool) -> Color {
    if hovered {
        p.bubble_tinted_hover
    } else {
        p.bubble_tinted
    }
}

/// Fill, text and border colors of a bubble body; `hovered` only applies to pressable bubbles.
pub(crate) fn colors(p: &Tokens, variant: BubbleVariant, hovered: bool) -> (Color, Color, Color) {
    let clear = Color::TRANSPARENT;
    let dark = p.is_dark();
    match variant {
        BubbleVariant::Default => (
            p.primary.with_alpha(if hovered { 0.8 } else { 1.0 }),
            p.primary_foreground,
            clear,
        ),
        BubbleVariant::Secondary => (
            if hovered {
                p.secondary.mix(p.foreground, 0.05)
            } else {
                p.secondary
            },
            p.secondary_foreground,
            clear,
        ),
        BubbleVariant::Muted => (
            if hovered {
                p.muted.mix(p.foreground, 0.05)
            } else {
                p.muted
            },
            p.foreground,
            clear,
        ),
        BubbleVariant::Tinted => (tinted(p, hovered), p.foreground, clear),
        BubbleVariant::Outline => (
            match (hovered, dark) {
                (false, _) => p.background,
                (true, false) => p.muted,
                (true, true) => p.input.with_alpha(0.3),
            },
            p.foreground,
            p.border,
        ),
        BubbleVariant::Ghost => (
            if hovered {
                p.muted.with_alpha(if dark { 0.5 } else { 1.0 })
            } else {
                clear
            },
            p.foreground,
            clear,
        ),
        BubbleVariant::Destructive => {
            let (base, over) = if dark { (0.2, 0.3) } else { (0.1, 0.2) };
            (
                p.destructive.with_alpha(if hovered { over } else { base }),
                p.destructive,
                clear,
            )
        }
    }
}
