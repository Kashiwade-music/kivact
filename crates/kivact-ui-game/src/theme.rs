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
/// supplied through [`theme_with`] and [`ThemeOptions`]: `theme_with(mode, ThemeOptions { density: Density::Compact })`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::style::{ThemeMode};
/// use kivact::style::{provide_theme};
/// use kivact_ui_game::{self as game, Density, ThemeOptions};
///
/// fn console(dark: Signal<bool>, compact: Signal<bool>) -> impl IntoView {
///     provide_theme(
///         move || {
///             let mode = if dark.get() { ThemeMode::Dark } else { ThemeMode::Light };
///             let density = if compact.get() { Density::Compact } else { Density::Comfortable };
///             game::theme_with(mode, ThemeOptions { density })
///         },
///         || text("Ready"),
///     )
/// }
/// ```
pub fn theme(mode: ThemeMode) -> Theme {
    theme_with(mode, ThemeOptions::default())
}

/// Options for the console theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ThemeOptions {
    /// Control and row heights and panel padding.
    pub density: Density,
}

/// Build the console theme with explicit metrics.
pub fn theme_with(mode: ThemeMode, options: ThemeOptions) -> Theme {
    let base = Theme::light();
    let tokens = Tokens::new(mode, options.density);
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
    let radius = Radii {
        sm: 0.,
        md: 0.,
        lg: 0.,
    };
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
    let mut ui = kivact::ui::UiTokens::default();
    ui.progress.motion.determinate_ms = 900;
    ui.progress.motion.determinate_easing = kivact::motion::Easing::CubicBezier(0.2, 0.7, 0.2, 1.);
    ui.indicators.avatar_overlap = 6.;
    ui.indicators.gap = 14.;
    ui.indicators.period_ms = 1400;
    ui.tree.marker_width = Some(10.);
    ui.tree.indent = Some(16.);
    ui.tree.padding = Some(10.);
    ui.tree.gap = Some(6.);
    ui.tree.row_height = Some(26.);
    ui.tree.radius = Some(0.);
    ui.table.cell_padding = Some(12.);
    ui.table.header_height = Some(34.);
    ui.table.font_size = Some(11.5);
    ui.item_list.gap = Some(0.);
    ui.item_list.padding = Some(10.);
    ui.item_list.row_gap = Some(6.);
    ui.item_list.description_size = Some(11.5);
    ui.toolbar.padding = 3.;
    ui.toolbar.gap = 2.;
    ui.toolbar.min_height = 0.;
    ui.toolbar.background = colors.surface;
    ui.toolbar.border = colors.border;
    ui.button_group.gap = 2.;
    ui.button_group.joined = false;
    ui.button_group.divider = tokens.line_strong;
    ui.button_group.solid_divider = colors.accent_text;
    ui.button_group.text_background = tokens.surface_3;
    ui.button_group.text_color = colors.text;
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
        .with(ui)
        .with(crate::recipes::recipes())
}
