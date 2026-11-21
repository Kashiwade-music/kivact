//! Role: The toolbar recipe state, recipe table and default looks.

use crate::control_size::ControlSize;
use crate::recipe::recipe_table;

use kivact_core::style::{Align, FlexDirection, Justify, Length, Track, pct, style};
use kivact_core::style::{Color, ThemeVar};
use kivact_core::view::Orientation;

/// The background of a toolbar.
pub static BACKGROUND_VAR: ThemeVar<Color> =
    ThemeVar::new("toolbar.background", Color::TRANSPARENT);
/// [`BACKGROUND_VAR`] as a color for a style.
pub const BACKGROUND: Color = Color::var(&BACKGROUND_VAR);

/// The border of a toolbar.
pub static BORDER_VAR: ThemeVar<Color> = ThemeVar::new("toolbar.border", Color::TRANSPARENT);
/// [`BORDER_VAR`] as a color for a style.
pub const BORDER: Color = Color::var(&BORDER_VAR);

/// The padding of a toolbar, in logical px.
pub static PADDING_VAR: ThemeVar<f64> = ThemeVar::new("toolbar.padding", 8.0);
/// [`PADDING_VAR`] as a length for a style.
pub const PADDING: Length = Length::var(&PADDING_VAR);

/// The gap between a toolbar's items, in logical px.
pub static GAP_VAR: ThemeVar<f64> = ThemeVar::new("toolbar.gap", 4.0);
/// [`GAP_VAR`] as a length for a style.
pub const GAP: Length = Length::var(&GAP_VAR);

/// The minimum height of a toolbar, in logical px.
pub static MIN_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::new("toolbar.min_height", 32.0);
/// [`MIN_HEIGHT_VAR`] as a length for a style.
pub const MIN_HEIGHT: Length = Length::var(&MIN_HEIGHT_VAR);

pub(crate) fn root_style(
    _theme: &kivact_core::style::Theme,
    look: &crate::recipe::Look<ToolbarState>,
) -> kivact_core::style::Style {
    let vertical = look.orientation == Orientation::Vertical;
    let mut base = style()
        .background(BACKGROUND)
        .padding(PADDING)
        .min_height(MIN_HEIGHT)
        .width(pct(100))
        .align_items(if vertical {
            Align::Stretch
        } else {
            Align::Center
        })
        .flex_direction(if vertical {
            FlexDirection::Column
        } else if look.rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .justify_content(Justify::SpaceBetween);
    if look.centered && !vertical {
        base = base
            .display(kivact_core::style::Display::Grid)
            .grid_template_columns((
                Track::minmax(0, Track::fr(1)),
                Length::Auto,
                Track::minmax(0, Track::fr(1)),
            ));
    }
    for edge in &look.dividers {
        base = match edge {
            ToolbarEdge::Top => base.border_top(1, BORDER),
            ToolbarEdge::Bottom => base.border_bottom(1, BORDER),
            ToolbarEdge::Start if !look.rtl => base.border_left(1, BORDER),
            ToolbarEdge::End if look.rtl => base.border_left(1, BORDER),
            _ => base.border_right(1, BORDER),
        };
    }
    base
}

/// Layout passed to toolbar recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolbarState {
    pub orientation: Orientation,
    pub size: ControlSize,
    pub gap: Option<f64>,
    pub centered: bool,
    pub rtl: bool,
    pub dividers: Vec<ToolbarEdge>,
}

recipe_table! {
    /// Toolbar surface recipes.
    ToolbarTheme {
        root(root_recipe): ToolbarState = root_style,
    }
}

/// Logical edges on which a toolbar draws a divider.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarEdge {
    Top,
    Bottom,
    Start,
    End,
}
