//! Role: Public Halo tokens; no renderer or engine values appear in this API.

use kivact::style::{Color, Theme, ThemeExtension, ThemeMode, rgb, rgba};

/// Semantic colors shared by badges, alerts, avatars and patterns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Hue {
    #[default]
    Blue,
    Mint,
    Yellow,
    Pink,
    Red,
    Gray,
}

/// Typed tokens corresponding to the original CSS custom properties.
#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    pub mode: ThemeMode,
    pub background: Color,
    pub surface: Color,
    pub surface_muted: Color,
    pub surface_active: Color,
    pub line: Color,
    pub line_strong: Color,
    pub text: Color,
    pub text_muted: Color,
    pub text_faint: Color,
    pub blue: Color,
    pub blue_deep: Color,
    pub cyan: Color,
    pub yellow: Color,
    pub pink: Color,
    pub mint: Color,
    pub red: Color,
    pub orange: Color,
    pub purple: Color,
    pub navy: Color,
    pub hue_backgrounds: [Color; 6],
    pub hue_foregrounds: [Color; 6],
    pub panel_padding: f32,
    pub control_height: f32,
    pub control_radius: f32,
    pub panel_radius: f32,
}

impl Tokens {
    /// Original light or dark slate palette, preserving the brand colors.
    pub fn new(mode: ThemeMode) -> Self {
        let dark = mode == ThemeMode::Dark;
        Self {
            mode,
            background: rgb(if dark { 0x0b1120 } else { 0xf8fafc }),
            surface: rgb(if dark { 0x0f172a } else { 0xffffff }),
            surface_muted: rgb(if dark { 0x131c31 } else { 0xf8fafc }),
            surface_active: rgb(if dark { 0x1e293b } else { 0xf1f5f9 }),
            line: rgb(if dark { 0x1e293b } else { 0xe2e8f0 }),
            line_strong: rgb(if dark { 0x334155 } else { 0xcbd5e1 }),
            text: rgb(if dark { 0xf1f5f9 } else { 0x0f172a }),
            text_muted: rgb(if dark { 0x94a3b8 } else { 0x64748b }),
            text_faint: rgb(if dark { 0x64748b } else { 0x94a3b8 }),
            blue: rgb(0x2b8bd8),
            blue_deep: rgb(0x1a66a8),
            cyan: rgb(0x5cbde8),
            yellow: rgb(0xffdc3c),
            pink: rgb(0xf2559a),
            mint: rgb(0x2fcfa6),
            red: rgb(0xef4458),
            orange: rgb(0xffa53c),
            purple: rgb(0x9d86ff),
            navy: rgb(0x1d2b4a),
            hue_backgrounds: if dark {
                [
                    rgba(0x2b8bd8, 0.16),
                    rgba(0x2fcfa6, 0.14),
                    rgba(0xffdc3c, 0.12),
                    rgba(0xff7cb0, 0.14),
                    rgba(0xef4458, 0.16),
                    rgba(0x94a3b8, 0.12),
                ]
            } else {
                [
                    rgb(0xe9f2fb),
                    rgb(0xe5f6f0),
                    rgb(0xfdf4d6),
                    rgb(0xfce8f1),
                    rgb(0xfdeaec),
                    rgb(0xf1f5f9),
                ]
            },
            hue_foregrounds: if dark {
                [
                    rgb(0x7cc0f0),
                    rgb(0x5ddcb9),
                    rgb(0xf5d457),
                    rgb(0xff9cc4),
                    rgb(0xff8a98),
                    rgb(0xa3b1c6),
                ]
            } else {
                [
                    rgb(0x1f6fb8),
                    rgb(0x0f8a6a),
                    rgb(0x8a6a00),
                    rgb(0xc8396f),
                    rgb(0xd22c43),
                    rgb(0x475569),
                ]
            },
            panel_padding: 24.0,
            control_height: 36.0,
            control_radius: 8.0,
            panel_radius: 14.0,
        }
    }

    pub fn hue(&self, hue: Hue) -> (Color, Color) {
        let i = match hue {
            Hue::Blue => 0,
            Hue::Mint => 1,
            Hue::Yellow => 2,
            Hue::Pink => 3,
            Hue::Red => 4,
            Hue::Gray => 5,
        };
        (self.hue_backgrounds[i], self.hue_foregrounds[i])
    }

    pub fn accent(&self, hue: Hue) -> Color {
        match hue {
            Hue::Blue => self.blue,
            Hue::Mint => self.mint,
            Hue::Yellow => self.yellow,
            Hue::Pink => self.pink,
            Hue::Red => self.red,
            Hue::Gray => self.text_muted,
        }
    }
}

impl ThemeExtension for Tokens {
    /// The values of the theme's mode.
    fn fallback(theme: &Theme) -> Self {
        Self::new(theme.mode())
    }
}
