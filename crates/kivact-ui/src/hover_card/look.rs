//! Role: The hover card recipe state, tokens, recipe table and default looks.

use crate::tooltip::look::tooltip;

use crate::recipe::recipe_table;
use kivact_core::style::{Style, Theme};
use kivact_core::view::Placement;
use kivact_interaction::PopupState;
pub(crate) fn hover_card(t: &Theme, _: &HoverCardSurfaceState) -> Style {
    tooltip(
        t,
        &PopupState {
            input: Default::default(),
            open: true,
        },
    )
}

/// The current input and requested placement exposed to preview recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoverCardSurfaceState {
    pub input: kivact_core::custom::PartState,
    pub open: bool,
    pub placement: Placement,
}

/// Per-theme geometry used by the preview overlay.
#[derive(Clone, Debug, PartialEq)]
pub struct HoverCardTokens {
    /// Logical pixel gap from the trigger.
    pub offset: f64,
    /// Logical pixel inset from the viewport edges.
    pub viewport_margin: f64,
}

impl Default for HoverCardTokens {
    fn default() -> Self {
        Self {
            offset: 4.0,
            viewport_margin: 8.0,
        }
    }
}

recipe_table! {
    /// Theme surface of a hover preview.
    HoverCardRecipes {
        surface(surface_recipe): HoverCardSurfaceState = hover_card,
    }
}
