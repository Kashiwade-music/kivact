//! Role: Prototype color tables and metrics carried by Kivact's public theme.
use kivact::style::{Color, Theme, ThemeExtension, ThemeMode, rgb, rgba};

/// Comfortable or compact prototype metrics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}

/// Additional operation-console tokens. [`theme`](crate::theme) installs
/// them; components read them with `theme.ext::<Tokens>()`, and a theme
/// without them reads the comfortable values of its [`ThemeMode`].
#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    pub display_family: String,
    pub mono_family: String,
    pub mode: ThemeMode,
    pub density: Density,
    pub primary_hover: Color,
    pub primary_strong: Color,
    pub primary_weak: Color,
    pub tape_stripe: Color,
    pub dropzone_stripe: Color,
    pub cyan: Color,
    pub yellow: Color,
    pub navy: Color,
    pub canvas: Color,
    pub surface_3: Color,
    pub line_strong: Color,
    pub text_2: Color,
    pub text_3: Color,
    pub sidebar: Color,
    pub sidebar_alt: Color,
    pub sidebar_text: Color,
    pub sidebar_dim: Color,
    pub control_height: f64,
    pub row_height: f64,
    pub padding: f64,
}

impl Tokens {
    /// The prototype's tables for a mode and a density.
    pub fn new(mode: ThemeMode, density: Density) -> Self {
        let mut tokens = Tokens {
            mode,
            density,
            ..Tokens::light()
        };
        if mode == ThemeMode::Dark {
            tokens.primary_hover = rgb(0x469ff5);
            tokens.primary_strong = rgb(0x1a6cc4);
            tokens.primary_weak = rgba(0x3a9dff, 0.15);
            tokens.navy = rgb(0x1a3150);
            tokens.canvas = rgb(0x08111c);
            tokens.surface_3 = rgb(0x192d45);
            tokens.line_strong = rgb(0x33506f);
            tokens.text_2 = rgb(0x9db1c8);
            tokens.text_3 = rgb(0x6880a0);
            tokens.sidebar = rgb(0x050d17);
            tokens.sidebar_alt = rgb(0x0b1a2c);
        }
        if density == Density::Compact {
            tokens.control_height = 28.;
            tokens.row_height = 32.;
            tokens.padding = 14.;
        }
        tokens
    }
}

impl ThemeExtension for Tokens {
    /// The comfortable values of the theme's mode.
    fn fallback(theme: &Theme) -> Self {
        Tokens::new(theme.mode(), Density::Comfortable)
    }
}

impl Tokens {
    /// The light comfortable tables.
    fn light() -> Self {
        Self {
            display_family: "sans-serif".to_owned(),
            mono_family: "monospace".to_owned(),
            mode: ThemeMode::Light,
            density: Density::Comfortable,
            primary_hover: rgb(0x2b93f5),
            primary_strong: rgb(0x0a62c2),
            primary_weak: rgb(0xe2f0ff),
            tape_stripe: rgb(0x1a1a1a),
            dropzone_stripe: rgba(0x1283f0, 0.04),
            cyan: rgb(0x2ec4f0),
            yellow: rgb(0xffc928),
            navy: rgb(0x0c1f36),
            canvas: rgb(0xedf2f7),
            surface_3: rgb(0xeaf0f6),
            line_strong: rgb(0xafc0d2),
            text_2: rgb(0x4b5e76),
            text_3: rgb(0x8093a9),
            sidebar: rgb(0x0c1f36),
            sidebar_alt: rgb(0x11294a),
            sidebar_text: rgb(0xcfdceb),
            sidebar_dim: rgb(0x7890ac),
            control_height: 34.,
            row_height: 42.,
            padding: 20.,
        }
    }
}
