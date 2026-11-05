//! Role: The Lyra button looks shared by recipes, triggers and kit components.

use crate::theme_access::with_focus;
use crate::tokens::Tokens;
use kivact::custom::PartState;
use kivact::style::{Align, Color, Direction, Justify, Style, TextDecorationLine, style};
use kivact::ui::{ButtonVariant, ControlSize};

/// The six button looks of button.tsx, one per core variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Look {
    Default,
    Outline,
    Secondary,
    Ghost,
    Destructive,
    Link,
}

impl Look {
    pub(crate) fn of(variant: ButtonVariant) -> Self {
        match variant {
            ButtonVariant::Primary => Look::Default,
            ButtonVariant::Outline => Look::Outline,
            ButtonVariant::Secondary => Look::Secondary,
            ButtonVariant::Ghost => Look::Ghost,
            ButtonVariant::Danger => Look::Destructive,
            ButtonVariant::Link => Look::Link,
        }
    }

    /// The text (and icon) color of the look.
    pub(crate) fn foreground(self, p: &Tokens) -> Color {
        match self {
            Look::Default => p.primary_foreground,
            Look::Outline | Look::Ghost => p.foreground,
            Look::Secondary => p.secondary_foreground,
            Look::Destructive => p.destructive,
            Look::Link => p.primary,
        }
    }
}

/// Height, horizontal padding, gap and icon side of a button size.
pub(crate) fn metrics(size: ControlSize) -> (f64, f64, f64, f64) {
    match size {
        ControlSize::ExtraSmall => (24.0, 8.0, 4.0, 12.0),
        ControlSize::Small => (28.0, 10.0, 4.0, 14.0),
        ControlSize::Medium => (32.0, 10.0, 6.0, 16.0),
        ControlSize::Large => (36.0, 10.0, 6.0, 16.0),
    }
}

/// The complete button style of `look` at `size` for the actual input state.
pub(crate) fn button_style(p: &Tokens, look: Look, size: ControlSize, input: &PartState) -> Style {
    trigger_style(p, look, size, input, None)
}

/// [`button_style`] for a button that opens something: `expanded` is
/// `Some(open)` for a popup or disclosure trigger, which does not move
/// while pressed (`active:not-aria-[haspopup]:translate-y-px`) and, open,
/// takes the muted fill of an outline or ghost button
/// (`aria-expanded:bg-muted`).
pub(crate) fn trigger_style(
    p: &Tokens,
    look: Look,
    size: ControlSize,
    input: &PartState,
    expanded: Option<bool>,
) -> Style {
    let (height, pad, gap, _) = metrics(size);
    let dark = p.is_dark();
    let s = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .shrink(0)
        .gap(gap)
        .height(height)
        .padding_x(pad)
        .radius(0)
        .border(1, Color::TRANSPARENT)
        // `text-xs`: 12 px on a 16 px line, whatever line height the
        // button's parent sets (a card body's `text-xs/relaxed`).
        .font_size(12)
        .line_height(16.0 / 12.0)
        .weight(500)
        .text_wrap(false)
        .color(look.foreground(p));
    let s = if expanded.is_none() {
        s.pressed(style().translate_y(1))
    } else {
        s
    };
    let open = expanded == Some(true);
    let s = match look {
        Look::Default => s
            .background(p.primary)
            .hover(style().background(p.primary.with_alpha(0.8))),
        Look::Outline => {
            let (fill, hover) = if dark {
                (p.input.with_alpha(0.3), p.input.with_alpha(0.5))
            } else {
                (p.background, p.muted)
            };
            s.background(if open { p.muted } else { fill })
                .border_color(if dark { p.input } else { p.border })
                .hover(style().background(hover))
        }
        Look::Secondary => s
            .background(p.secondary)
            .hover(style().background(p.secondary.mix(p.foreground, 0.05))),
        Look::Ghost => s
            .background(if open { p.muted } else { Color::TRANSPARENT })
            .hover(style().background(if dark {
                p.muted.with_alpha(0.5)
            } else {
                p.muted
            })),
        Look::Destructive => {
            let (fill, hover) = if dark { (0.2, 0.3) } else { (0.1, 0.2) };
            s.background(p.destructive.with_alpha(fill))
                .hover(style().background(p.destructive.with_alpha(hover)))
        }
        Look::Link => {
            let s = s.background(Color::TRANSPARENT).text_underline_offset(4);
            if input.hovered && !input.disabled {
                s.text_decoration(TextDecorationLine::UNDERLINE)
            } else {
                s
            }
        }
    };
    if look == Look::Destructive {
        let ring = p.destructive.with_alpha(if dark { 0.4 } else { 0.2 });
        s.focus_visible(
            style()
                .border_color(p.destructive.with_alpha(0.4))
                .ring(1, ring),
        )
    } else {
        with_focus(s, p)
    }
}
