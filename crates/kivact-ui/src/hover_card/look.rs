//! Role: The hover card recipe state, recipe table and default looks.

use crate::theme::Look;
use crate::tooltip::look::tooltip;

use crate::recipe::recipe_table;
use kivact_core::style::{Length, ThemeVar};
use kivact_core::style::{Style, Theme};
use kivact_core::view::Placement;
use kivact_interaction::PopupState;

/// The distance between a hover card and its trigger, in logical px.
pub static OFFSET_VAR: ThemeVar<f64> = ThemeVar::new("hover_card.offset", 4.0);
/// [`OFFSET_VAR`] as a length for a style.
pub const OFFSET: Length = Length::var(&OFFSET_VAR);

/// The margin a hover card keeps from the viewport's edges, in logical px.
pub static VIEWPORT_MARGIN_VAR: ThemeVar<f64> = ThemeVar::new("hover_card.viewport_margin", 8.0);
/// [`VIEWPORT_MARGIN_VAR`] as a length for a style.
pub const VIEWPORT_MARGIN: Length = Length::var(&VIEWPORT_MARGIN_VAR);

pub(crate) fn hover_card(t: &Theme, _: &Look<HoverCardSurfaceState>) -> Style {
    tooltip(
        t,
        &Look::new(PopupState {
            input: Default::default(),
            open: true,
        }),
    )
}

/// The current input and requested placement exposed to preview recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoverCardSurfaceState {
    pub input: kivact_core::custom::PartState,
    pub open: bool,
    pub placement: Placement,
}

recipe_table! {
    /// Theme surface of a hover preview.
    HoverCardTheme {
        surface(surface_recipe): HoverCardSurfaceState = hover_card,
    }
}
