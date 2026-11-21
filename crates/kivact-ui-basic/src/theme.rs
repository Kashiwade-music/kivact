//! Role: Install the Lyra Neutral colors and the core variables of the standard components.

use kivact::style::{Color, Theme, ThemeMode, control, font, radius, rgb, space};
use kivact::style::{LineHeight, color};

thread_local! {
    /// The kit's light theme, built once per UI thread on first use.
    static LIGHT: Theme = build(ThemeMode::Light);
    /// The kit's dark theme, built once per UI thread on first use.
    static DARK: Theme = build(ThemeMode::Dark);
}

/// The complete theme of the kit in `mode`. The same mode returns the same
/// shared theme, so installing it again never restyles.
pub fn theme(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Light => LIGHT.with(Theme::clone),
        ThemeMode::Dark => DARK.with(Theme::clone),
    }
}

fn build(mode: ThemeMode) -> Theme {
    let theme = crate::color::set_colors(Theme::light().with_mode(mode), mode);
    let theme = set_core(theme, mode);
    theme.with_icons(crate::icons::set())
}

/// The core variables the kit sets: the standard colors, spacing, radii, text and control
/// metrics, and the standard components' tokens. Colors that are a value of the kit's own
/// variables alias them, so the core variables follow the kit's colors.
fn set_core(theme: Theme, mode: ThemeMode) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let focus = theme.get(&crate::color::RING_VAR).with_alpha(0.5);
    let pressed = theme.get(&crate::color::SECONDARY_VAR).mix(
        theme.get(&crate::color::FOREGROUND_VAR),
        0.05,
        &theme,
    );
    let selection = theme.get(&crate::color::PRIMARY_VAR).with_alpha(0.15);
    let theme = theme
        .set(&color::SURFACE_VAR, &crate::color::BACKGROUND_VAR)
        .set(&color::SURFACE_ALT_VAR, &crate::color::MUTED_VAR)
        .set(&color::TEXT_VAR, &crate::color::FOREGROUND_VAR)
        .set(&color::TEXT_MUTED_VAR, &crate::color::MUTED_FOREGROUND_VAR)
        .set(&color::ACCENT_VAR, &crate::color::PRIMARY_VAR)
        .set(
            &color::ACCENT_TEXT_VAR,
            &crate::color::PRIMARY_FOREGROUND_VAR,
        )
        .set(&color::BORDER_VAR, &crate::color::BORDER_VAR)
        .set(&color::FOCUS_VAR, focus)
        .set(&color::DANGER_VAR, &crate::color::DESTRUCTIVE_VAR)
        .set(
            &color::SUCCESS_VAR,
            rgb(if dark { 0x4ade80 } else { 0x16a34a }),
        )
        .set(
            &color::WARNING_VAR,
            rgb(if dark { 0xfbbf24 } else { 0xd97706 }),
        )
        // dialog.tsx: `bg-black/10` in both schemes.
        .set(&color::OVERLAY_VAR, Color::BLACK.with_alpha(0.1))
        .set(&color::SHADOW_VAR, Color::BLACK.with_alpha(0.1))
        .set(&color::SELECTION_VAR, selection)
        .set(&color::HOVER_VAR, &crate::color::MUTED_VAR)
        .set(&color::PRESSED_VAR, pressed)
        .set(&color::RAISED_VAR, &crate::color::POPOVER_VAR)
        .set(&space::XS_VAR, 4.0)
        .set(&space::SM_VAR, 8.0)
        .set(&space::MD_VAR, 12.0)
        .set(&space::LG_VAR, 16.0)
        .set(&space::XL_VAR, 24.0)
        .set(&radius::SM_VAR, 0.0)
        .set(&radius::MD_VAR, 0.0)
        .set(&radius::LG_VAR, 0.0)
        .set(&font::SIZE_VAR, 12.0)
        .set(&font::SIZE_SMALL_VAR, 12.0)
        .set(&font::SIZE_LARGE_VAR, 14.0)
        .set(&font::WEIGHT_VAR, 400u16)
        .set(&font::WEIGHT_STRONG_VAR, 500u16)
        .set(&font::LINE_HEIGHT_VAR, LineHeight::Scale(16.0 / 12.0))
        .set(&control::HEIGHT_VAR, 32.0)
        .with_focus_ring(1.0);
    set_components(theme)
}

/// The standard components' variables the kit sets (U 変数表).
fn set_components(theme: Theme) -> Theme {
    use kivact::ui::theme::{SpinnerTheme, calendar, chat_thread, code_block, combobox};
    theme
        .set(&chat_thread::FOLLOW_SLACK_VAR, 1.5)
        .set(&code_block::FONT_SIZE_VAR, 14.0)
        .set(&code_block::LINE_HEIGHT_VAR, 23.0)
        .set(&code_block::COPY_FEEDBACK_MS_VAR, 2000.0)
        .with(SpinnerTheme::default().reduced_period(Some(1000.0)))
        .set(&calendar::CELL_SIZE_VAR, 28.0)
        .set(&calendar::MONTH_GAP_VAR, 16.0)
        .set(&calendar::WEEK_GAP_VAR, 8.0)
        .set(&combobox::LIST_HEIGHT_VAR, 252.0)
        .set(&combobox::POPUP_OFFSET_VAR, 4.0)
        .set(&combobox::FIELD_RADIUS_VAR, 0.0)
        .set(&combobox::CHIP_RADIUS_VAR, 0.0)
        .set(&combobox::OPTION_RADIUS_VAR, 0.0)
}
