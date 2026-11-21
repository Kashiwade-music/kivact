//! Role: The console's theme: its color, length and font variables, its icons and its recipe tables.
use crate::color as kit;
use crate::fonts::{DISPLAY_FAMILY_VAR, MONO_FAMILY_VAR};
use kivact::style::{
    Color, FontFamily, LineHeight, Theme, ThemeMode, color, control, font, radius, rgb, rgba, space,
};
use kivact::ui::theme::{
    SparklineFill, SparklineTheme, avatar_group, button_group, item_list, skeleton, spinner, table,
    toolbar, tree,
};

/// Comfortable or compact metrics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    /// The comfortable metrics: 34 px controls and 42 px rows.
    #[default]
    Comfortable,
    /// The compact metrics: 28 px controls and 32 px rows.
    Compact,
}

/// Options of the console theme. The default options give [`theme`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemeOptions {
    /// The row and control metrics.
    pub density: Density,
    /// The family of headings, buttons and numbers; `None` keeps the theme's family.
    pub display_font: Option<String>,
    /// The family of codes and identifiers; `None` keeps `monospace`.
    pub mono_font: Option<String>,
}

thread_local! {
    /// The light theme of the kit, built once per UI thread on first use.
    static LIGHT: Theme = build(ThemeMode::Light, &ThemeOptions::default());
    /// The dark theme of the kit, built once per UI thread on first use.
    static DARK: Theme = build(ThemeMode::Dark, &ThemeOptions::default());
}

/// The console's theme for `mode`, with the comfortable [`Density`]: the
/// light or dark colors, the metrics, the icons and the recipes of the
/// standard components.
///
/// The same mode returns the same shared theme, so installing it again never
/// restyles. The kit's options come from [`theme_with`].
pub fn theme(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Light => LIGHT.with(Theme::clone),
        ThemeMode::Dark => DARK.with(Theme::clone),
    }
}

/// The console's theme for `mode` with `options`; the default options return [`theme`].
pub fn theme_with(mode: ThemeMode, options: ThemeOptions) -> Theme {
    if options == ThemeOptions::default() {
        theme(mode)
    } else {
        build(mode, &options)
    }
}

fn build(mode: ThemeMode, options: &ThemeOptions) -> Theme {
    let theme = Theme::light().with_mode(mode);
    let theme = kit::set_colors(theme, mode);
    let theme = crate::size::set_sizes(theme, options.density);
    let theme = set_fonts(theme, options);
    let theme = set_core(theme, mode);
    let theme = set_components(theme);
    let theme = theme.with_icons(crate::icons::set());
    install_recipes(theme)
}

/// The kit's font families; the options replace the defaults.
fn set_fonts(theme: Theme, options: &ThemeOptions) -> Theme {
    let theme = theme
        .set(&DISPLAY_FAMILY_VAR, FontFamily::from("sans-serif"))
        .set(&MONO_FAMILY_VAR, FontFamily::from("monospace"));
    let theme = match &options.display_font {
        Some(family) => theme.set(&DISPLAY_FAMILY_VAR, FontFamily::from(family.as_str())),
        None => theme,
    };
    match &options.mono_font {
        Some(family) => theme.set(&MONO_FAMILY_VAR, FontFamily::from(family.as_str())),
        None => theme,
    }
}

