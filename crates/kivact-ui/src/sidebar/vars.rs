//! Role: The sidebar's theme variables. Their fallbacks are the core variables.

use kivact_core::style::{Color, Length, ThemeVar, color};

macro_rules! sidebar_color {
    ($var:ident, $name:ident, $init:expr, $doc:literal) => {
        #[doc = $doc]
        pub static $var: ThemeVar<Color> = $init;
        #[doc = $doc]
        pub const $name: Color = Color::var(&$var);
    };
}

sidebar_color!(
    BACKGROUND_VAR,
    BACKGROUND,
    ThemeVar::alias("ui.sidebar.background", &color::SURFACE_ALT_VAR),
    "The panel."
);
sidebar_color!(
    FOREGROUND_VAR,
    FOREGROUND,
    ThemeVar::alias("ui.sidebar.foreground", &color::TEXT_VAR),
    "Text on the panel."
);
sidebar_color!(
    PRIMARY_VAR,
    PRIMARY,
    ThemeVar::alias("ui.sidebar.primary", &color::ACCENT_VAR),
    "A primary mark on the panel."
);
sidebar_color!(
    PRIMARY_FOREGROUND_VAR,
    PRIMARY_FOREGROUND,
    ThemeVar::alias("ui.sidebar.primary_foreground", &color::ACCENT_TEXT_VAR),
    "Text on the primary mark."
);
sidebar_color!(
    ACCENT_VAR,
    ACCENT,
    ThemeVar::alias("ui.sidebar.accent", &color::HOVER_VAR),
    "The active and hovered row."
);
sidebar_color!(
    ACCENT_FOREGROUND_VAR,
    ACCENT_FOREGROUND,
    ThemeVar::alias("ui.sidebar.accent_foreground", &color::TEXT_VAR),
    "Text on the active row."
);
sidebar_color!(
    BORDER_VAR,
    BORDER,
    ThemeVar::alias("ui.sidebar.border", &color::BORDER_VAR),
    "The panel's edge and separators."
);
sidebar_color!(
    RING_VAR,
    RING,
    ThemeVar::alias("ui.sidebar.ring", &color::FOCUS_VAR),
    "The focus ring of rows."
);
sidebar_color!(
    PAGE_VAR,
    PAGE,
    ThemeVar::alias("ui.sidebar.page", &color::SURFACE_VAR),
    "The page beside an inset sidebar."
);

/// The width of an open sidebar.
pub static WIDTH_VAR: ThemeVar<f64> = ThemeVar::new("ui.sidebar.width", 256.0);
/// The width of an open sidebar.
pub const WIDTH: Length = Length::var(&WIDTH_VAR);
/// The width of an icon-collapsed sidebar.
pub static WIDTH_ICON_VAR: ThemeVar<f64> = ThemeVar::new("ui.sidebar.width_icon", 48.0);
/// The width of an icon-collapsed sidebar.
pub const WIDTH_ICON: Length = Length::var(&WIDTH_ICON_VAR);
/// The corner radius of rows.
pub static ITEM_RADIUS_VAR: ThemeVar<f64> = ThemeVar::new("ui.sidebar.item_radius", 0.0);
/// The corner radius of rows.
pub const ITEM_RADIUS: Length = Length::var(&ITEM_RADIUS_VAR);
