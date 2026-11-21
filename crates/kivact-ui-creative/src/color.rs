//! Role: The Creative kit's color variables and their light and dark values.
//!
//! Each `X_VAR` is a theme variable of the kit and `X` is the value a style
//! takes. The surface, text and border colors fall back to the core colors
//! (`kivact::style::color`), so a theme that sets the core colors moves them too.

use crate::options::Accent;
use kivact::style::{Color, Theme, ThemeMode, ThemeVar, color, rgb, rgba};

pub static CANVAS_VAR: ThemeVar<Color> = ThemeVar::new("creative.canvas", rgb(0xeceef1));
/// The island: the surface of a panel.
pub static ISLAND_VAR: ThemeVar<Color> = ThemeVar::alias("creative.island", &color::SURFACE_VAR);
pub static HEADER_VAR: ThemeVar<Color> = ThemeVar::new("creative.header", rgb(0xf5f6f8));
pub static STRONG_VAR: ThemeVar<Color> = ThemeVar::alias("creative.strong", &color::PRESSED_VAR);
pub static FIELD_VAR: ThemeVar<Color> = ThemeVar::new("creative.field", rgb(0xf1f2f5));
pub static HAIR_VAR: ThemeVar<Color> = ThemeVar::alias("creative.hair", &color::BORDER_VAR);
pub static HAIR_STRONG_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.hair_strong", rgba(0x000000, 0.18));
pub static TEXT_VAR: ThemeVar<Color> = ThemeVar::alias("creative.text", &color::TEXT_VAR);
pub static TEXT_HIGH_VAR: ThemeVar<Color> = ThemeVar::new("creative.text_high", rgb(0x0a0b0d));
pub static MUTED_VAR: ThemeVar<Color> = ThemeVar::alias("creative.muted", &color::TEXT_MUTED_VAR);
pub static DIM_VAR: ThemeVar<Color> = ThemeVar::new("creative.dim", rgb(0x8c8f99));
pub static ACCENT_VAR: ThemeVar<Color> = ThemeVar::new("creative.accent", rgb(0x8b7cff));
pub static ACCENT_HIGH_VAR: ThemeVar<Color> = ThemeVar::new("creative.accent_high", rgb(0xa397ff));
pub static ACCENT_INK_VAR: ThemeVar<Color> = ThemeVar::new("creative.accent_ink", rgb(0x0c0a1f));
pub static SELECTION_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.selection", rgba(0x8b7cff, 0.2));
pub static KEY_VAR: ThemeVar<Color> = ThemeVar::new("creative.key", rgb(0xc98a00));
pub static X_VAR: ThemeVar<Color> = ThemeVar::alias("creative.x", &color::DANGER_VAR);
pub static Y_VAR: ThemeVar<Color> = ThemeVar::new("creative.y", rgb(0x4f9a1c));
pub static Z_VAR: ThemeVar<Color> = ThemeVar::new("creative.z", rgb(0x2f6fe0));
pub static SUCCESS_VAR: ThemeVar<Color> = ThemeVar::alias("creative.success", &color::SUCCESS_VAR);
pub static WARNING_VAR: ThemeVar<Color> = ThemeVar::alias("creative.warning", &color::WARNING_VAR);

pub const CANVAS: Color = Color::var(&CANVAS_VAR);
pub const ISLAND: Color = Color::var(&ISLAND_VAR);
pub const HEADER: Color = Color::var(&HEADER_VAR);
pub const STRONG: Color = Color::var(&STRONG_VAR);
pub const FIELD: Color = Color::var(&FIELD_VAR);
pub const HAIR: Color = Color::var(&HAIR_VAR);
pub const HAIR_STRONG: Color = Color::var(&HAIR_STRONG_VAR);
pub const TEXT: Color = Color::var(&TEXT_VAR);
pub const TEXT_HIGH: Color = Color::var(&TEXT_HIGH_VAR);
pub const MUTED: Color = Color::var(&MUTED_VAR);
pub const DIM: Color = Color::var(&DIM_VAR);
pub const ACCENT: Color = Color::var(&ACCENT_VAR);
pub const ACCENT_HIGH: Color = Color::var(&ACCENT_HIGH_VAR);
pub const ACCENT_INK: Color = Color::var(&ACCENT_INK_VAR);
pub const SELECTION: Color = Color::var(&SELECTION_VAR);
pub const KEY: Color = Color::var(&KEY_VAR);
pub const X: Color = Color::var(&X_VAR);
pub const Y: Color = Color::var(&Y_VAR);
pub const Z: Color = Color::var(&Z_VAR);
pub const SUCCESS: Color = Color::var(&SUCCESS_VAR);
pub const WARNING: Color = Color::var(&WARNING_VAR);

