//! Role: The toolbar recipe state, tokens, recipe table and default looks.

use crate::control_size::ControlSize;
use crate::recipe::recipe_table;

use kivact_core::style::{Align, Color, Direction, Justify, Length, Track, pct, style};
use kivact_core::view::Orientation;
pub(crate) fn root_style(
    theme: &kivact_core::style::Theme,
    look: &crate::variants::Look<ToolbarState>,
) -> kivact_core::style::Style {
    let t = &theme.ext::<crate::tokens::UiTokens>().toolbar;
    let vertical = look.orientation == Orientation::Vertical;
    let mut base = style()
        .background(t.background)
        .padding(t.padding)
        .min_height(t.min_height)
        .width(pct(100))
        .align_items(if vertical {
            Align::Stretch
        } else {
            Align::Center
        })
        .direction(if vertical {
            Direction::Column
        } else if look.rtl {
            Direction::RowReverse
        } else {
            Direction::Row
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
            ToolbarEdge::Top => base.border_top(1, t.border),
            ToolbarEdge::Bottom => base.border_bottom(1, t.border),
            ToolbarEdge::Start if !look.rtl => base.border_left(1, t.border),
            ToolbarEdge::End if look.rtl => base.border_left(1, t.border),
            _ => base.border_right(1, t.border),
        };
    }
    base
}

/// Passive toolbar colors and metrics supplied through [`crate::UiTokens`].
#[derive(Clone, Debug, PartialEq)]
pub struct ToolbarTokens {
    pub background: Color,
    pub border: Color,
    pub padding: f64,
    pub gap: f64,
    pub min_height: f64,
}

impl Default for ToolbarTokens {
    fn default() -> Self {
        Self {
            background: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            padding: 8.0,
            gap: 4.0,
            min_height: 32.0,
        }
    }
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
    ToolbarRecipes {
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
