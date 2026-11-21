//! Role: The core color variables of a theme (`color::ACCENT`), with the light theme's values as fallbacks.

use crate::color_value::{Color, rgb, rgba};
use kivact_types::ThemeVar;

/// Declares each color variable (`X_VAR`, the `static`) and its value form
/// (`X`, the `const` a style takes).
macro_rules! color_vars {
    ($($(#[$doc:meta])* $var:ident / $value:ident = $name:literal, $fallback:expr;)*) => {$(
        $(#[$doc])*
        pub static $var: ThemeVar<Color> = ThemeVar::new($name, $fallback);
        $(#[$doc])*
        pub const $value: Color = Color::var(&$var);
    )*};
}

color_vars! {
    /// The page background.
    SURFACE_VAR / SURFACE = "surface", rgb(0xffffff);
    /// A secondary background (a header, a well).
    SURFACE_ALT_VAR / SURFACE_ALT = "surface_alt", rgb(0xf3f4f6);
    /// Body text.
    TEXT_VAR / TEXT = "text", rgb(0x111827);
    /// Secondary text.
    TEXT_MUTED_VAR / TEXT_MUTED = "text_muted", rgb(0x6b7280);
    /// The accent (primary actions, the selection).
    ACCENT_VAR / ACCENT = "accent", rgb(0x2563eb);
    /// Text on the accent.
    ACCENT_TEXT_VAR / ACCENT_TEXT = "accent_text", rgb(0xffffff);
    /// Borders and dividers.
    BORDER_VAR / BORDER = "border", rgb(0xd1d5db);
    /// The focus ring.
    FOCUS_VAR / FOCUS = "focus", rgb(0x2563eb);
    /// Destructive actions and errors.
    DANGER_VAR / DANGER = "danger", rgb(0xdc2626);
    /// Success.
    SUCCESS_VAR / SUCCESS = "success", rgb(0x16a34a);
    /// Warnings.
    WARNING_VAR / WARNING = "warning", rgb(0xd97706);
    /// The scrim behind a modal.
    OVERLAY_VAR / OVERLAY = "overlay", rgba(0x000000, 0.35);
    /// Shadows.
    SHADOW_VAR / SHADOW = "shadow", rgba(0x000000, 0.2);
    /// The band behind selected text.
    SELECTION_VAR / SELECTION = "selection", rgba(0x2563eb, 0.15);
    /// A tint over a surface under the pointer; translucent, so it works on any background.
    HOVER_VAR / HOVER = "hover", rgba(0x111827, 0.05);
    /// A tint over a pressed surface, stronger than `HOVER`.
    PRESSED_VAR / PRESSED = "pressed", rgba(0x111827, 0.10);
    /// The background of a menu, a popover or a tooltip.
    RAISED_VAR / RAISED = "raised", rgb(0xffffff);
}
