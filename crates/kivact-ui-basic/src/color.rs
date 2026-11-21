//! Role: The basic kit's color variables (shadcn/ui Neutral) and the values a theme sets for them.

use kivact::style::{Color, Theme, ThemeMode, ThemeVar, rgb, rgba};

/// `--background`: the page.
pub static BACKGROUND_VAR: ThemeVar<Color> =
    ThemeVar::alias("basic.background", &kivact::style::color::SURFACE_VAR);
/// `--background` as a color for a style.
pub const BACKGROUND: Color = Color::var(&BACKGROUND_VAR);

/// `--foreground`: text on the page.
pub static FOREGROUND_VAR: ThemeVar<Color> =
    ThemeVar::alias("basic.foreground", &kivact::style::color::TEXT_VAR);
/// `--foreground` as a color for a style.
pub const FOREGROUND: Color = Color::var(&FOREGROUND_VAR);

/// `--card`: a card's surface.
pub static CARD_VAR: ThemeVar<Color> = ThemeVar::new("basic.card", rgb(0xffffff));
/// `--card` as a color for a style.
pub const CARD: Color = Color::var(&CARD_VAR);

/// `--card-foreground`: text on a card.
pub static CARD_FOREGROUND_VAR: ThemeVar<Color> =
    ThemeVar::new("basic.card_foreground", rgb(0x0a0a0a));
/// `--card-foreground` as a color for a style.
pub const CARD_FOREGROUND: Color = Color::var(&CARD_FOREGROUND_VAR);

/// `--popover`: menus, dialogs and other floating surfaces.
pub static POPOVER_VAR: ThemeVar<Color> =
    ThemeVar::alias("basic.popover", &kivact::style::color::RAISED_VAR);
/// `--popover` as a color for a style.
pub const POPOVER: Color = Color::var(&POPOVER_VAR);

/// `--popover-foreground`: text on a popover.
pub static POPOVER_FOREGROUND_VAR: ThemeVar<Color> =
    ThemeVar::new("basic.popover_foreground", rgb(0x0a0a0a));
/// `--popover-foreground` as a color for a style.
pub const POPOVER_FOREGROUND: Color = Color::var(&POPOVER_FOREGROUND_VAR);

/// `--primary`: the default button and checked controls.
pub static PRIMARY_VAR: ThemeVar<Color> =
    ThemeVar::alias("basic.primary", &kivact::style::color::ACCENT_VAR);
/// `--primary` as a color for a style.
pub const PRIMARY: Color = Color::var(&PRIMARY_VAR);

/// `--primary-foreground`: text on primary.
pub static PRIMARY_FOREGROUND_VAR: ThemeVar<Color> = ThemeVar::alias(
    "basic.primary_foreground",
    &kivact::style::color::ACCENT_TEXT_VAR,
);
/// `--primary-foreground` as a color for a style.
pub const PRIMARY_FOREGROUND: Color = Color::var(&PRIMARY_FOREGROUND_VAR);

/// `--secondary`: the secondary button.
pub static SECONDARY_VAR: ThemeVar<Color> = ThemeVar::new("basic.secondary", rgb(0xf5f5f5));
/// `--secondary` as a color for a style.
pub const SECONDARY: Color = Color::var(&SECONDARY_VAR);

/// `--secondary-foreground`: text on secondary.
pub static SECONDARY_FOREGROUND_VAR: ThemeVar<Color> =
    ThemeVar::new("basic.secondary_foreground", rgb(0x171717));
/// `--secondary-foreground` as a color for a style.
pub const SECONDARY_FOREGROUND: Color = Color::var(&SECONDARY_FOREGROUND_VAR);

/// `--muted`: quiet fills such as tab lists, skeletons and kbd.
pub static MUTED_VAR: ThemeVar<Color> =
    ThemeVar::alias("basic.muted", &kivact::style::color::SURFACE_ALT_VAR);
