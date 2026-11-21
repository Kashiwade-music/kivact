//! Role: The Pop theme: the kit's variables, the core variables they set, and the recipes of the standard components.

use crate::color;
use crate::icons;
use crate::recipes_calendar;
use crate::recipes_chat;
use crate::recipes_code;
use crate::recipes_collections;
use crate::recipes_controls;
use crate::recipes_feedback;
use crate::recipes_input;
use crate::recipes_metrics;
use crate::recipes_navigation;
use crate::recipes_overlays;
use crate::size;
use kivact::style::color as core;
use kivact::style::{LineHeight, Theme, ThemeMode, control, font, radius, rgba, space};

thread_local! {
    /// The Pop theme in the light mode, built once per UI thread on first use.
    static LIGHT: Theme = build(ThemeMode::Light);
    /// The Pop theme in the dark mode, built once per UI thread on first use.
    static DARK: Theme = build(ThemeMode::Dark);
}

/// The complete Pop theme in `mode`. The same mode returns the same shared
/// theme, so installing it again never restyles.
pub fn theme(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Light => LIGHT.with(Theme::clone),
        ThemeMode::Dark => DARK.with(Theme::clone),
    }
}

fn build(mode: ThemeMode) -> Theme {
    let theme = Theme::light().with_mode(mode);
    let theme = color::set_colors(theme, mode);
    let theme = size::set_sizes(theme);
    let theme = set_core(theme, mode == ThemeMode::Dark);
    let theme = set_components(theme);
    let theme = theme.with_focus_ring(3.0).with_icons(icons::set());
    let theme = recipes_controls::install(theme);
    let theme = recipes_overlays::install(theme);
    let theme = recipes_collections::install(theme);
    let theme = recipes_navigation::install(theme);
    let theme = recipes_feedback::install(theme);
    let theme = recipes_metrics::install(theme);
    let theme = recipes_chat::install(theme);
    let theme = recipes_input::install(theme);
    let theme = recipes_calendar::install(theme);
    recipes_code::install(theme)
}

/// Set the core variables of the theme to the kit's: the palette, the
/// spacing, the radii, the typography and the control height.
fn set_core(theme: Theme, dark: bool) -> Theme {
    theme
        .set(&core::SURFACE_VAR, &color::SURFACE_VAR)
        .set(&core::SURFACE_ALT_VAR, &color::SURFACE_ACTIVE_VAR)
        .set(&core::TEXT_VAR, &color::TEXT_VAR)
        .set(&core::TEXT_MUTED_VAR, &color::TEXT_MUTED_VAR)
        .set(&core::ACCENT_VAR, &color::BLUE_VAR)
        .set(&core::ACCENT_TEXT_VAR, kivact::style::Color::WHITE)
        .set(&core::BORDER_VAR, &color::LINE_VAR)
        .set(&core::FOCUS_VAR, rgba(0x2b8bd8, 0.4))
        .set(&core::DANGER_VAR, &color::RED_VAR)
        .set(&core::SUCCESS_VAR, &color::MINT_VAR)
        .set(&core::WARNING_VAR, &color::YELLOW_VAR)
        .set(&core::OVERLAY_VAR, rgba(0x0f172a, 0.45))
        .set(
            &core::SHADOW_VAR,
            rgba(0x0f172a, if dark { 0.35 } else { 0.07 }),
        )
        .set(&core::SELECTION_VAR, rgba(0x2b8bd8, 0.22))
        .set(&core::HOVER_VAR, &color::SURFACE_ACTIVE_VAR)
        .set(
            &core::PRESSED_VAR,
            if dark {
                &color::LINE_STRONG_VAR
            } else {
                &color::LINE_VAR
            },
        )
        .set(&core::RAISED_VAR, &color::SURFACE_VAR)
        .set(&space::XS_VAR, 4.0)
        .set(&space::SM_VAR, 8.0)
        .set(&space::MD_VAR, 12.0)
        .set(&space::LG_VAR, 16.0)
        .set(&space::XL_VAR, 24.0)
        .set(&radius::SM_VAR, 4.0)
        .set(&radius::MD_VAR, 8.0)
        .set(&radius::LG_VAR, 14.0)
        .set(&font::SIZE_VAR, 14.0)
        .set(&font::SIZE_SMALL_VAR, 12.0)
        .set(&font::SIZE_LARGE_VAR, 20.0)
        .set(&font::LINE_HEIGHT_VAR, LineHeight::Scale(1.5))
        .set(&control::HEIGHT_VAR, 36.0)
}

/// Set the component variables the kit changes: the indicators, the code
/// block, the groups, the toolbar, the sparkline and the calendar.
fn set_components(theme: Theme) -> Theme {
    theme
        .set(&kivact::ui::theme::code_block::FONT_SIZE_VAR, 13.0)
        .set(&kivact::ui::theme::code_block::LINE_HEIGHT_VAR, 20.0)
        .set(&kivact::ui::theme::button_group::RADIUS_VAR, 8.0)
        .set(
            &kivact::ui::theme::button_group::DIVIDER_VAR,
            &color::LINE_VAR,
        )
        .set(
            &kivact::ui::theme::button_group::SOLID_DIVIDER_VAR,
            &color::LINE_STRONG_VAR,
        )
        .set(
            &kivact::ui::theme::button_group::TEXT_BACKGROUND_VAR,
            &color::SURFACE_MUTED_VAR,
        )
        .set(
            &kivact::ui::theme::button_group::TEXT_COLOR_VAR,
            &color::TEXT_VAR,
        )
        .set(
            &kivact::ui::theme::toolbar::BACKGROUND_VAR,
            &color::SURFACE_VAR,
        )
        .set(&kivact::ui::theme::toolbar::BORDER_VAR, &color::LINE_VAR)
        .set(&kivact::ui::theme::toolbar::PADDING_VAR, 8.0)
        .set(&kivact::ui::theme::toolbar::GAP_VAR, 8.0)
        .set(&kivact::ui::theme::toolbar::MIN_HEIGHT_VAR, 36.0)
        .set(&kivact::ui::theme::sparkline::LINE_VAR, &color::BLUE_VAR)
        .set(&kivact::ui::theme::sparkline::LINE_WIDTH_VAR, 2.4)
        .set(&kivact::ui::theme::sparkline::INSET_VAR, 6.0)
        .set(&kivact::ui::theme::calendar::CELL_SIZE_VAR, 40.0)
        .set(&kivact::ui::theme::calendar::MONTH_GAP_VAR, 12.0)
        .set(&kivact::ui::theme::calendar::WEEK_GAP_VAR, 2.0)
}
