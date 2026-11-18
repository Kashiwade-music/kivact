//! Role: The DevTools app's own theme: neutral colors, square corners and 12 px text.
//!
//! The values are the Lyra Neutral ones the app used through the Basic kit;
//! the standard components keep their own recipes.

use kivact::style::theme::{Palette, Radii, Spacing, Typography};
use kivact::style::{Color, LineHeight, Theme, ThemeMode, rgb, rgba};

/// The app's theme in `mode`.
pub(crate) fn theme(mode: ThemeMode) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let pick = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
    let base = Theme::light();
    let colors = Palette {
        surface: pick(0xffffff, 0x0a0a0a),
        surface_alt: pick(0xf5f5f5, 0x262626),
        text: pick(0x0a0a0a, 0xfafafa),
        text_muted: pick(0x737373, 0xa1a1a1),
        accent: pick(0x171717, 0xe5e5e5),
        accent_text: pick(0xfafafa, 0x171717),
        border: if dark {
            rgba(0xffffff, 0.10)
        } else {
            rgb(0xe5e5e5)
        },
        focus: rgba(if dark { 0x737373 } else { 0xa1a1a1 }, 0.5),
        danger: pick(0xe7000b, 0xff6467),
        success: pick(0x16a34a, 0x4ade80),
        warning: pick(0xd97706, 0xfbbf24),
        overlay: Color::BLACK.with_alpha(0.1),
        shadow: Color::BLACK.with_alpha(0.1),
        selection: rgba(if dark { 0xe5e5e5 } else { 0x171717 }, 0.15),
        hover: pick(0xf5f5f5, 0x262626),
        pressed: pick(0xf5f5f5, 0x262626).mix(pick(0x0a0a0a, 0xfafafa), 0.05),
        raised: pick(0xffffff, 0x171717),
    };
    let text = Typography {
        family: Some("Inter, sans-serif".into()),
        size: 12.0,
        size_small: 12.0,
        size_large: 14.0,
        weight: 400,
        weight_strong: 500,
        line_height: LineHeight::Scale(16.0 / 12.0),
    };
    base.with_mode(mode)
        .with_colors(colors)
        .with_space(Spacing {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 16.0,
            xl: 24.0,
        })
        .with_radius(Radii {
            sm: 0.0,
            md: 0.0,
            lg: 0.0,
        })
        .with_text(text)
        .with_focus_ring(1.0)
        .with_control_height(32.0)
}
