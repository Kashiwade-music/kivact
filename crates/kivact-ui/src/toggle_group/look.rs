//! Role: The toggle group recipe state, tokens, recipe table and default looks.

use crate::control_size::ControlSize;
use crate::recipe::recipe_table;
use crate::toggle_button::look::ToggleButtonState;
use kivact_core::style::{Align, Direction, style};
use kivact_core::view::Orientation;
pub(crate) fn toggle_root(
    theme: &kivact_core::style::Theme,
    look: &crate::variants::Look<ToggleGroupState>,
) -> kivact_core::style::Style {
    style()
        .direction(if look.orientation == Orientation::Vertical {
            Direction::Column
        } else if look.rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(Align::Stretch)
        .gap(look.spacing.map_or(
            theme.ext::<crate::tokens::UiTokens>().toggle_group.gap,
            |spacing| spacing * 4.0,
        ))
}

pub(crate) fn toggle_item(
    theme: &kivact_core::style::Theme,
    look: &crate::variants::Look<ToggleGroupItemState>,
) -> kivact_core::style::Style {
    crate::toggle_button::look::toggle_button(theme, &crate::variants::Look::new(look.toggle))
}

/// Passive spacing for toggle groups.
#[derive(Clone, Debug, PartialEq)]
pub struct ToggleGroupTokens {
    pub gap: f64,
}

impl Default for ToggleGroupTokens {
    fn default() -> Self {
        Self { gap: 4.0 }
    }
}

/// Layout supplied to the group recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleGroupState {
    pub orientation: Orientation,
    pub size: ControlSize,
    pub spacing: Option<f64>,
    pub rtl: bool,
}

/// A toggle's actual state together with its derived position and layout.
#[derive(Clone, Debug, PartialEq)]
pub struct ToggleGroupItemState {
    pub toggle: ToggleButtonState,
    pub layout: ToggleGroupState,
    pub first: bool,
    pub last: bool,
}

impl std::ops::Deref for ToggleGroupItemState {
    type Target = ToggleButtonState;
    fn deref(&self) -> &Self::Target {
        &self.toggle
    }
}

recipe_table! {
    /// Toggle group surface and item recipes.
    ToggleGroupRecipes {
        root(root_recipe): ToggleGroupState = toggle_root,
        item(item_recipe): ToggleGroupItemState = toggle_item,
    }
}
