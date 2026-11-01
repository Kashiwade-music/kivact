//! Role: Theme tokens of the Creative kit: accent, palette, density and label tones.

use kivact::style::{Color, Theme, ThemeExtension, ThemeMode, rgb, rgba};

/// The five accent choices in the original creative-tool catalog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Accent {
    #[default]
    Iris,
    Lime,
    Aqua,
    Coral,
    Amber,
}

impl Accent {
    /// Primary, highlight and readable foreground colors.
    pub fn colors(self) -> (Color, Color, Color) {
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

/// The color of a label, resolved against the installed [`Tokens`] when
/// drawn, so a label follows a theme switch.
///
/// Application components resolve it with [`LabelTone::color`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LabelTone {
    /// The ordinary label color (`Tokens::text`).
    #[default]
    Normal,
    /// De-emphasized text (`Tokens::muted`).
    Muted,
    /// The accent color (`Tokens::accent_high`).
    Accent,
    /// A warning state (`Tokens::warning`).
    Warning,
    /// An error state (`Tokens::x`).
    Error,
    /// A color chosen by the application, independent of the palette.
    Custom(Color),
}

impl LabelTone {
    /// The color this tone resolves to in `palette`.
    pub fn color(self, palette: &Tokens) -> Color {
        match self {
            Self::Normal => palette.text,
            Self::Muted => palette.muted,
            Self::Accent => palette.accent_high,
            Self::Warning => palette.warning,
            Self::Error => palette.x,
            Self::Custom(color) => color,
        }
    }
}

/// Control-row density, independent of display scale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}

/// Typed theme tokens, also usable by application-authored components.
#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    /// A caller-selected body font family. The default needs no bundled assets.
    pub body_font: String,
    /// A caller-selected numeric/code font family.
    pub mono_font: String,
    pub canvas: Color,
    pub island: Color,
    pub header: Color,
    pub strong: Color,
    pub field: Color,
    pub hair: Color,
    pub hair_strong: Color,
    pub text: Color,
    pub text_high: Color,
    pub muted: Color,
    pub dim: Color,
    pub accent: Color,
    pub accent_high: Color,
    pub accent_ink: Color,
    pub selection: Color,
    pub key: Color,
    pub x: Color,
    pub y: Color,
    pub z: Color,
    pub success: Color,
    pub warning: Color,
    pub row: f32,
    pub gap: f32,
}

impl Tokens {
    /// Match the source CSS tokens for a mode, an accent and a density. The
    /// source is dark; the light tables keep its accents and metrics on
    /// light surfaces.
    pub fn new(mode: ThemeMode, accent: Accent, density: Density) -> Self {
        let dark = mode == ThemeMode::Dark;
        let pick = |dark_value: u32, light: u32| rgb(if dark { dark_value } else { light });
        let ink = |alpha_dark: f32, alpha_light: f32| {
            if dark {
                rgba(0xffffff, alpha_dark)
            } else {
                rgba(0x000000, alpha_light)
            }
        };
        Self {
            body_font: "sans-serif".into(),
            mono_font: "monospace".into(),
            canvas: pick(0x0a0b0d, 0xeceef1),
            island: pick(0x131418, 0xffffff),
            header: pick(0x16171c, 0xf5f6f8),
            strong: pick(0x2c2e36, 0xd5d7dd),
            field: pick(0x1d1e24, 0xf1f2f5),
            hair: ink(0.065, 0.08),
            hair_strong: ink(0.16, 0.18),
            text: pick(0xe4e5e9, 0x23252c),
            text_high: pick(0xfafafb, 0x0a0b0d),
            muted: pick(0x9b9ea8, 0x5d606b),
            dim: pick(0x686b75, 0x8c8f99),
            accent: rgb(0),
            accent_high: rgb(0),
            accent_ink: rgb(0),
            selection: rgb(0),
            key: pick(0xffd25a, 0xc98a00),
            x: pick(0xff5c6c, 0xe5394b),
            y: pick(0x8fd94a, 0x4f9a1c),
            z: pick(0x4d9bff, 0x2f6fe0),
            success: pick(0x34d399, 0x16a34a),
            warning: pick(0xfbbf24, 0xd97706),
            row: 0.0,
            gap: 0.0,
        }
        .with_accent(accent)
        .with_density(density)
    }

    /// These tokens in another accent: the accent colors and the selection.
    pub(crate) fn with_accent(mut self, accent: Accent) -> Self {
        (self.accent, self.accent_high, self.accent_ink) = accent.colors();
        self.selection = self.accent.with_alpha(0.2);
        self
    }

    /// These tokens in another density: the row height and the gap.
    pub(crate) fn with_density(mut self, density: Density) -> Self {
        (self.row, self.gap) = match density {
            Density::Comfortable => (24.0, 8.0),
            Density::Compact => (20.0, 6.0),
        };
        self
    }
}

impl ThemeExtension for Tokens {
    /// The default accent and density in the theme's mode.
    fn fallback(theme: &Theme) -> Self {
        Self::new(theme.mode(), Accent::Iris, Density::Comfortable)
    }
}
