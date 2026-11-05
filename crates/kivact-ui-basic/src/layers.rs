//! Role: Shared looks of the overlays: scrim, close button box and text styles.

use crate::button::{Look, button_style};

use kivact::custom::PartState;
use kivact::style::{Position, Style, style, use_theme};
use kivact::ui::ControlSize;
use kivact::view::{Node, node};

/// The layer behind a modal: `fixed inset-0 bg-black/10 backdrop-blur-xs`,
/// in the theme's `overlay` color (black at 10 % in both modes). Call it
/// inside a style closure or a recipe.
pub(crate) fn scrim_style() -> Style {
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(use_theme().get().colors().overlay)
        .backdrop_blur(4)
}

/// A node drawn as [`scrim_style`].
pub(crate) fn scrim() -> Node {
    node().style(scrim_style)
}

/// A title: `text-sm font-medium` in the popover foreground. Call it inside
/// a style closure or a recipe.
pub(crate) fn title_style() -> Style {
    let p = kivact::style::use_tokens::<crate::Tokens>();
    style()
        .font_size(14)
        .weight(500)
        .line_height(20.0 / 14.0)
        .color(p.popover_foreground)
}

/// A description: `text-xs/relaxed text-muted-foreground`. Call it inside a
/// style closure or a recipe.
pub(crate) fn description_style() -> Style {
    style()
        .font_size(12)
        .line_height(1.625)
        .color(kivact::style::use_tokens::<crate::Tokens>().muted_foreground)
}

/// The ghost `icon-sm` close button's box, `top` and `right` px inside the
/// corner of its panel, for the input state `input`; the button holds the
/// theme's `x` icon. Call it inside a style closure or a recipe.
pub(crate) fn close_box(input: &PartState, top: f64, right: f64) -> Style {
    let p = kivact::style::use_tokens::<crate::Tokens>();
    button_style(&p, Look::Ghost, ControlSize::Small, input)
        .position(Position::Absolute)
        .top(top)
        .right(right)
        .size(28, 28)
        .padding(0)
}
