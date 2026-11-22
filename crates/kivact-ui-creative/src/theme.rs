//! Role: Construct the Creative theme: its variables, its recipes and its icon set.
use crate::accent::Accent;
use crate::color;
use crate::fonts::MONO_FAMILY_VAR;
use crate::options::Density;
use crate::recipes_collections;
use crate::recipes_controls;
use crate::recipes_feedback;
use crate::recipes_input;
use crate::recipes_metrics;
use crate::recipes_navigation;
use crate::recipes_overlays;
use crate::size::{GAP_VAR, ROW_VAR};
use kivact::style::color as style_color;
use kivact::style::{
    FontFamily, LineHeight, Theme, ThemeMode, control, font, radius, rgb, rgba, space,
};
use kivact::ui::theme::{button_group, toggle_group, toolbar};

/// Options for the Creative theme. Use struct update syntax to keep defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ThemeOptions {
    /// Accent colors for controls, selection and the focus ring.
    pub accent: Accent,
    /// Control row height and gaps.
    pub density: Density,
}

thread_local! {
    /// The default-options light theme, built once per UI thread on first use.
    static LIGHT: Theme = build(ThemeMode::Light, ThemeOptions::default());
    /// The default-options dark theme, built once per UI thread on first use.
    static DARK: Theme = build(ThemeMode::Dark, ThemeOptions::default());
}

/// The Creative theme with the default Iris accent and comfortable density.
pub fn theme(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Light => LIGHT.with(Theme::clone),
        ThemeMode::Dark => DARK.with(Theme::clone),
    }
}

/// Construct the Creative theme with its variables, recipes and icon set.
/// The default options return [`theme`].
pub fn theme_with(mode: ThemeMode, options: ThemeOptions) -> Theme {
    if options == ThemeOptions::default() {
        theme(mode)
    } else {
        build(mode, options)
    }
}

fn build(mode: ThemeMode, options: ThemeOptions) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let theme = if dark { Theme::dark() } else { Theme::light() };
    let theme = color::set_colors(theme, mode, options.accent);
    let theme = crate::size::set_sizes(theme, options.density);
    let theme = set_core(theme, dark);
    let theme = theme.set(&MONO_FAMILY_VAR, FontFamily::from("monospace"));
    let theme = theme.with_focus_ring(2.0).with_icons(crate::icons::set());
    install_recipes(theme)
}

/// Set the core variables and the standard components' variables from the kit's colors and lengths.
fn set_core(theme: Theme, dark: bool) -> Theme {
    let solid_divider = theme.get(&color::ACCENT_INK_VAR).with_alpha(0.22);
    let theme = theme
        .set(&style_color::SURFACE_VAR, &color::ISLAND_VAR)
        .set(
            &style_color::SURFACE_ALT_VAR,
            rgb(if dark { 0x1b1c22 } else { 0xf3f4f6 }),
        );
    let theme = if dark {
        theme.set(&style_color::RAISED_VAR, rgb(0x1b1c22))
    } else {
        theme.set(&style_color::RAISED_VAR, &color::ISLAND_VAR)
    };
    theme
        .set(
            &style_color::HOVER_VAR,
            rgb(if dark { 0x23252c } else { 0xe9eaee }),
        )
        .set(&style_color::PRESSED_VAR, &color::STRONG_VAR)
        .set(&style_color::TEXT_VAR, &color::TEXT_VAR)
        .set(&style_color::TEXT_MUTED_VAR, &color::MUTED_VAR)
        .set(&style_color::BORDER_VAR, &color::HAIR_VAR)
        .set(&style_color::DANGER_VAR, &color::X_VAR)
        .set(&style_color::SUCCESS_VAR, &color::SUCCESS_VAR)
        .set(&style_color::WARNING_VAR, &color::WARNING_VAR)
        .set(
            &style_color::OVERLAY_VAR,
            rgba(0x000000, if dark { 0.45 } else { 0.25 }),
        )
        .set(&style_color::ACCENT_VAR, &color::ACCENT_VAR)
        .set(&style_color::ACCENT_TEXT_VAR, &color::ACCENT_INK_VAR)
        .set(&style_color::FOCUS_VAR, &color::ACCENT_VAR)
        .set(&style_color::SELECTION_VAR, &color::SELECTION_VAR)
        .set(&font::FAMILY_VAR, FontFamily::from("sans-serif"))
        .set(&font::SIZE_VAR, 12.5)
        .set(&font::SIZE_SMALL_VAR, 10.5)
        .set(&font::SIZE_LARGE_VAR, 24.0)
        .set(&font::LINE_HEIGHT_VAR, LineHeight::Scale(1.4))
        .set(&radius::SM_VAR, 5.0)
        .set(&radius::MD_VAR, 7.0)
        .set(&radius::LG_VAR, 12.0)
        .set(&space::MD_VAR, &GAP_VAR)
        .set(&control::HEIGHT_VAR, &ROW_VAR)
        .set(&control::DISABLED_OPACITY_VAR, 0.35)
        .set(&toolbar::BACKGROUND_VAR, &color::HEADER_VAR)
        .set(&toolbar::BORDER_VAR, &color::HAIR_VAR)
        .set(&toolbar::PADDING_VAR, &GAP_VAR)
        .set(&toolbar::GAP_VAR, 4.0)
        .set(&toolbar::MIN_HEIGHT_VAR, &ROW_VAR)
        .set(&toggle_group::GAP_VAR, 4.0)
        .set(&button_group::GAP_VAR, 0.0)
        .set(&button_group::RADIUS_VAR, 7.0)
        .set(&button_group::DIVIDER_VAR, &color::HAIR_VAR)
        .set(&button_group::SOLID_DIVIDER_VAR, solid_divider)
        .set(&button_group::TEXT_BACKGROUND_VAR, &color::FIELD_VAR)
        .set(&button_group::TEXT_COLOR_VAR, &color::TEXT_VAR)
}

/// Install every recipe of the kit into `theme`, in the order of the groups.
fn install_recipes(theme: Theme) -> Theme {
    let theme = recipes_controls::install(theme);
    let theme = recipes_overlays::install(theme);
    let theme = recipes_collections::install(theme);
    let theme = recipes_navigation::install(theme);
    let theme = recipes_feedback::install(theme);
    let theme = recipes_metrics::install(theme);
    recipes_input::install(theme)
}