// Fixed paint colors shared by Creative surfaces and initial domain values.
pub static TRANSPARENT_INK_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.transparent_ink", rgba(0x000000, 0.0));
pub static BUTTON_SURFACE_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.button_surface", rgba(0xffffff, 0.065));
pub static CONTROL_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.control_border", rgba(0xffffff, 0.04));
pub static OVERLAY_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.overlay_border", rgba(0xffffff, 0.1));
pub static BUTTON_PRESSED_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.button_pressed", rgba(0xffffff, 0.05));
pub static MENU_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.menu_hover", rgba(0xffffff, 0.07));
pub static CONTROL_SHADOW_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.control_shadow", rgba(0x000000, 0.28));
pub static OVERLAY_SURFACE_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.overlay_surface", rgba(0x1e1f26, 0.94));
pub static WINDOW_SHADOW_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.window_shadow", rgba(0x000000, 0.65));
pub static TRANSPARENT_BACKDROP_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.transparent_backdrop", rgba(0x050608, 0.0));
pub static MODAL_BACKDROP_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.modal_backdrop", rgba(0x050608, 0.35));
pub static DIALOG_SHADOW_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.dialog_shadow", rgba(0x000000, 0.5));
pub static MENU_SHADOW_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.menu_shadow", rgba(0x000000, 0.6));
pub static DEFAULT_ITEM_COLOR_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.default_item_color", rgb(0x8b7cff));
pub static DRIVEN_PROPERTY_COLOR_VAR: ThemeVar<Color> =
    ThemeVar::new("creative.driven_property_color", rgb(0xc084fc));

pub const TRANSPARENT_INK: Color = Color::var(&TRANSPARENT_INK_VAR);
pub const BUTTON_SURFACE: Color = Color::var(&BUTTON_SURFACE_VAR);
pub const CONTROL_BORDER: Color = Color::var(&CONTROL_BORDER_VAR);
pub const OVERLAY_BORDER: Color = Color::var(&OVERLAY_BORDER_VAR);
pub const BUTTON_PRESSED: Color = Color::var(&BUTTON_PRESSED_VAR);
pub const MENU_HOVER: Color = Color::var(&MENU_HOVER_VAR);
pub const CONTROL_SHADOW: Color = Color::var(&CONTROL_SHADOW_VAR);
pub const OVERLAY_SURFACE: Color = Color::var(&OVERLAY_SURFACE_VAR);
pub const WINDOW_SHADOW: Color = Color::var(&WINDOW_SHADOW_VAR);
pub const TRANSPARENT_BACKDROP: Color = Color::var(&TRANSPARENT_BACKDROP_VAR);
pub const MODAL_BACKDROP: Color = Color::var(&MODAL_BACKDROP_VAR);
pub const DIALOG_SHADOW: Color = Color::var(&DIALOG_SHADOW_VAR);
pub const MENU_SHADOW: Color = Color::var(&MENU_SHADOW_VAR);
pub const DEFAULT_ITEM_COLOR: Color = Color::var(&DEFAULT_ITEM_COLOR_VAR);
pub const DRIVEN_PROPERTY_COLOR: Color = Color::var(&DRIVEN_PROPERTY_COLOR_VAR);

pub(crate) fn checker_color(light: bool) -> Color {
    rgb(if light { 0x57585d } else { 0x272830 })
}

/// Set every color variable of the kit to its value in `mode` with `accent`.
pub(crate) fn set_colors(theme: Theme, mode: ThemeMode, accent: Accent) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let pick = |dark_value: u32, light: u32| rgb(if dark { dark_value } else { light });
    let ink = |alpha_dark: f64, alpha_light: f64| {
        if dark {
            rgba(0xffffff, alpha_dark)
        } else {
            rgba(0x000000, alpha_light)
        }
    };
    let (accent_color, accent_high, accent_ink) = accent.colors();
    theme
        .set(&CANVAS_VAR, pick(0x0a0b0d, 0xeceef1))
        .set(&ISLAND_VAR, pick(0x131418, 0xffffff))
        .set(&HEADER_VAR, pick(0x16171c, 0xf5f6f8))
        .set(&STRONG_VAR, pick(0x2c2e36, 0xd5d7dd))
        .set(&FIELD_VAR, pick(0x1d1e24, 0xf1f2f5))
        .set(&HAIR_VAR, ink(0.065, 0.08))
        .set(&HAIR_STRONG_VAR, ink(0.16, 0.18))
        .set(&TEXT_VAR, pick(0xe4e5e9, 0x23252c))
        .set(&TEXT_HIGH_VAR, pick(0xfafafb, 0x0a0b0d))
        .set(&MUTED_VAR, pick(0x9b9ea8, 0x5d606b))
        .set(&DIM_VAR, pick(0x686b75, 0x8c8f99))
        .set(&ACCENT_VAR, accent_color)
        .set(&ACCENT_HIGH_VAR, accent_high)
        .set(&ACCENT_INK_VAR, accent_ink)
        .set(&SELECTION_VAR, accent_color.with_alpha(0.2))
        .set(&KEY_VAR, pick(0xffd25a, 0xc98a00))
        .set(&X_VAR, pick(0xff5c6c, 0xe5394b))
        .set(&Y_VAR, pick(0x8fd94a, 0x4f9a1c))
        .set(&Z_VAR, pick(0x4d9bff, 0x2f6fe0))
        .set(&SUCCESS_VAR, pick(0x34d399, 0x16a34a))
        .set(&WARNING_VAR, pick(0xfbbf24, 0xd97706))
}

impl Accent {
    /// Primary, highlight and readable foreground colors.
    pub(crate) fn colors(self) -> (Color, Color, Color) {
        let (a, h, i) = match self {
            Self::Iris => (0x8b7cff, 0xa397ff, 0x0c0a1f),
            Self::Lime => (0xb8f04a, 0xccf779, 0x11180a),
            Self::Aqua => (0x3ad3e0, 0x6fe2ec, 0x04191b),
            Self::Coral => (0xff7a59, 0xff9a80, 0x210a04),
            Self::Amber => (0xffb224, 0xffc75c, 0x1f1402),
        };
        (rgb(a), rgb(h), rgb(i))
    }
}