/// The standard components' core variables the console sets: the palette of
/// its mode, the spacing, the radii, the type and the control.
fn set_core(theme: Theme, mode: ThemeMode) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let pick = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
    let (overlay, shadow) = if dark {
        (rgba(0x00040a, 0.7), rgba(0x000000, 0.6))
    } else {
        (rgba(0x081424, 0.55), rgba(0x0c1f36, 0.28))
    };
    let raised = if dark {
        &color::SURFACE_ALT_VAR
    } else {
        &color::SURFACE_VAR
    };
    theme
        .set(&color::SURFACE_VAR, pick(0xffffff, 0x0f1c2c))
        .set(&color::SURFACE_ALT_VAR, pick(0xf5f8fb, 0x132438))
        .set(&color::TEXT_VAR, pick(0x13233a, 0xe4edf7))
        .set(&color::TEXT_MUTED_VAR, &kit::TEXT_2_VAR)
        .set(&color::ACCENT_VAR, pick(0x1283f0, 0x2b8ceb))
        .set(&color::ACCENT_TEXT_VAR, rgb(0xffffff))
        .set(&color::BORDER_VAR, pick(0xd6e0ea, 0x1f3550))
        .set(&color::FOCUS_VAR, &kit::YELLOW_VAR)
        .set(&color::SUCCESS_VAR, pick(0x12a06e, 0x2cc48a))
        .set(&color::WARNING_VAR, pick(0xd98300, 0xf5a524))
        .set(&color::DANGER_VAR, pick(0xe0364d, 0xff5a6e))
        .set(&color::OVERLAY_VAR, overlay)
        .set(&color::SHADOW_VAR, shadow)
        .set(&color::SELECTION_VAR, &kit::PRIMARY_WEAK_VAR)
        .set(&color::HOVER_VAR, &color::SURFACE_ALT_VAR)
        .set(&color::PRESSED_VAR, &kit::SURFACE_3_VAR)
        .set(&color::RAISED_VAR, raised)
        .set(&radius::SM_VAR, 0.)
        .set(&radius::MD_VAR, 0.)
        .set(&radius::LG_VAR, 0.)
        .set(&space::XS_VAR, 3.)
        .set(&space::SM_VAR, 6.)
        .set(&space::MD_VAR, 10.)
        .set(&space::LG_VAR, 18.)
        .set(&space::XL_VAR, 20.)
        .set(&font::SIZE_VAR, 13.)
        .set(&font::SIZE_SMALL_VAR, 11.5)
        .set(&font::SIZE_LARGE_VAR, 22.)
        .set(&font::WEIGHT_STRONG_VAR, 700)
        .set(&font::LINE_HEIGHT_VAR, LineHeight::Scale(1.55))
        .set(&control::HEIGHT_VAR, &crate::size::CONTROL_HEIGHT_VAR)
        .set(&control::DISABLED_OPACITY_VAR, 0.42)
        .with_focus_ring(2.)
}

/// The component variables the console sets: the metrics of the indicators,
/// the tree, the table, the lists and the groups.
fn set_components(theme: Theme) -> Theme {
    theme
        .set(&avatar_group::OVERLAP_VAR, 6.)
        .set(&skeleton::GAP_VAR, 14.)
        .set(&skeleton::PERIOD_MS_VAR, 1400.)
        .set(&spinner::PERIOD_MS_VAR, 1400.)
        .set(&tree::MARKER_WIDTH_VAR, 10.)
        .set(&tree::INDENT_VAR, 16.)
        .set(&tree::PADDING_VAR, 10.)
        .set(&tree::GAP_VAR, 6.)
        .set(&tree::ROW_HEIGHT_VAR, 26.)
        .set(&tree::RADIUS_VAR, 0.)
        .set(&table::CELL_PADDING_VAR, 12.)
        .set(&table::HEADER_HEIGHT_VAR, 34.)
        .set(&table::FONT_SIZE_VAR, 11.5)
        .set(&item_list::GAP_VAR, 0.)
        .set(&item_list::PADDING_VAR, 10.)
        .set(&item_list::ROW_GAP_VAR, 6.)
        .set(&item_list::DESCRIPTION_SIZE_VAR, 11.5)
        .set(&toolbar::PADDING_VAR, 3.)
        .set(&toolbar::GAP_VAR, 2.)
        .set(&toolbar::MIN_HEIGHT_VAR, 0.)
        .set(&toolbar::BACKGROUND_VAR, &color::SURFACE_VAR)
        .set(&toolbar::BORDER_VAR, &color::BORDER_VAR)
        .set(&button_group::GAP_VAR, 2.)
        .set(&button_group::DIVIDER_VAR, &kit::LINE_STRONG_VAR)
        .set(&button_group::SOLID_DIVIDER_VAR, &color::ACCENT_TEXT_VAR)
        .set(&button_group::TEXT_BACKGROUND_VAR, &kit::SURFACE_3_VAR)
        .set(&button_group::TEXT_COLOR_VAR, &color::TEXT_VAR)
}

/// Install the recipes of every group the kit styles, in the theme's order.
fn install_recipes(theme: Theme) -> Theme {
    let theme = crate::recipes_controls::install(theme);
    let theme = crate::recipes_overlays::install(theme);
    let theme = crate::recipes_collections::install(theme);
    let theme = crate::recipes_navigation::install(theme);
    let theme = crate::recipes_feedback::install(theme);
    let theme = crate::recipes_metrics::install(theme);
    crate::recipes_input::install(theme)
}

/// The sparkline's fill: the primary tint, solid.
pub(crate) fn sparkline_table() -> SparklineTheme {
    SparklineTheme::default().fill(SparklineFill::Solid(Color::var(&kit::PRIMARY_WEAK_VAR)))
}
