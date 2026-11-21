//! Role: The toggle group recipe state, recipe table and default looks.

use crate::control_size::ControlSize;
use crate::recipe::recipe_table;
use crate::toggle_button::look::ToggleButtonState;
use kivact_core::style::{Align, FlexDirection, style};
use kivact_core::style::{Length, ThemeVar};
use kivact_core::view::Orientation;

/// The gap between the items of a toggle group, in logical px.
pub static GAP_VAR: ThemeVar<f64> = ThemeVar::new("toggle_group.gap", 4.0);
/// [`GAP_VAR`] as a length for a style.
pub const GAP: Length = Length::var(&GAP_VAR);

pub(crate) fn toggle_root(
    theme: &kivact_core::style::Theme,
    look: &crate::variants::Look<ToggleGroupState>,
) -> kivact_core::style::Style {
    style()
        .flex_direction(if look.orientation == Orientation::Vertical {
            FlexDirection::Column
        } else if look.rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Stretch)
        .gap(
            look.spacing
                .map_or(theme.get(&GAP_VAR), |spacing| spacing * 4.0),
        )
}

pub(crate) fn toggle_item(
    theme: &kivact_core::style::Theme,
    look: &crate::variants::Look<ToggleGroupItemState>,
) -> kivact_core::style::Style {
    crate::toggle_button::look::toggle_button(theme, &crate::variants::Look::new(look.toggle))
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
    ToggleGroupTheme {
        root(root_recipe): ToggleGroupState = toggle_root,
        item(item_recipe): ToggleGroupItemState = toggle_item,
    }
}
