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
    let radius = Radii {
        sm: 4.0,
        md: 8.0,
        lg: 14.0,
    };
    let text = Typography {
        size: 14.0,
        size_small: 12.0,
        size_large: 20.0,
        line_height: LineHeight::Scale(1.5),
        ..base.text().clone()
    };
    let mut ui = kivact::ui::UiTokens::default();
    ui.code_block.font_size = 13.0;
    ui.code_block.line_height = 20.0;
    ui.button_group.radius = 8.0;
    ui.button_group.divider = p.line;
    ui.button_group.solid_divider = p.line_strong;
    ui.button_group.text_background = p.surface_muted;
    ui.button_group.text_color = p.text;
    ui.toolbar.background = p.surface;
    ui.toolbar.border = p.line;
    ui.toolbar.padding = 8.0;
    ui.toolbar.gap = 8.0;
    ui.toolbar.min_height = 36.0;
    ui.calendar = kivact::ui::CalendarTokens {
        cell_size: 40.0,
        month_gap: 12.0,
        week_gap: 2.0,
    };
    base.with_mode(mode)
        .with_colors(colors)
        .with_space(space)
        .with_radius(radius)
        .with_text(text)
        .with_focus_ring(3.0)
        .with_control_height(36.0)
        .with_icons(crate::icons::set())
        .with(ui)
        .with(p)
        .with(crate::recipes::recipes())
}
