//! Role: Install the Pop palette and complete standard component recipes.
use crate::tokens::Tokens;
use kivact::style::theme::{Palette, Radii, Spacing, Typography};
use kivact::style::{Color, LineHeight, Theme, ThemeMode, rgba};
/// Install the complete public theme and its typed Pop token table.
pub fn theme(mode: ThemeMode) -> Theme {
    let p = Tokens::new(mode);
    let base = Theme::light();
    let colors = Palette {
        surface: p.surface,
        surface_alt: p.surface_active,
        text: p.text,
        text_muted: p.text_muted,
        accent: p.blue,
        accent_text: Color::WHITE,
        border: p.line,
        focus: rgba(0x2b8bd8, 0.4),
        danger: p.red,
        success: p.mint,
        warning: p.yellow,
        overlay: rgba(0x0f172a, 0.45),
        shadow: rgba(0x0f172a, if mode == ThemeMode::Dark { 0.35 } else { 0.07 }),
        selection: rgba(0x2b8bd8, 0.22),
        hover: p.surface_active,
        pressed: if mode == ThemeMode::Dark {
            p.line_strong
        } else {
            p.line
        },
        raised: p.surface,
    };
    let space = Spacing {
        xs: 4.0,
        sm: 8.0,
        md: 12.0,
        lg: 16.0,
        xl: 24.0,
    };
    let radius = Radii::new(4.0, 8.0, 14.0);
    let text = Typography {
        size: 14.0,
        size_small: 12.0,
        size_large: 20.0,
        line_height: LineHeight::Scale(1.5),
        ..base.text().clone()
    };
    base.with_mode(mode)
        .with_colors(colors)
        .with_space(space)
        .with_radius(radius)
        .with_text(text)
        .with_focus_ring(3.0)
        .with_control_height(36.0)
        .with_icons(crate::icons::set())
        .with(p)
        .with(crate::recipes::recipes())
}
