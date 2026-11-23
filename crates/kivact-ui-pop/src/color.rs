//! Role: The Pop kit's colors as theme variables, and the values each mode sets them to.
//!
//! The surface, text, accent, border and status colors alias the core
//! variables of `kivact::style::color`, so a theme that changes the core
//! palette changes the kit too, unless the kit's own value is set.

use kivact::style::{Color, Theme, ThemeMode, ThemeVar, color as core, rgb, rgba};

/// Declares each color variable (`X_VAR`, the `static`) and its value form
/// (`X`, the `const` a style takes).
macro_rules! color_vars {
    ($($(#[$doc:meta])* $var:ident / $value:ident = $name:literal, $fallback:expr;)*) => {$(
        $(#[$doc])*
        pub(crate) static $var: ThemeVar<Color> = ThemeVar::new($name, $fallback);
        $(#[$doc])*
        pub(crate) const $value: Color = Color::var(&$var);
    )*};
}

// These variables are set by the kit's colors but no component reads them by a value form.
/// The page background.
pub(crate) static BACKGROUND_VAR: ThemeVar<Color> = ThemeVar::new("pop.background", rgb(0xf8fafc));
/// The deep blue of a pressed accent.
pub(crate) static BLUE_DEEP_VAR: ThemeVar<Color> = ThemeVar::new("pop.blue_deep", rgb(0x1a66a8));
/// The orange accent.
pub(crate) static ORANGE_VAR: ThemeVar<Color> = ThemeVar::new("pop.orange", rgb(0xffa53c));
/// The purple accent.
pub(crate) static PURPLE_VAR: ThemeVar<Color> = ThemeVar::new("pop.purple", rgb(0x9d86ff));

color_vars! {
    /// A muted surface (a table header, a well).
    SURFACE_MUTED_VAR / SURFACE_MUTED = "pop.surface_muted", rgb(0xf8fafc);
    /// Strong lines and the dark track of a toggle.
    LINE_STRONG_VAR / LINE_STRONG = "pop.line_strong", rgb(0xcbd5e1);
    /// Faint text (a placeholder, a disabled label).
    TEXT_FAINT_VAR / TEXT_FAINT = "pop.text_faint", rgb(0x94a3b8);
    /// The cyan end of the blue gradient.
    CYAN_VAR / CYAN = "pop.cyan", rgb(0x5cbde8);
    /// The pink accent.
    PINK_VAR / PINK = "pop.pink", rgb(0xf2559a);
    /// The navy of the dark button.
    NAVY_VAR / NAVY = "pop.navy", rgb(0x1d2b4a);
    /// The background of a blue tag, a blue card.
    BLUE_BACKGROUND_VAR / BLUE_BACKGROUND = "pop.blue_background", rgb(0xe9f2fb);
    /// The text of a blue tag.
    BLUE_FOREGROUND_VAR / BLUE_FOREGROUND = "pop.blue_foreground", rgb(0x1f6fb8);
    /// The background of a mint tag.
    MINT_BACKGROUND_VAR / MINT_BACKGROUND = "pop.mint_background", rgb(0xe5f6f0);
    /// The text of a mint tag.
    MINT_FOREGROUND_VAR / MINT_FOREGROUND = "pop.mint_foreground", rgb(0x0f8a6a);
    /// The background of a yellow tag.
    YELLOW_BACKGROUND_VAR / YELLOW_BACKGROUND = "pop.yellow_background", rgb(0xfdf4d6);
    /// The text of a yellow tag.
    YELLOW_FOREGROUND_VAR / YELLOW_FOREGROUND = "pop.yellow_foreground", rgb(0x8a6a00);
    /// The background of a pink tag.
    PINK_BACKGROUND_VAR / PINK_BACKGROUND = "pop.pink_background", rgb(0xfce8f1);
    /// The text of a pink tag.
    PINK_FOREGROUND_VAR / PINK_FOREGROUND = "pop.pink_foreground", rgb(0xc8396f);
    /// The background of a red tag.
    RED_BACKGROUND_VAR / RED_BACKGROUND = "pop.red_background", rgb(0xfdeaec);
    /// The text of a red tag.
    RED_FOREGROUND_VAR / RED_FOREGROUND = "pop.red_foreground", rgb(0xd22c43);
    /// The background of a gray tag.
    GRAY_BACKGROUND_VAR / GRAY_BACKGROUND = "pop.gray_background", rgb(0xf1f5f9);
    /// The text of a gray tag.
    GRAY_FOREGROUND_VAR / GRAY_FOREGROUND = "pop.gray_foreground", rgb(0x475569);
    /// The end of the blue button's gradient: the blue mixed toward cyan.
    BLUE_GRADIENT_END_VAR / BLUE_GRADIENT_END = "pop.blue_gradient_end", rgb(0x2b8bd8);
}

/// The cards and panels of the kit: the core surface unless the theme sets it.
pub static SURFACE_VAR: ThemeVar<Color> = ThemeVar::alias("pop.surface", &core::SURFACE_VAR);
/// The cards and panels of the kit.
pub const SURFACE: Color = Color::var(&SURFACE_VAR);
/// Hovered and selected surfaces: the core alternate surface unless set.
pub(crate) static SURFACE_ACTIVE_VAR: ThemeVar<Color> =
    ThemeVar::alias("pop.surface_active", &core::SURFACE_ALT_VAR);
/// Hovered and selected surfaces.
pub(crate) const SURFACE_ACTIVE: Color = Color::var(&SURFACE_ACTIVE_VAR);
/// Body text: the core text unless set.
pub static TEXT_VAR: ThemeVar<Color> = ThemeVar::alias("pop.text", &core::TEXT_VAR);
/// Body text.
pub const TEXT: Color = Color::var(&TEXT_VAR);
/// Secondary text: the core muted text unless set.
pub static TEXT_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::alias("pop.text_muted", &core::TEXT_MUTED_VAR);
/// Secondary text.
pub const TEXT_MUTED: Color = Color::var(&TEXT_MUTED_VAR);
/// Hairlines and borders: the core border unless set.
pub static LINE_VAR: ThemeVar<Color> = ThemeVar::alias("pop.line", &core::BORDER_VAR);
/// Hairlines and borders.
pub const LINE: Color = Color::var(&LINE_VAR);
/// The blue accent: the core accent unless set.
pub static BLUE_VAR: ThemeVar<Color> = ThemeVar::alias("pop.blue", &core::ACCENT_VAR);
/// The blue accent.
pub const BLUE: Color = Color::var(&BLUE_VAR);
/// The red status color: the core danger unless set.
pub static RED_VAR: ThemeVar<Color> = ThemeVar::alias("pop.red", &core::DANGER_VAR);
/// The red status color.
pub const RED: Color = Color::var(&RED_VAR);
/// The mint status color: the core success unless set.
pub static MINT_VAR: ThemeVar<Color> = ThemeVar::alias("pop.mint", &core::SUCCESS_VAR);
/// The mint status color.
pub const MINT: Color = Color::var(&MINT_VAR);
/// The yellow status color: the core warning unless set.
pub static YELLOW_VAR: ThemeVar<Color> = ThemeVar::alias("pop.yellow", &core::WARNING_VAR);
/// The yellow status color.
pub const YELLOW: Color = Color::var(&YELLOW_VAR);

/// The accent gradient's end (constant in both modes).
pub(crate) static ACCENT_GRADIENT_END_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.accent_gradient_end", rgb(0xffe36a));
/// The accent gradient's end, constant in both modes.
pub(crate) const ACCENT_GRADIENT_END: Color = Color::var(&ACCENT_GRADIENT_END_VAR);
/// The text on the yellow accent (constant in both modes).
pub(crate) static ACCENT_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.accent_text", rgb(0x3b2f00));
/// The text on the yellow accent, constant in both modes.
pub(crate) const ACCENT_TEXT: Color = Color::var(&ACCENT_TEXT_VAR);
/// The pink gradient's end (constant in both modes).
pub(crate) static PINK_GRADIENT_END_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.pink_gradient_end", rgb(0xff7cb0));
/// The pink gradient's end, constant in both modes.
pub(crate) const PINK_GRADIENT_END: Color = Color::var(&PINK_GRADIENT_END_VAR);
/// The danger gradient's end (constant in both modes).
pub(crate) static DANGER_GRADIENT_END_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.danger_gradient_end", rgb(0xf45d6f));
/// The danger gradient's end, constant in both modes.
pub(crate) const DANGER_GRADIENT_END: Color = Color::var(&DANGER_GRADIENT_END_VAR);
/// The shadow under a control (constant in both modes).
pub(crate) static CONTROL_SHADOW_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.control_shadow", rgba(0x0f172a, 0.12));
/// The shadow under a control, constant in both modes.
pub(crate) const CONTROL_SHADOW: Color = Color::var(&CONTROL_SHADOW_VAR);
/// The faint shadow under a card (constant in both modes).
pub(crate) static SUBTLE_SHADOW_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.subtle_shadow", rgba(0x0f172a, 0.05));
/// The faint shadow under a card, constant in both modes.
pub(crate) const SUBTLE_SHADOW: Color = Color::var(&SUBTLE_SHADOW_VAR);
/// The fill of a glass button (constant in both modes).
pub(crate) static GLASS_SURFACE_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.glass_surface", rgba(0xffffff, 0.14));
/// The fill of a glass button, constant in both modes.
pub(crate) const GLASS_SURFACE: Color = Color::var(&GLASS_SURFACE_VAR);
/// The border of a glass button (constant in both modes).
pub(crate) static GLASS_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.glass_border", rgba(0xffffff, 0.4));
/// The border of a glass button, constant in both modes.
pub(crate) const GLASS_BORDER: Color = Color::var(&GLASS_BORDER_VAR);
/// The hovered fill of a glass button (constant in both modes).
pub(crate) static GLASS_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.glass_hover", rgba(0xffffff, 0.24));
/// The hovered fill of a glass button, constant in both modes.
pub(crate) const GLASS_HOVER: Color = Color::var(&GLASS_HOVER_VAR);
/// The fill of a selected segment of a pill choice, set per mode by [`set_colors`].
pub(crate) static SELECTED_SEGMENT_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.selected_segment", rgb(0xffffff));
/// The fill of a selected segment of a pill choice.
pub(crate) const SELECTED_SEGMENT: Color = Color::var(&SELECTED_SEGMENT_VAR);
/// The fill under a sparkline (constant in both modes).
pub(crate) static SPARKLINE_FILL_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.sparkline_fill", rgba(0x2b8bd8, 0.32));
/// The fill under a sparkline, constant in both modes.
pub(crate) const SPARKLINE_FILL: Color = Color::var(&SPARKLINE_FILL_VAR);
/// The yellow gradient's end (constant in both modes).
pub(crate) static YELLOW_GRADIENT_END_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.yellow_gradient_end", rgb(0xffe98a));
/// The yellow gradient's end, constant in both modes.
pub(crate) const YELLOW_GRADIENT_END: Color = Color::var(&YELLOW_GRADIENT_END_VAR);
/// The red gradient's end (constant in both modes).
pub(crate) static RED_GRADIENT_END_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.red_gradient_end", rgb(0xff9c6e));
/// The red gradient's end, constant in both modes.
pub(crate) const RED_GRADIENT_END: Color = Color::var(&RED_GRADIENT_END_VAR);
/// The scrim behind a dialog (constant in both modes).
pub(crate) static OVERLAY_SCRIM_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.overlay_scrim", rgba(0x0f172a, 0.45));
/// The scrim behind a dialog, constant in both modes.
pub(crate) const OVERLAY_SCRIM: Color = Color::var(&OVERLAY_SCRIM_VAR);
/// The stripes of a striped progress bar (constant in both modes).
pub(crate) static PROGRESS_STRIPE_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.progress_stripe", rgba(0xffffff, 0.3));
/// The stripes of a striped progress bar, constant in both modes.
pub(crate) const PROGRESS_STRIPE: Color = Color::var(&PROGRESS_STRIPE_VAR);

/// The hovered ghost-like button's fill: the surface active mixed toward the text.
pub(crate) static SURFACE_ACTIVE_BUTTON_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.surface_active_button_hover", rgb(0xf1f5f9));
/// The hovered fill of a soft button, set per mode by [`set_colors`].
pub(crate) const SURFACE_ACTIVE_BUTTON_HOVER: Color = Color::var(&SURFACE_ACTIVE_BUTTON_HOVER_VAR);
/// The border of a red alert: the red foreground mixed toward the line.
pub(crate) static RED_FOREGROUND_ALERT_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("pop.red_foreground_alert_border", rgb(0xd22c43));
/// The border of a red alert, set per mode by [`set_colors`].
pub(crate) const RED_FOREGROUND_ALERT_BORDER: Color = Color::var(&RED_FOREGROUND_ALERT_BORDER_VAR);
/// Set every color variable of the kit to its value in `mode`, and the
/// derived colors computed from them.
pub(crate) fn set_colors(theme: Theme, mode: ThemeMode) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let theme = set_surfaces(theme, dark);
    set_hues(theme, dark)
}

/// Set the surface, text, line and accent variables.
fn set_surfaces(theme: Theme, dark: bool) -> Theme {
    let pick = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
    theme
        .set(&BACKGROUND_VAR, pick(0xf8fafc, 0x0b1120))
        .set(&SURFACE_VAR, pick(0xffffff, 0x0f172a))
        .set(&SURFACE_MUTED_VAR, pick(0xf8fafc, 0x131c31))
        .set(&SURFACE_ACTIVE_VAR, pick(0xf1f5f9, 0x1e293b))
        .set(&SELECTED_SEGMENT_VAR, pick(0xffffff, 0x273449))
        .set(&LINE_VAR, pick(0xe2e8f0, 0x1e293b))
        .set(&LINE_STRONG_VAR, pick(0xcbd5e1, 0x334155))
        .set(&TEXT_VAR, pick(0x0f172a, 0xf1f5f9))
        .set(&TEXT_MUTED_VAR, pick(0x64748b, 0x94a3b8))
        .set(&TEXT_FAINT_VAR, pick(0x94a3b8, 0x64748b))
        .set(&BLUE_VAR, rgb(0x2b8bd8))
        .set(&BLUE_DEEP_VAR, rgb(0x1a66a8))
        .set(&CYAN_VAR, rgb(0x5cbde8))
        .set(&YELLOW_VAR, rgb(0xffdc3c))
        .set(&PINK_VAR, rgb(0xf2559a))
        .set(&MINT_VAR, rgb(0x2fcfa6))
        .set(&RED_VAR, rgb(0xef4458))
        .set(&ORANGE_VAR, rgb(0xffa53c))
        .set(&PURPLE_VAR, rgb(0x9d86ff))
        .set(&NAVY_VAR, rgb(0x1d2b4a))
}

/// Set the tag colors, the gradient end and the derived shades.
fn set_hues(theme: Theme, dark: bool) -> Theme {
    let tone = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
    let tint = |light: u32, dark_value: (u32, f64)| {
        if dark {
            rgba(dark_value.0, dark_value.1)
        } else {
            rgb(light)
        }
    };
    // The blue gradient's end: the blue mixed toward cyan, computed from the
    // blue of the mode before the theme is set.
    let blue_end = rgb(0x2b8bd8).mix(rgb(0x5cbde8), 0.3, &theme);
    let hover = tone(0xf1f5f9, 0x1e293b).mix(tone(0x0f172a, 0xf1f5f9), 0.15, &theme);
    let alert_border = tone(0xd22c43, 0xff8a98).mix(tone(0xe2e8f0, 0x1e293b), 0.7, &theme);
    theme
        .set(&SURFACE_ACTIVE_BUTTON_HOVER_VAR, hover)
        .set(&RED_FOREGROUND_ALERT_BORDER_VAR, alert_border)
        .set(&BLUE_GRADIENT_END_VAR, blue_end)
        .set(&BLUE_BACKGROUND_VAR, tint(0xe9f2fb, (0x2b8bd8, 0.16)))
        .set(&BLUE_FOREGROUND_VAR, tone(0x1f6fb8, 0x7cc0f0))
        .set(&MINT_BACKGROUND_VAR, tint(0xe5f6f0, (0x2fcfa6, 0.14)))
        .set(&MINT_FOREGROUND_VAR, tone(0x0f8a6a, 0x5ddcb9))
        .set(&YELLOW_BACKGROUND_VAR, tint(0xfdf4d6, (0xffdc3c, 0.12)))
        .set(&YELLOW_FOREGROUND_VAR, tone(0x8a6a00, 0xf5d457))
        .set(&PINK_BACKGROUND_VAR, tint(0xfce8f1, (0xff7cb0, 0.14)))
        .set(&PINK_FOREGROUND_VAR, tone(0xc8396f, 0xff9cc4))
        .set(&RED_BACKGROUND_VAR, tint(0xfdeaec, (0xef4458, 0.16)))
        .set(&RED_FOREGROUND_VAR, tone(0xd22c43, 0xff8a98))
        .set(&GRAY_BACKGROUND_VAR, tint(0xf1f5f9, (0x94a3b8, 0.12)))
        .set(&GRAY_FOREGROUND_VAR, tone(0x475569, 0xa3b1c6))
}

/// The halo ring of an avatar in `hue` degrees.
pub(crate) fn avatar_halo(hue: f64) -> Color {
    kivact::style::hsl(hue, 0.95, 0.62)
}

/// The glow under an avatar's halo in `hue` degrees.
pub(crate) fn avatar_glow(hue: f64) -> Color {
    kivact::style::hsl(hue, 1.0, 0.65)
}

/// The start of an avatar's gradient in `hue` degrees.
pub(crate) fn avatar_gradient_start(hue: f64) -> Color {
    kivact::style::hsl(hue, 0.85, 0.7)
}

/// The end of an avatar's gradient in `hue` degrees.
pub(crate) fn avatar_gradient_end(hue: f64) -> Color {
    kivact::style::hsl(hue + 18.0, 0.75, 0.52)
}
