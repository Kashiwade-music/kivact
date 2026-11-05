//! Role: Construct the Creative theme and install standard UI recipes.
use crate::tokens::{Accent, Density, Tokens};
use kivact::style::theme::{Palette, Radii, Spacing, Typography};
use kivact::style::{LineHeight, Theme, ThemeMode, rgb, rgba};

/// Options for the Creative theme. Use struct update syntax to keep defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ThemeOptions {
    /// Accent colors for controls, selection and the focus ring.
    pub accent: Accent,
    /// Control row height and gaps.
    pub density: Density,
}

/// The Creative theme with the default Iris accent and comfortable density.
pub fn theme(mode: ThemeMode) -> Theme {
    theme_with(mode, ThemeOptions::default())
}

/// Construct the Creative theme with its tokens, recipes and icon set.
pub fn theme_with(mode: ThemeMode, options: ThemeOptions) -> Theme {
    let p = Tokens::new(mode, options.accent, options.density);
    let dark = mode == ThemeMode::Dark;
    let base = if dark { Theme::dark() } else { Theme::light() };
    let colors = Palette {
        surface: p.island,
        surface_alt: rgb(if dark { 0x1b1c22 } else { 0xf3f4f6 }),
        raised: if dark { rgb(0x1b1c22) } else { p.island },
        hover: rgb(if dark { 0x23252c } else { 0xe9eaee }),
        pressed: p.strong,
        text: p.text,
        text_muted: p.muted,
        border: p.hair,
        danger: p.x,
        success: p.success,
        warning: p.warning,
        overlay: rgba(0x000000, if dark { 0.45 } else { 0.25 }),
        ..base.colors().clone()
    };
    let text = Typography {
        family: Some(p.body_font.clone()),
        size: 12.5,
        size_small: 10.5,
        size_large: 24.0,
        line_height: LineHeight::Scale(1.4),
        ..base.text().clone()
    };
    let radius = Radii {
        sm: 5.0,
        md: 7.0,
        lg: 12.0,
    };
    let t = base
        .with_colors(colors)
        .with_text(text)
        .with_radius(radius)
        // The ring every focus stop of the kit and the standard parts shows
        // while the focus is visible, in the accent (`colors.focus`).
        .with_focus_ring(2.0)
        .with_disabled_opacity(0.35)
        .with_icons(crate::icons::set());
    let colors = Palette {
        accent: p.accent,
        accent_text: p.accent_ink,
        focus: p.accent,
        selection: p.selection,
        ..t.colors().clone()
    };
    let space = Spacing {
        md: p.gap,
        ..*t.space()
    };
    t.with_colors(colors)
        .with_space(space)
        .with_control_height(p.row)
        .with(p)
        .with(crate::recipes::recipes())
}
