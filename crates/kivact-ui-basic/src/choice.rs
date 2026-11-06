//! Role: Toggle group built on the toggle and list interactions, and the toggle look.

use crate::theme_access::wide_focus_ring;
use crate::tokens::Tokens;
use kivact::style::{Align, Color, Direction, Justify, Style, style};
use kivact::ui::{ControlSize, Variant};

/// The two looks of a toggle: the variant of a `kivact::ui::toggle_button` and of
/// a standard `kivact::ui::toggle_group`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToggleVariant {
    /// Transparent.
    #[default]
    Default,
    /// With a 1 px `input` border.
    Outline,
}

impl Variant for ToggleVariant {}

/// toggle.tsx: `inline-flex center gap-1 text-xs font-medium`, muted when on,
/// with the wide ring while the focus is visible.
pub(crate) fn toggle_style(
    p: &Tokens,
    variant: ToggleVariant,
    size: ControlSize,
    pressed: bool,
    icon_start: bool,
    joined: bool,
) -> Style {
    let (height, pad_left) = match size {
        ControlSize::ExtraSmall | ControlSize::Small => (28.0, if icon_start { 6.0 } else { 10.0 }),
        ControlSize::Medium => (32.0, if icon_start { 8.0 } else { 10.0 }),
        ControlSize::Large => (36.0, if icon_start { 8.0 } else { 10.0 }),
    };
    // `group-data-[spacing=0]/toggle-group:px-2` and `has-data-[icon=inline-start]:pl-1.5`.
    let (pad_left, pad_right) = if joined {
        (if icon_start { 6.0 } else { 8.0 }, 8.0)
    } else {
        (pad_left, 10.0)
    };
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(4)
        .shrink(0)
        .height(height)
        .min_width(height)
        .padding_left(pad_left)
        .padding_right(pad_right)
        .radius(0)
        .border(
            1,
            if variant == ToggleVariant::Outline {
                p.input
            } else {
                Color::TRANSPARENT
            },
        )
        .font_size(12)
        .weight(500)
        .text_wrap(false)
        .color(p.foreground)
        .background(if pressed { p.muted } else { Color::TRANSPARENT })
        .hover(style().background(p.muted))
        // The focused item of a group draws its ring over its neighbours.
        .focus_visible(wide_focus_ring(p).z_index(1))
}
