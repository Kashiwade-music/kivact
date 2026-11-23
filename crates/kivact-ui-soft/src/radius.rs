//! Role: The corner radii of Astryx Neutral as theme variables, in logical px.
use kivact::style::{Length, Theme, ThemeVar};

/// No rounding: 0 px.
pub(crate) static RADIUS_NONE_VAR: ThemeVar<f64> = ThemeVar::new("soft.radius_none", 0.0);
/// The radius of an inner surface: 6 px.
pub(crate) static RADIUS_INNER_VAR: ThemeVar<f64> = ThemeVar::new("soft.radius_inner", 6.0);
/// The radius of an element: 10 px.
pub(crate) static RADIUS_ELEMENT_VAR: ThemeVar<f64> = ThemeVar::new("soft.radius_element", 10.0);
/// The radius of a container: 12 px.
pub(crate) static RADIUS_CONTAINER_VAR: ThemeVar<f64> =
    ThemeVar::new("soft.radius_container", 12.0);
/// The radius of a page: 28 px.
pub(crate) static RADIUS_PAGE_VAR: ThemeVar<f64> = ThemeVar::new("soft.radius_page", 28.0);
/// The radius of a chat bubble: 28 px.
pub(crate) static RADIUS_CHAT_VAR: ThemeVar<f64> = ThemeVar::new("soft.radius_chat", 28.0);
/// The radius of a fully round shape: 9999 px.
pub(crate) static RADIUS_FULL_VAR: ThemeVar<f64> = ThemeVar::new("soft.radius_full", 9999.0);

/// No rounding.
pub const RADIUS_NONE: Length = Length::var(&RADIUS_NONE_VAR);
/// The radius of an inner surface.
pub const RADIUS_INNER: Length = Length::var(&RADIUS_INNER_VAR);
/// The radius of an element.
pub const RADIUS_ELEMENT: Length = Length::var(&RADIUS_ELEMENT_VAR);
/// The radius of a container.
pub const RADIUS_CONTAINER: Length = Length::var(&RADIUS_CONTAINER_VAR);
/// The radius of a page.
pub const RADIUS_PAGE: Length = Length::var(&RADIUS_PAGE_VAR);
/// The radius of a chat bubble.
pub const RADIUS_CHAT: Length = Length::var(&RADIUS_CHAT_VAR);
/// A fully round shape.
pub const RADIUS_FULL: Length = Length::var(&RADIUS_FULL_VAR);

/// Set every radius variable of the kit. The radii do not depend on the appearance.
pub(crate) fn set_radius(theme: Theme) -> Theme {
    theme
        .set(&RADIUS_NONE_VAR, 0.0)
        .set(&RADIUS_INNER_VAR, 6.0)
        .set(&RADIUS_ELEMENT_VAR, 10.0)
        .set(&RADIUS_CONTAINER_VAR, 12.0)
        .set(&RADIUS_PAGE_VAR, 28.0)
        .set(&RADIUS_CHAT_VAR, 28.0)
        .set(&RADIUS_FULL_VAR, 9999.0)
}
