//! Role: Install Game colors, metrics and standard UI recipes.
use crate::tokens::{Density, Tokens};
use kivact::style::theme::{Radii, Spacing, Typography};
use kivact::style::{LineHeight, Theme, ThemeMode, rgb, rgba};
/// The console's theme for `mode`, in the comfortable [`Density`]: the
/// light or dark color table, the metrics, the icons and the recipes of the
/// standard components.
///
/// The result is a plain [`Theme`], so it goes to `App::theme`,
/// `provide_theme` and a signal as it is. The kit's options are
/// [`ThemeOptions`] methods on it: `theme(mode).density(Density::Compact)`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::style::{ThemeMode};
/// use kivact::view::{provide_theme};
/// use kivact_ui_game::{self as game, Density, ThemeOptions as _};
///
/// fn console(dark: Signal<bool>, compact: Signal<bool>) -> impl IntoView {
///     provide_theme(
///         move || {
///             let mode = if dark.get() { ThemeMode::Dark } else { ThemeMode::Light };
///             let density = if compact.get() { Density::Compact } else { Density::Comfortable };
///             game::theme(mode).density(density)
///         },
///         || text("Ready"),
///     )
/// }
/// ```
pub fn theme(mode: ThemeMode) -> Theme {
    let base = Theme::light();
    let tokens = Tokens::new(mode, Density::Comfortable);
    let mut colors = base.colors().clone();
    colors.surface = rgb(0xffffff);
    colors.surface_alt = rgb(0xf5f8fb);
    colors.text = rgb(0x13233a);
    colors.text_muted = tokens.text_2;
    colors.accent = rgb(0x1283f0);
    colors.accent_text = rgb(0xffffff);
    colors.border = rgb(0xd6e0ea);
    colors.focus = tokens.yellow;
    colors.success = rgb(0x12a06e);
    colors.warning = rgb(0xd98300);
    colors.danger = rgb(0xe0364d);
    colors.overlay = rgba(0x081424, 0.55);
    colors.shadow = rgba(0x0c1f36, 0.28);
    colors.selection = tokens.primary_weak;
    if mode == ThemeMode::Dark {
        colors.surface = rgb(0x0f1c2c);
        colors.surface_alt = rgb(0x132438);
        colors.text = rgb(0xe4edf7);
        colors.text_muted = tokens.text_2;
        colors.accent = rgb(0x2b8ceb);
        colors.border = rgb(0x1f3550);
        colors.success = rgb(0x2cc48a);
        colors.warning = rgb(0xf5a524);
        colors.danger = rgb(0xff5a6e);
        colors.overlay = rgba(0x00040a, 0.7);
        colors.shadow = rgba(0x000000, 0.6);
        colors.selection = tokens.primary_weak;
    }
    // Rows hover onto the alternate surface and press onto the third; a
    // light panel floats as the surface itself, a dark one one step lighter.
    colors.hover = colors.surface_alt;
    colors.pressed = tokens.surface_3;
    colors.raised = if mode == ThemeMode::Dark {
        colors.surface_alt
    } else {
        colors.surface
    };
    let radius = Radii::new(0., 0., 0.);
    let space = Spacing {
        xs: 3.,
        sm: 6.,
        md: 10.,
        lg: 18.,
        xl: 20.,
    };
    let text = Typography {
        size: 13.,
        size_small: 11.5,
        size_large: 22.,
        weight_strong: 700,
        line_height: LineHeight::Scale(1.55),
        ..base.text().clone()
    };
    base.with_mode(mode)
        .with_colors(colors)
        .with_control_height(tokens.control_height)
        .with_radius(radius)
        .with_space(space)
        .with_text(text)
        .with_focus_ring(2.)
        .with_disabled_opacity(0.42)
        .with_icons(crate::icons::set())
        .with(tokens)
        .with(crate::recipes::recipes())
}

/// The kit's options on a [`Theme`] made by [`theme`].
pub trait ThemeOptions {
    /// Use the metrics of `density`: the control and row heights and the
    /// padding of the console's panels. The colors, the type families and
    /// the rest of the theme's [`Tokens`] stay as they are.
    #[must_use]
    fn density(self, density: Density) -> Theme;
}

impl ThemeOptions for Theme {
    fn density(self, density: Density) -> Theme {
        let metrics = Tokens::new(self.mode(), density);
        let mut tokens = (*self.get::<Tokens>()).clone();
        tokens.density = density;
        tokens.control_height = metrics.control_height;
        tokens.row_height = metrics.row_height;
        tokens.padding = metrics.padding;
        self.with_control_height(metrics.control_height)
            .with(tokens)
    }
}