/// `--muted` as a color for a style.
pub const MUTED: Color = Color::var(&MUTED_VAR);

/// Neutral tinted bubble fill, independently customizable.
pub static BUBBLE_TINTED_VAR: ThemeVar<Color> = ThemeVar::new("basic.bubble_tinted", rgb(0xe8e8e8));
/// [`BUBBLE_TINTED_VAR`] as a color for a style.
pub const BUBBLE_TINTED: Color = Color::var(&BUBBLE_TINTED_VAR);

/// Hover fill of an actionable tinted bubble.
pub static BUBBLE_TINTED_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::new("basic.bubble_tinted_hover", rgb(0xd7d7d7));
/// [`BUBBLE_TINTED_HOVER_VAR`] as a color for a style.
pub const BUBBLE_TINTED_HOVER: Color = Color::var(&BUBBLE_TINTED_HOVER_VAR);

/// `--muted-foreground`: secondary text.
pub static MUTED_FOREGROUND_VAR: ThemeVar<Color> = ThemeVar::alias(
    "basic.muted_foreground",
    &kivact::style::color::TEXT_MUTED_VAR,
);
/// `--muted-foreground` as a color for a style.
pub const MUTED_FOREGROUND: Color = Color::var(&MUTED_FOREGROUND_VAR);

/// `--accent`: the focused menu or select row.
pub static ACCENT_VAR: ThemeVar<Color> = ThemeVar::new("basic.accent", rgb(0xf5f5f5));
/// `--accent` as a color for a style.
pub const ACCENT: Color = Color::var(&ACCENT_VAR);

/// `--accent-foreground`: text on accent.
pub static ACCENT_FOREGROUND_VAR: ThemeVar<Color> =
    ThemeVar::new("basic.accent_foreground", rgb(0x171717));
/// `--accent-foreground` as a color for a style.
pub const ACCENT_FOREGROUND: Color = Color::var(&ACCENT_FOREGROUND_VAR);

/// `--destructive`: destructive actions and errors.
pub static DESTRUCTIVE_VAR: ThemeVar<Color> =
    ThemeVar::alias("basic.destructive", &kivact::style::color::DANGER_VAR);
/// `--destructive` as a color for a style.
pub const DESTRUCTIVE: Color = Color::var(&DESTRUCTIVE_VAR);

/// `--border`: separators and outlines.
pub static BORDER_VAR: ThemeVar<Color> =
    ThemeVar::alias("basic.border", &kivact::style::color::BORDER_VAR);
/// `--border` as a color for a style.
pub const BORDER: Color = Color::var(&BORDER_VAR);

/// `--input`: the borders of fields and unchecked controls.
pub static INPUT_VAR: ThemeVar<Color> = ThemeVar::new("basic.input", rgb(0xe5e5e5));
/// `--input` as a color for a style.
pub const INPUT: Color = Color::var(&INPUT_VAR);

/// `--ring`: the focus indication.
pub static RING_VAR: ThemeVar<Color> = ThemeVar::new("basic.ring", rgb(0xa1a1a1));
/// `--ring` as a color for a style.
pub const RING: Color = Color::var(&RING_VAR);

/// Thumb fill of a switch in either scheme.
pub static SWITCH_THUMB_VAR: ThemeVar<Color> = ThemeVar::new("basic.switch_thumb", Color::WHITE);
/// [`SWITCH_THUMB_VAR`] as a color for a style.
pub const SWITCH_THUMB: Color = Color::var(&SWITCH_THUMB_VAR);

/// `--chart-1`: the first chart color.
pub static CHART_1_VAR: ThemeVar<Color> = ThemeVar::new("basic.chart_1", rgb(0xd4d4d4));
/// [`CHART_1_VAR`] as a color for a style.
pub const CHART_1: Color = Color::var(&CHART_1_VAR);

