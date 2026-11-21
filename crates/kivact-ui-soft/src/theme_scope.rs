//! Role: Scope Neutral media contrast palettes to a subtree.
use crate::color as c;
use crate::theme::theme;
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
                    let alpha = surface.components(&outer)[3];
                    let background = outer.get(&c::BG_SURFACE_VAR);
                    let composite = surface.mix(background, 1.0 - alpha, &outer);
                    let foreground = outer.get(&c::TEXT_PRIMARY_VAR);
                    if composite.contrast_ratio(foreground) >= 3.0 {
                        return outer;
                    }
                    let on_light = outer.get(&c::ON_LIGHT_VAR);
                    if composite.contrast_ratio(outer.get(&c::WHITE_VAR))
                        >= composite.contrast_ratio(on_light)
                    {
                        ThemeMode::Dark
                    } else {
                        ThemeMode::Light
                    }
                }
            };
            let ink = match mode {
                ThemeMode::Dark => &c::WHITE_VAR,
                ThemeMode::Light => &c::ON_LIGHT_VAR,
            };
            theme(mode)
                .set(&c::TEXT_PRIMARY_VAR, ink)
                .set(&c::ICON_PRIMARY_VAR, ink)
                .set(&c::ACCENT_VAR, ink)
        }),
        move || content,
    )
}
