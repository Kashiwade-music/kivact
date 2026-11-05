//! Role: Install the Lyra Neutral palette and complete standard component recipes.

use crate::tokens::Tokens;
use kivact::style::theme::{Palette, Radii, Spacing, Typography};
use kivact::style::{Color, LineHeight, Theme, ThemeMode, rgb};

/// The complete public theme: the facade palette, the typed [`Tokens`] and the
/// Lyra recipes of the standard `kivact::ui` components.
///
/// Radii are zero everywhere, controls are 32 px high and the base text is
/// 12 px, as in the shadcn/ui Lyra preset. Fonts are application-owned: set
/// the `family` of the theme's text (`Theme::with_text`; Inter is
/// recommended) after registering the font.
pub fn theme(mode: ThemeMode) -> Theme {
    let p = Tokens::new(mode);
    let dark = mode == ThemeMode::Dark;
    let base = Theme::light();
    let colors = Palette {
        surface: p.background,
        surface_alt: p.muted,
        text: p.foreground,
        text_muted: p.muted_foreground,
        accent: p.primary,
        accent_text: p.primary_foreground,
        border: p.border,
        focus: p.ring.with_alpha(0.5),
        danger: p.destructive,
        success: rgb(if dark { 0x4ade80 } else { 0x16a34a }),
        warning: rgb(if dark { 0xfbbf24 } else { 0xd97706 }),
        // dialog.tsx: `bg-black/10` in both schemes.
        overlay: Color::BLACK.with_alpha(0.1),
        shadow: Color::BLACK.with_alpha(0.1),
        selection: p.primary.with_alpha(0.15),
        hover: p.muted,
        pressed: p.secondary.mix(p.foreground, 0.05),
        raised: p.popover,
    };
    let space = Spacing {
        xs: 4.0,
        sm: 8.0,
        md: 12.0,
        lg: 16.0,
        xl: 24.0,
    };
    let radius = Radii {
        sm: 0.0,
        md: 0.0,
        lg: 0.0,
    };
    let text = Typography {
        size: 12.0,
        size_small: 12.0,
        size_large: 14.0,
        weight: 400,
        weight_strong: 500,
        line_height: LineHeight::Scale(16.0 / 12.0),
        ..base.text().clone()
    };
    base.with_mode(mode)
        .with_colors(colors)
        .with_space(space)
        .with_radius(radius)
        .with_text(text)
        .with_focus_ring(1.0)
        .with_control_height(32.0)
        .with_icons(crate::icons::set())
        .with(p)
        .with(standard_tokens())
        .with(crate::recipes::recipes())
}

fn standard_tokens() -> kivact::ui::UiTokens {
    kivact::ui::UiTokens {
        chat: kivact::ui::ChatTokens {
            follow_slack: 1.5,
            ..kivact::ui::ChatTokens::default()
        },
        code_block: kivact::ui::CodeBlockTokens {
            font_size: 14.0,
            line_height: 23.0,
            copy_feedback_ms: 2000,
        },
        indicators: kivact::ui::IndicatorTokens {
            reduced_period_ms: Some(1000),
            ..kivact::ui::IndicatorTokens::default()
        },
        calendar: kivact::ui::CalendarTokens {
            cell_size: 28.0,
            month_gap: 16.0,
            week_gap: 8.0,
        },
        combobox: kivact::ui::ComboboxTokens {
            list_height: 252.0,
            popup_offset: 4.0,
            field_radius: Some(0.0),
            chip_radius: 0.0,
            option_radius: 0.0,
            ..kivact::ui::ComboboxTokens::default()
        },
        ..kivact::ui::UiTokens::default()
    }
}