/// `--chart-2`: the second chart color.
pub static CHART_2_VAR: ThemeVar<Color> = ThemeVar::new("basic.chart_2", rgb(0x737373));
/// [`CHART_2_VAR`] as a color for a style.
pub const CHART_2: Color = Color::var(&CHART_2_VAR);

/// `--chart-3`: the third chart color.
pub static CHART_3_VAR: ThemeVar<Color> = ThemeVar::new("basic.chart_3", rgb(0x525252));
/// [`CHART_3_VAR`] as a color for a style.
pub const CHART_3: Color = Color::var(&CHART_3_VAR);

/// `--chart-4`: the fourth chart color.
pub static CHART_4_VAR: ThemeVar<Color> = ThemeVar::new("basic.chart_4", rgb(0x404040));
/// [`CHART_4_VAR`] as a color for a style.
pub const CHART_4: Color = Color::var(&CHART_4_VAR);

/// `--chart-5`: the fifth chart color.
pub static CHART_5_VAR: ThemeVar<Color> = ThemeVar::new("basic.chart_5", rgb(0x262626));
/// [`CHART_5_VAR`] as a color for a style.
pub const CHART_5: Color = Color::var(&CHART_5_VAR);

/// `--sidebar`: the sidebar surface.
pub static SIDEBAR_VAR: ThemeVar<Color> = ThemeVar::new("basic.sidebar", rgb(0xfafafa));
/// `--sidebar` as a color for a style.
pub const SIDEBAR: Color = Color::var(&SIDEBAR_VAR);

/// `--sidebar-foreground`.
pub static SIDEBAR_FOREGROUND_VAR: ThemeVar<Color> =
    ThemeVar::new("basic.sidebar_foreground", rgb(0x0a0a0a));
/// `--sidebar-foreground` as a color for a style.
pub const SIDEBAR_FOREGROUND: Color = Color::var(&SIDEBAR_FOREGROUND_VAR);

/// `--sidebar-primary`.
pub static SIDEBAR_PRIMARY_VAR: ThemeVar<Color> =
    ThemeVar::new("basic.sidebar_primary", rgb(0x171717));
/// `--sidebar-primary` as a color for a style.
pub const SIDEBAR_PRIMARY: Color = Color::var(&SIDEBAR_PRIMARY_VAR);

/// `--sidebar-primary-foreground`.
pub static SIDEBAR_PRIMARY_FOREGROUND_VAR: ThemeVar<Color> =
    ThemeVar::new("basic.sidebar_primary_foreground", rgb(0xfafafa));
/// `--sidebar-primary-foreground` as a color for a style.
pub const SIDEBAR_PRIMARY_FOREGROUND: Color = Color::var(&SIDEBAR_PRIMARY_FOREGROUND_VAR);

/// `--sidebar-accent`.
pub static SIDEBAR_ACCENT_VAR: ThemeVar<Color> =
    ThemeVar::new("basic.sidebar_accent", rgb(0xf5f5f5));
/// `--sidebar-accent` as a color for a style.
pub const SIDEBAR_ACCENT: Color = Color::var(&SIDEBAR_ACCENT_VAR);

/// `--sidebar-accent-foreground`.
pub static SIDEBAR_ACCENT_FOREGROUND_VAR: ThemeVar<Color> =
    ThemeVar::new("basic.sidebar_accent_foreground", rgb(0x171717));
/// `--sidebar-accent-foreground` as a color for a style.
pub const SIDEBAR_ACCENT_FOREGROUND: Color = Color::var(&SIDEBAR_ACCENT_FOREGROUND_VAR);

/// `--sidebar-border`.
pub static SIDEBAR_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("basic.sidebar_border", rgb(0xe5e5e5));
/// `--sidebar-border` as a color for a style.
pub const SIDEBAR_BORDER: Color = Color::var(&SIDEBAR_BORDER_VAR);

