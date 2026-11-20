//! Role: The tooltip recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::style::{Position, Style, Theme, style};
use kivact_interaction::PopupState;
pub(crate) fn tooltip(t: &Theme, _: &PopupState) -> Style {
    style()
        .padding_x(8)
        .padding_y(4)
        .background(kivact_core::style::color::SURFACE)
        .border(1, kivact_core::style::color::BORDER)
        .radius(t.get(&kivact_core::style::radius::SM_VAR))
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
}

/// No arrow: an empty box out of the layout.
pub(crate) fn no_arrow(_: &Theme, _: &TooltipArrowState) -> Style {
    style().position(Position::Absolute)
}

/// Where a tooltip's arrow points, for the tooltip's arrow recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TooltipArrowState {
    /// The side of the trigger the tooltip shows on (base-ui's
    /// `data-side`): `Top` when it is above the trigger, so the arrow sits
    /// at the tooltip's bottom edge. It follows the host's flip once the
    /// tooltip is placed; before that, it is the requested placement.
    pub side: kivact_core::geometry::Edge,
    /// Where the trigger's centre is along that edge, in logical pixels
    /// from the tooltip's left (`Top`, `Bottom`) or top (`Left`, `Right`)
    /// edge, within the tooltip.
    pub offset: f64,
}

recipe_table! {
    /// The recipes of a [`tooltip`](super::tooltip).
    TooltipRecipes {
        surface(surface_recipe): PopupState = tooltip,
        /// The arrow toward the trigger: an absolutely positioned box in the
        /// surface, given the side the tooltip shows on and where the
        /// trigger's centre is along that edge. The default draws no arrow.
        arrow(arrow_recipe): TooltipArrowState = no_arrow,
    }
}
