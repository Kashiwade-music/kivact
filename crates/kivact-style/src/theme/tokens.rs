//! Role: Theme token tables: palette, spacing, radii and typography.

use crate::color::{Color, ColorToken};
use kivact_types::LineHeight;

/// The theme's colors, one per [`ColorToken`].
///
/// Beside the surfaces and texts it has the state and surface tokens
/// `Hover` (a tint laid over a surface under the pointer; it may be
/// translucent, so it works on any background), `Pressed` (stronger than
/// `Hover`) and `Raised` (the background of a floating surface: a menu, a
/// popover, a tooltip). Their defaults derive from `Surface` and `Text`:
/// in the light theme `Hover` and `Pressed` are `Text` at 5% and 10% (over
/// `Surface` about `SurfaceAlt`), in the dark theme at 7.5% and 15%;
/// `Raised` is `Surface` in the light theme (its shadow lifts it) and
/// `SurfaceAlt` in the dark one (where a shadow does not show, it is one
/// step lighter). A kit's theme fills these three when it maps its colors
/// to the core palette, and keeps no token of its own with the same
/// meaning.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    pub surface: Color,
    pub surface_alt: Color,
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_text: Color,
    pub border: Color,
    pub focus: Color,
    pub danger: Color,
    pub success: Color,
    pub warning: Color,
    pub overlay: Color,
    pub shadow: Color,
    pub selection: Color,
    /// [`ColorToken::Hover`]: a tint over a surface under the pointer.
    pub hover: Color,
    /// [`ColorToken::Pressed`]: a tint over a pressed surface.
    pub pressed: Color,
    /// [`ColorToken::Raised`]: the background of a menu, popover or tooltip.
    pub raised: Color,
}

impl Palette {
    pub fn get(&self, t: ColorToken) -> Color {
        match t {
            ColorToken::Surface => self.surface,
            ColorToken::SurfaceAlt => self.surface_alt,
            ColorToken::Text => self.text,
            ColorToken::TextMuted => self.text_muted,
            ColorToken::Accent => self.accent,
            ColorToken::AccentText => self.accent_text,
            ColorToken::Border => self.border,
            ColorToken::Focus => self.focus,
            ColorToken::Danger => self.danger,
            ColorToken::Success => self.success,
            ColorToken::Warning => self.warning,
            ColorToken::Overlay => self.overlay,
            ColorToken::Shadow => self.shadow,
            ColorToken::Selection => self.selection,
            ColorToken::Hover => self.hover,
            ColorToken::Pressed => self.pressed,
            ColorToken::Raised => self.raised,
        }
    }
}

/// Spacing scale (logical px).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radii {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typography {
    /// A family name, generic family, or ordered comma-separated CSS list.
    pub family: Option<String>,
    pub size: f32,
    pub size_small: f32,
    pub size_large: f32,
    pub weight: u16,
    pub weight_strong: u16,
    /// The line height of text that sets none: the root's text style
    /// carries it and every text below inherits it, as it does the family
    /// (a `line_height` in a style wins). A number is a multiple of the
    /// font size, as CSS's unitless `line-height`.
    pub line_height: LineHeight,
}