/// `--sidebar-ring`.
pub static SIDEBAR_RING_VAR: ThemeVar<Color> = ThemeVar::new("basic.sidebar_ring", rgb(0xa1a1a1));
/// `--sidebar-ring` as a color for a style.
pub const SIDEBAR_RING: Color = Color::var(&SIDEBAR_RING_VAR);

/// Set every color variable of the kit to its value in `mode`: the Tailwind v4
/// Neutral values of the light or dark scheme.
pub(crate) fn set_colors(theme: Theme, mode: ThemeMode) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let pick = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
    let border = if dark {
        rgba(0xffffff, 0.10)
    } else {
        rgb(0xe5e5e5)
    };
    let input = if dark {
        rgba(0xffffff, 0.15)
    } else {
        rgb(0xe5e5e5)
    };
    let bubble = if dark { 0.180 } else { 0.910 };
    let bubble_hover = if dark { 0.227 } else { 0.843 };
    theme
        .set(&BACKGROUND_VAR, pick(0xffffff, 0x0a0a0a))
        .set(&FOREGROUND_VAR, pick(0x0a0a0a, 0xfafafa))
        .set(&CARD_VAR, pick(0xffffff, 0x171717))
        .set(&CARD_FOREGROUND_VAR, pick(0x0a0a0a, 0xfafafa))
        .set(&POPOVER_VAR, pick(0xffffff, 0x171717))
        .set(&POPOVER_FOREGROUND_VAR, pick(0x0a0a0a, 0xfafafa))
        .set(&PRIMARY_VAR, pick(0x171717, 0xe5e5e5))
        .set(&PRIMARY_FOREGROUND_VAR, pick(0xfafafa, 0x171717))
        .set(&SECONDARY_VAR, pick(0xf5f5f5, 0x262626))
        .set(&SECONDARY_FOREGROUND_VAR, pick(0x171717, 0xfafafa))
        .set(&MUTED_VAR, pick(0xf5f5f5, 0x262626))
        .set(
            &BUBBLE_TINTED_VAR,
            Color::from_rgba_f64(bubble, bubble, bubble, 1.0),
        )
        .set(
            &BUBBLE_TINTED_HOVER_VAR,
            Color::from_rgba_f64(bubble_hover, bubble_hover, bubble_hover, 1.0),
        )
        .set(&MUTED_FOREGROUND_VAR, pick(0x737373, 0xa1a1a1))
        .set(&ACCENT_VAR, pick(0xf5f5f5, 0x262626))
        .set(&ACCENT_FOREGROUND_VAR, pick(0x171717, 0xfafafa))
        .set(&DESTRUCTIVE_VAR, pick(0xe7000b, 0xff6467))
        .set(&BORDER_VAR, border)
        .set(&INPUT_VAR, input)
        .set(&RING_VAR, pick(0xa1a1a1, 0x737373))
        .set(&SWITCH_THUMB_VAR, Color::WHITE)
        .set(&CHART_1_VAR, rgb(0xd4d4d4))
        .set(&CHART_2_VAR, rgb(0x737373))
        .set(&CHART_3_VAR, rgb(0x525252))
        .set(&CHART_4_VAR, rgb(0x404040))
        .set(&CHART_5_VAR, rgb(0x262626))
        .set(&SIDEBAR_VAR, pick(0xfafafa, 0x171717))
        .set(&SIDEBAR_FOREGROUND_VAR, pick(0x0a0a0a, 0xfafafa))
        .set(&SIDEBAR_PRIMARY_VAR, pick(0x171717, 0x1447e6))
        .set(&SIDEBAR_PRIMARY_FOREGROUND_VAR, rgb(0xfafafa))
        .set(&SIDEBAR_ACCENT_VAR, pick(0xf5f5f5, 0x262626))
        .set(&SIDEBAR_ACCENT_FOREGROUND_VAR, pick(0x171717, 0xfafafa))
        .set(&SIDEBAR_BORDER_VAR, border)
        .set(&SIDEBAR_RING_VAR, pick(0xa1a1a1, 0x737373))
}
