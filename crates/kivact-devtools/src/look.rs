//! Role: The DevTools app's own theme: neutral colors, square corners and 12 px text.
//!
//! The values are the Lyra Neutral ones the app used through the Basic kit;
//! the standard components keep their own recipes, except the tabs, which
//! stay tight so that the eleven panel tabs fit an 800 px window.

use kivact::interaction::{TabListState, TabState};
use kivact::style::{
    Align, Color, Direction, FontFamily, LineHeight, Style, Theme, ThemeMode, color, control, font,
    pct, radius, rgb, rgba, space, style,
};
use kivact::ui::{Look, UiRecipes};
use kivact::view::Orientation;

/// The app's theme in `mode`.
pub(crate) fn theme(mode: ThemeMode) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let pick = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
    Theme::light()
        .with_mode(mode)
        .set(&color::SURFACE_VAR, pick(0xffffff, 0x0a0a0a))
        .set(&color::SURFACE_ALT_VAR, pick(0xf5f5f5, 0x262626))
        .set(&color::TEXT_VAR, pick(0x0a0a0a, 0xfafafa))
        .set(&color::TEXT_MUTED_VAR, pick(0x737373, 0xa1a1a1))
        .set(&color::ACCENT_VAR, pick(0x171717, 0xe5e5e5))
        .set(&color::ACCENT_TEXT_VAR, pick(0xfafafa, 0x171717))
        .set(
            &color::BORDER_VAR,
            if dark {
                rgba(0xffffff, 0.10)
            } else {
                rgb(0xe5e5e5)
            },
        )
        .set(
            &color::FOCUS_VAR,
            rgba(if dark { 0x737373 } else { 0xa1a1a1 }, 0.5),
        )
        .set(&color::DANGER_VAR, pick(0xe7000b, 0xff6467))
        .set(&color::SUCCESS_VAR, pick(0x16a34a, 0x4ade80))
        .set(&color::WARNING_VAR, pick(0xd97706, 0xfbbf24))
        .set(&color::OVERLAY_VAR, Color::BLACK.with_alpha(0.1))
        .set(&color::SHADOW_VAR, Color::BLACK.with_alpha(0.1))
        .set(
            &color::SELECTION_VAR,
            rgba(if dark { 0xe5e5e5 } else { 0x171717 }, 0.15),
        )
        .set(&color::HOVER_VAR, pick(0xf5f5f5, 0x262626))
        .set(
            &color::PRESSED_VAR,
            pick(0xf5f5f5, 0x262626).mix(pick(0x0a0a0a, 0xfafafa), 0.05, &Theme::light()),
        )
        .set(&color::RAISED_VAR, pick(0xffffff, 0x171717))
        .set(&space::XS_VAR, 4.0)
        .set(&space::SM_VAR, 8.0)
        .set(&space::MD_VAR, 12.0)
        .set(&space::LG_VAR, 16.0)
        .set(&space::XL_VAR, 24.0)
        .set(&radius::SM_VAR, 0.0)
        .set(&radius::MD_VAR, 0.0)
        .set(&radius::LG_VAR, 0.0)
        .set(&font::FAMILY_VAR, FontFamily::from("Inter, sans-serif"))
        .set(&font::SIZE_VAR, 12.0)
        .set(&font::SIZE_SMALL_VAR, 12.0)
        .set(&font::SIZE_LARGE_VAR, 14.0)
        .set(&font::WEIGHT_VAR, 400)
        .set(&font::WEIGHT_STRONG_VAR, 500)
        .set(&font::LINE_HEIGHT_VAR, LineHeight::Scale(16.0 / 12.0))
        .with_focus_ring(1.0)
        .set(&control::HEIGHT_VAR, 32.0)
        .with(UiRecipes::default().tabs(|tabs| tabs.list(|_| tab_list).tab(|_| tab)))
}

/// The tab list: a muted strip with no gap between the tabs.
fn tab_list(_: &Theme, look: &Look<TabListState>) -> Style {
    let base = style()
        .padding(3)
        .background(kivact::style::color::SURFACE_ALT);
    if look.orientation == Orientation::Vertical {
        base.direction(Direction::Column)
    } else {
        base.direction(Direction::Row)
            .align_self(Align::Start)
            .height(32)
    }
}

/// A tab: 6 px on each side, 12 px medium text; the selected one is raised.
fn tab(_: &Theme, look: &Look<TabState>) -> Style {
    let base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .padding_x(6)
        .padding_y(2)
        .border(1, Color::TRANSPARENT)
        .font_size(12)
        .weight(500)
        .text_wrap(false)
        .color(kivact::style::color::TEXT_MUTED);
    let base = if look.orientation == Orientation::Vertical {
        base.width(pct(100))
    } else {
        base.height(pct(100))
    };
    if look.selected {
        base.background(kivact::style::color::SURFACE)
            .color(kivact::style::color::TEXT)
    } else {
        base
    }
}
