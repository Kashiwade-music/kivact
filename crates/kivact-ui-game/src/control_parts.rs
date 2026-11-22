//! Role: The button look and the diamond outline, shared by the console's recipes.

use crate::color::{
    DARK_SHADOW, NAVY, PRIMARY_HOVER, PRIMARY_STRONG, PRIMARY_WEAK, SIDEBAR_ALT, TEXT_2, YELLOW,
};
use crate::size::CONTROL_HEIGHT;
use kivact::draw::Path;
use kivact::geometry::{Point, Rect};
use kivact::style::color::{ACCENT, ACCENT_TEXT, DANGER, SURFACE};
use kivact::style::{Align, Color, FlexDirection, Justify, Shadow, Style, Transform, style};
use kivact::ui::{ButtonVariant, ControlSize};

/// The skew of a slanted button's root.
pub(crate) fn skew(slant: bool) -> f64 {
    if slant { -12.0 } else { 0.0 }
}

/// The look of a button of `variant` and `size`, slanted or not, busy or
/// not: what a segment of the console's radio group draws too.
pub(crate) fn button_style(
    variant: ButtonVariant,
    size: ControlSize,
    loading: bool,
    slant: bool,
) -> Style {
    let skew = skew(slant);
    let (bg, fg, hover, shadow) = match variant {
        ButtonVariant::Primary => (ACCENT, ACCENT_TEXT, PRIMARY_HOVER, PRIMARY_STRONG),
        ButtonVariant::Secondary => (NAVY, ACCENT_TEXT, SIDEBAR_ALT, DARK_SHADOW),
        ButtonVariant::Outline => (SURFACE, ACCENT, PRIMARY_WEAK, PRIMARY_WEAK),
        ButtonVariant::Ghost | ButtonVariant::Link => {
            (Color::TRANSPARENT, TEXT_2, PRIMARY_WEAK, Color::TRANSPARENT)
        }
        ButtonVariant::Danger => (DANGER, ACCENT_TEXT, DANGER, DANGER),
    };
    let mut s = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(bg)
        .color(fg)
        .font_size(13)
        .font_weight(700)
        .text_wrap(false)
        .transform(Transform::new().skew_x(skew))
        .shadow(Shadow::new(0, 2, 0, shadow))
        .hover(style().background(hover))
        .active(style().transform(Transform::new().skew_x(skew).translate_y(2)));
    if variant == ButtonVariant::Outline {
        s = s.border(1.5, ACCENT);
    }
    s = s.merge(&match size {
        ControlSize::ExtraSmall => style()
            .height(CONTROL_HEIGHT - 12.)
            .padding_x(8)
            .font_size(11),
        ControlSize::Small => style()
            .height(CONTROL_HEIGHT - 6.)
            .padding_x(12)
            .font_size(12),
        ControlSize::Medium => style().height(CONTROL_HEIGHT).padding_x(18).font_size(13),
        ControlSize::Large => style()
            .height(CONTROL_HEIGHT + 8.)
            .padding_x(28)
            .font_size(14),
    });
    if loading {
        s = s.disabled(style().opacity(1.));
    }
    if variant == ButtonVariant::Primary {
        s = s.paint(|d, s| d.fill_rect(Rect::new(s.width - 10., 5., 5., 5.), YELLOW));
    }
    s
}

/// The diamond outline filling the `side` square at `(x, y)`, inset by `inset`.
pub(crate) fn diamond(x: f64, y: f64, side: f64, inset: f64) -> Path {
    let (lo, mid, hi) = (inset, side / 2., side - inset);
    Path::new()
        .move_to(Point::new(x + mid, y + lo))
        .line_to(Point::new(x + hi, y + mid))
        .line_to(Point::new(x + mid, y + hi))
        .line_to(Point::new(x + lo, y + mid))
        .close()
}
