//! Role: Scope Neutral media contrast and syntax palettes to a subtree.
use crate::{SyntaxColors, Tokens, theme};
use kivact::state::memo;
use kivact::style::Color;
use kivact::style::ThemeMode;
use kivact::style::provide_theme;
use kivact::style::use_theme;
use kivact::view::IntoView;
use kivact::view::View;

/// Contrast policy for content placed over a media surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MediaMode {
    /// Use light foregrounds from the dark Neutral theme.
    Dark,
    /// Use dark foregrounds from the light Neutral theme.
    Light,
    /// Use the supplied surface color, including its alpha over the current surface.
    Auto {
        /// Media surface composited over the current theme surface for contrast selection.
        surface: Color,
    },
    /// Inherit the surrounding theme without a media-specific override.
    Off,
}

/// Draw content using Neutral colors suited to the media underneath it.
///
/// The selected theme tracks surrounding theme changes. `Auto` keeps that theme
/// when its primary text has a contrast ratio of at least 3 against the composited
/// surface; otherwise it selects the more contrasting light or dark foreground.
pub fn media_theme(mode: MediaMode, content: impl IntoView) -> View {
    provide_theme(
        memo(move || {
            let outer = use_theme().get();
            let mode = match mode {
                MediaMode::Dark => ThemeMode::Dark,
                MediaMode::Light => ThemeMode::Light,
                MediaMode::Off => return outer,
                MediaMode::Auto { surface } => {
                    let t = outer.ext::<Tokens>();
                    let foreground = t.colors.text_primary.resolve(&outer);
                    let surface = surface.resolve(&outer);
                    let background = t.colors.bg_surface.resolve(&outer);
                    let mix = |channel: f64, background: f64| {
                        channel * surface.a + background * (1.0 - surface.a)
                    };
                    let surface_luminance = luminance(
                        mix(surface.r, background.r),
                        mix(surface.g, background.g),
                        mix(surface.b, background.b),
                    );
                    if contrast(
                        surface_luminance,
                        luminance(foreground.r, foreground.g, foreground.b),
                    ) >= 3.0
                    {
                        return outer;
                    }
                    let on_light = t.colors.on_light.resolve(&outer);
                    if contrast(surface_luminance, 1.0)
                        >= contrast(
                            surface_luminance,
                            luminance(on_light.r, on_light.g, on_light.b),
                        )
                    {
                        ThemeMode::Dark
                    } else {
                        ThemeMode::Light
                    }
                }
            };
            let media = theme(mode);
            let mut tokens = (*media.ext::<Tokens>()).clone();
            let ink = match mode {
                ThemeMode::Dark => tokens.colors.on_dark,
                ThemeMode::Light => tokens.colors.on_light,
            };
            tokens.colors.text_primary = ink;
            tokens.colors.icon_primary = ink;
            tokens.colors.accent = ink;
            let mut palette = media.colors().clone();
            palette.text = ink;
            palette.accent = ink;
            media.with_colors(palette).with(tokens)
        }),
        move || content,
    )
}

/// Replace only the syntax colors of the active Neutral theme.
///
/// The supplied palette stays fixed within this subtree while all other theme
/// values continue to follow the surrounding theme.
pub fn syntax_theme(colors: SyntaxColors, content: impl IntoView) -> View {
    provide_theme(
        memo(move || {
            let outer = use_theme().get();
            let mut tokens = (*outer.ext::<Tokens>()).clone();
            tokens.colors.syntax = colors;
            outer.with(tokens)
        }),
        move || content,
    )
}

fn luminance(r: f64, g: f64, b: f64) -> f64 {
    let linear = |channel: f64| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}

fn contrast(first: f64, second: f64) -> f64 {
    (first.max(second) + 0.05) / (first.min(second) + 0.05)
}
